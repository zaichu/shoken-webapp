//! NUMERIC 型の wire codec。`rust_decimal` の postgres 実装は wasm32 では
//! 除外されるため、全ターゲット共通でこの newtype 経由で encode/decode する。

use bytes::{BufMut, BytesMut};
use postgres_types::{FromSql, IsNull, ToSql, Type, to_sql_checked};
use rust_decimal::Decimal;
use std::error::Error;
use std::fmt;

/// `Decimal` の NUMERIC codec ラッパー。`Bind`/`GetCol` が内部で使う
#[derive(Debug, Clone, Copy)]
pub struct Numeric(pub Decimal);

impl From<Decimal> for Numeric {
    fn from(v: Decimal) -> Self {
        Self(v)
    }
}

impl From<Numeric> for Decimal {
    fn from(v: Numeric) -> Self {
        v.0
    }
}

fn err(message: impl Into<String>) -> Box<dyn Error + Sync + Send> {
    message.into().into()
}

const BASE: u128 = 10000;
const SIGN_NEGATIVE: u16 = 0x4000;
const SIGN_NAN: u16 = 0xC000;

/// NUMERIC ヘッダを `out` に書き、digits は base-10000 big-endian で続く
fn encode_numeric(d: &Decimal, out: &mut Vec<u8>) -> Result<(), Box<dyn Error + Sync + Send>> {
    let scale = d.scale();
    let neg = d.is_sign_negative() && !d.is_zero();

    let mut digits: Vec<u16> = Vec::new();
    let mut weight: i32 = 0;
    if !d.is_zero() {
        // 小数部を base-10000 の桁境界まで右にパディングしてからグループ化する
        let frac_groups = scale.div_ceil(4);
        let mut mantissa = d.mantissa().unsigned_abs() * 10u128.pow(frac_groups * 4 - scale);
        while mantissa > 0 {
            digits.push((mantissa % BASE) as u16);
            mantissa /= BASE;
        }
        digits.reverse();
        weight = i32::try_from(digits.len()).map_err(|_| err("NUMERIC の桁数が上限を超えた"))?
            - i32::try_from(frac_groups).map_err(|_| err("NUMERIC の桁数が上限を超えた"))?
            - 1;
        // 後続の 0 グループは値を変えないので削る
        while digits.pop_if(|digit| *digit == 0).is_some() {}
    }

    out.put_i16(i16::try_from(digits.len()).map_err(|_| err("NUMERIC の桁数が上限を超えた"))?);
    out.put_i16(i16::try_from(weight).map_err(|_| err("NUMERIC の weight が範囲外"))?);
    out.put_u16(if neg { SIGN_NEGATIVE } else { 0 });
    out.put_u16(u16::try_from(scale).map_err(|_| err("NUMERIC の scale が範囲外"))?);
    for digit in digits {
        out.put_u16(digit);
    }
    Ok(())
}

/// NUMERIC wire bytes を `Decimal` に戻す。
/// 変換手順は rust_decimal の postgres 実装と同じ（base-10000 桁を整数部・小数部に分けて累積）
fn decode_numeric(raw: &[u8]) -> Result<Decimal, Box<dyn Error + Sync + Send>> {
    if raw.len() < 8 {
        return Err(err("NUMERIC のバイト列が短い"));
    }
    let ndigits = i16::from_be_bytes(raw[0..2].try_into().expect("2バイト取り出し: len>=8で保証"));
    let ndigits_len =
        usize::try_from(ndigits).map_err(|_| err("NUMERIC の桁数とバイト長が不一致"))?;
    if ndigits < 0 || raw.len() != 8 + ndigits_len * 2 {
        return Err(err("NUMERIC の桁数とバイト長が不一致"));
    }
    let weight = i32::from(i16::from_be_bytes(
        raw[2..4].try_into().expect("2バイト取り出し: len>=8で保証"),
    ));
    let sign = u16::from_be_bytes(raw[4..6].try_into().expect("2バイト取り出し: len>=8で保証"));
    let dscale = u16::from_be_bytes(raw[6..8].try_into().expect("2バイト取り出し: len>=8で保証"));
    let neg = match sign {
        0 => false,
        SIGN_NEGATIVE => true,
        SIGN_NAN => return Err(err("NUMERIC NaN は Decimal に変換できない")),
        s => return Err(err(format!("NUMERIC の sign が不正: {s}"))),
    };
    let digits: Vec<i64> = raw[8..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&c| i64::from(u16::from_be_bytes(c)))
        .collect();
    if digits.iter().any(|&d| !(0..10000).contains(&d)) {
        return Err(err("NUMERIC の digit が base-10000 範囲外"));
    }

    let mut digits = digits.into_iter();
    let digits_len =
        i32::try_from(digits.len()).map_err(|_| err("NUMERIC が Decimal の範囲を超えた"))?;
    let fractionals_part_count = digits_len - weight - 1;
    let integers_part_count = weight + 1;

    let mut result = Decimal::ZERO;
    if integers_part_count > 0 {
        let (start_integers, last) = if integers_part_count > digits_len {
            (integers_part_count - digits_len, digits_len)
        } else {
            (0, integers_part_count)
        };
        let last_len =
            usize::try_from(last).map_err(|_| err("NUMERIC が Decimal の範囲を超えた"))?;
        for digit in (&mut digits).take(last_len) {
            result = result
                .checked_mul(Decimal::from_i128_with_scale(10i128.pow(4), 0))
                .ok_or_else(|| err("NUMERIC が Decimal の範囲を超えた"))?;
            result = result
                .checked_add(Decimal::new(digit, 0))
                .ok_or_else(|| err("NUMERIC が Decimal の範囲を超えた"))?;
        }
        let scale_pow = 10i128
            .checked_pow(
                4 * u32::try_from(start_integers)
                    .map_err(|_| err("NUMERIC が Decimal の範囲を超えた"))?,
            )
            .ok_or_else(|| err("NUMERIC が Decimal の範囲を超えた"))?;
        result = result
            .checked_mul(Decimal::from_i128_with_scale(scale_pow, 0))
            .ok_or_else(|| err("NUMERIC が Decimal の範囲を超えた"))?;
    }
    if fractionals_part_count > 0 {
        let start_fractionals = if weight < 0 {
            u32::try_from(-weight).map_err(|_| err("NUMERIC が Decimal の範囲を超えた"))? - 1
        } else {
            0
        };
        for (i, digit) in digits.enumerate() {
            let fract_pow = 4_u32
                .checked_mul(
                    u32::try_from(i).map_err(|_| err("NUMERIC が Decimal の範囲を超えた"))?
                        + 1
                        + start_fractionals,
                )
                .ok_or_else(|| err("NUMERIC が Decimal の範囲を超えた"))?;
            if fract_pow <= Decimal::MAX_SCALE {
                result = result
                    .checked_add(
                        Decimal::new(digit, 0)
                            / Decimal::from_i128_with_scale(10i128.pow(fract_pow), 0),
                    )
                    .ok_or_else(|| err("NUMERIC が Decimal の範囲を超えた"))?;
            } else if fract_pow == Decimal::MAX_SCALE + 4 && digit >= 5000 {
                // 残りの最下位で丸める（rust_decimal の実装と同じ規則）
                result = result
                    .checked_add(
                        Decimal::new(1, 0)
                            / Decimal::from_i128_with_scale(10i128.pow(Decimal::MAX_SCALE), 0),
                    )
                    .ok_or_else(|| err("NUMERIC が Decimal の範囲を超えた"))?;
            }
        }
    }

    result.set_sign_negative(neg);
    result.rescale(u32::from(dscale).min(Decimal::MAX_SCALE));
    Ok(result)
}

impl ToSql for Numeric {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        if !<Self as ToSql>::accepts(ty) {
            return Err(err(format!("NUMERIC として encode できない型: {ty}")));
        }
        let mut buf = Vec::new();
        encode_numeric(&self.0, &mut buf)?;
        out.put_slice(&buf);
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        *ty == Type::NUMERIC
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for Numeric {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        if !<Self as FromSql>::accepts(ty) {
            return Err(err(format!("NUMERIC として decode できない型: {ty}")));
        }
        decode_numeric(raw).map(Numeric)
    }

    fn accepts(ty: &Type) -> bool {
        *ty == Type::NUMERIC
    }
}

impl fmt::Display for Numeric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn round_trip(d: Decimal, expected_scale: u32) {
        let mut buf = Vec::new();
        encode_numeric(&d, &mut buf).unwrap();
        let back = decode_numeric(&buf).unwrap();
        assert_eq!(back, d, "value mismatch for {d}");
        assert_eq!(back.scale(), expected_scale, "scale mismatch for {d}");
    }

    #[test]
    fn numeric_round_trip() {
        round_trip(dec!(0), 0);
        round_trip(dec!(12), 0);
        round_trip(dec!(-12), 0);
        round_trip(dec!(1.5), 1);
        round_trip(dec!(-1.5), 1);
        round_trip(dec!(12.00), 2);
        round_trip(dec!(0.5), 1);
        round_trip(dec!(0.0005), 4);
        round_trip(dec!(0.00005), 5);
        round_trip(dec!(1.0005), 4);
        round_trip(dec!(123456789), 0);
        round_trip(dec!(7000.0), 1);
        round_trip(dec!(2000.00), 2);
        round_trip(Decimal::MAX, 0);
        round_trip(dec!(-9876543210.123456), 6);
        round_trip(dec!(6.67), 2);
    }

    #[test]
    fn numeric_decode_rejects_nan() {
        let nan = [0u8, 0, 0, 0, 0xC0, 0, 0, 0];
        assert!(decode_numeric(&nan).is_err());
    }

    #[test]
    fn numeric_decode_rejects_bad_digit() {
        // ndigits=1, weight=0, sign=0, dscale=0, digit=10000
        let bad = [0, 1, 0, 0, 0, 0, 0, 0, 0x27, 0x10];
        assert!(decode_numeric(&bad).is_err());
    }

    #[test]
    fn numeric_decode_rejects_negative_ndigits() {
        // ndigits=-1 は桁数エラーになる（i16→usize の符号喪失をエラーで扱う）
        let bad = [0xFF, 0xFF, 0, 0, 0, 0, 0, 0];
        assert!(decode_numeric(&bad).is_err());
    }

    #[test]
    fn numeric_decode_boundary_weights() {
        // weight=i16::MAX: 整数部が桁数を大きく上回り、Decimal の範囲を超えてエラー
        let max_weight = [0, 1, 0x7F, 0xFF, 0, 0, 0, 0, 0, 1];
        assert!(decode_numeric(&max_weight).is_err());
        // weight=i16::MIN: 小数部だけの 0 扱いになり、値・桁数は変わらない
        let min_weight = [0, 1, 0x80, 0x00, 0, 0, 0, 0, 0, 1];
        let back = decode_numeric(&min_weight).unwrap();
        assert_eq!(back, Decimal::ZERO);
    }
}
