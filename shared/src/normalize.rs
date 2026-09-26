/// 取込時の銘柄名正規化。
///
/// "K D D I" のように各文字間にスペースが挿入される証券会社CSV向けに、
/// すべてのトークンが1文字の場合はスペースを除去して結合する（"K D D I" → "KDDI"）。
/// 複数文字のトークンが含まれる場合（"eMAXIS Slim 全世界株式"）は trim のみ。
///
/// 一意制約と content_hash はこの規則で正規化された名前をキーにしている。
/// 半角化などを追加する場合は既存行の名前・ハッシュを移行する必要がある。
pub fn normalize_security_name(name: &str) -> String {
    let tokens: Vec<&str> = name.split_whitespace().collect();
    if tokens.len() > 1 && tokens.iter().all(|t| t.chars().count() == 1) {
        tokens.concat()
    } else {
        name.trim().to_string()
    }
}

/// 表示・照合用の銘柄名正規化。全角英数字（Ａ-Ｚ／ａ-ｚ／０-９）を半角に直す。
/// 取込時の正規化とは用途が違うため、trim やトークン結合は行わない。
pub fn normalize_display_name(name: &str) -> String {
    name.chars()
        .map(|character| match character {
            'Ａ'..='Ｚ' | 'ａ'..='ｚ' | '０'..='９' => {
                char::from_u32(character as u32 - 0xfee0).unwrap_or(character)
            }
            _ => character,
        })
        .collect()
}

/// 証券コード正規化。`:`・`：` 以降を捨て、空白をすべて除去して大文字化する
/// （" 7974: 任天堂 " → "7974"、"brk.b" → "BRK.B"）。
pub fn normalize_security_code(value: &str) -> String {
    value
        .split([':', '：'])
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .collect::<String>()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::{normalize_display_name, normalize_security_code, normalize_security_name};

    #[test]
    fn security_name_joins_single_character_tokens() {
        for (input, expected) in [
            ("K D D I", "KDDI"),
            ("I N P E X", "INPEX"),
            ("ト ヨ タ", "トヨタ"),
            // 全角の1文字トークンも結合するが半角にはしない
            ("Ａ Ｂ Ｃ", "ＡＢＣ"),
            ("Ｋ Ｄ Ｄ Ｉ", "ＫＤＤＩ"),
        ] {
            assert_eq!(normalize_security_name(input), expected);
        }
    }

    #[test]
    fn security_name_keeps_multi_character_tokens_after_trim() {
        for (input, expected) in [
            ("eMAXIS Slim 全世界株式", "eMAXIS Slim 全世界株式"),
            ("トヨタ 自動車", "トヨタ 自動車"),
            ("任天堂", "任天堂"),
            ("  KDDI  ", "KDDI"),
            // 重複排除キーの互換のため全角はそのまま保持する
            ("ＥＮＥＯＳホールディングス", "ＥＮＥＯＳホールディングス"),
            ("", ""),
        ] {
            assert_eq!(normalize_security_name(input), expected);
        }
    }

    #[test]
    fn display_name_maps_fullwidth_alphanumerics_to_halfwidth() {
        assert_eq!(normalize_display_name("ＫＤＤＩ１２３"), "KDDI123");
        assert_eq!(normalize_display_name("日本株ＡＢＣ１２３"), "日本株ABC123");
        assert_eq!(normalize_display_name("カＡタ"), "カAタ");
        // 表示用は変換のみ。trim・結合はしない
        assert_eq!(normalize_display_name(" J T "), " J T ");
    }

    #[test]
    fn security_code_takes_head_before_colon_and_uppercases() {
        for (input, expected) in [
            (" 7974: 任天堂 ", "7974"),
            ("7203: トヨタ自動車", "7203"),
            ("6758：ソニー", "6758"),
            ("brk.b", "BRK.B"),
            ("a b 1: x", "AB1"),
        ] {
            assert_eq!(normalize_security_code(input), expected);
        }
    }

    #[test]
    fn security_code_handles_empty_and_leading_colon() {
        // 早期 return の有無で分岐し得る境界入力
        for (input, expected) in [("", ""), ("  ", ""), (" : ABC", ""), ("：", "")] {
            assert_eq!(normalize_security_code(input), expected);
        }
    }
}
