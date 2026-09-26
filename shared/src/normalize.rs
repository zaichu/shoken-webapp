/// 銘柄名を正規化する。
///
/// 2つの処理を順に適用する:
/// 1. "K D D I" のように各文字間にスペースが挿入される証券会社CSV向けに、
///    すべてのトークンが1文字の場合はスペースを除去して結合する（"K D D I" → "KDDI"）。
///    複数文字のトークンが含まれる場合（"eMAXIS Slim 全世界株式"）は trim のみ。
/// 2. 全角英数字（Ａ-Ｚ／ａ-ｚ／０-９）を半角に変換する。
///    表示・検索は半角を正準とするため、取込時点でも半角に寄せる。
pub fn normalize_security_name(name: &str) -> String {
    let tokens: Vec<&str> = name.split_whitespace().collect();
    let joined = if tokens.len() > 1 && tokens.iter().all(|t| t.chars().count() == 1) {
        tokens.concat()
    } else {
        name.trim().to_string()
    };
    joined
        .chars()
        .map(|character| match character {
            'Ａ'..='Ｚ' | 'ａ'..='ｚ' | '０'..='９' => {
                char::from_u32(character as u32 - 0xfee0).unwrap_or(character)
            }
            _ => character,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::normalize_security_name;

    #[test]
    fn joins_single_character_tokens() {
        for (input, expected) in [
            ("K D D I", "KDDI"),
            ("I N P E X", "INPEX"),
            ("ト ヨ タ", "トヨタ"),
            ("Ａ Ｂ Ｃ", "ABC"),
            ("Ｋ Ｄ Ｄ Ｉ", "KDDI"),
        ] {
            assert_eq!(normalize_security_name(input), expected);
        }
    }

    #[test]
    fn keeps_multi_character_tokens_after_trim() {
        for (input, expected) in [
            ("eMAXIS Slim 全世界株式", "eMAXIS Slim 全世界株式"),
            ("トヨタ 自動車", "トヨタ 自動車"),
            ("任天堂", "任天堂"),
            ("  KDDI  ", "KDDI"),
            ("", ""),
        ] {
            assert_eq!(normalize_security_name(input), expected);
        }
    }

    #[test]
    fn maps_fullwidth_alphanumerics_to_halfwidth() {
        assert_eq!(normalize_security_name("ＫＤＤＩ１２３"), "KDDI123");
        assert_eq!(
            normalize_security_name("日本株ＡＢＣ１２３"),
            "日本株ABC123"
        );
        // 全角カナ・全角スペースを含まない記号は変換しない
        assert_eq!(normalize_security_name("カＡタ"), "カAタ");
    }
}
