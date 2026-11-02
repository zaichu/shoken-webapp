use super::*;
use std::path::{Path, PathBuf};

#[test]
fn previous_dividend_span_collapses_single_month_and_ends_in_december() {
    let amount = "123,456";
    for month in [1, 9, 10] {
        let text = previous_dividend_span(month, amount);
        assert!(text.contains(&format!("{}〜12", month + 1)));
        assert!(text.contains(amount));
    }
    let single_month = previous_dividend_span(11, amount);
    assert!(single_month.contains("12"));
    assert!(single_month.contains(amount));
    assert!(!single_month.contains('〜'));
    assert!(previous_dividend_span(12, amount).is_empty());
}

fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).expect("src ディレクトリを読める");
    for entry in entries {
        let path = entry.expect("ディレクトリ項目を読める").path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

/// 検査そのものと用語集(禁止語の一覧を含む)は検査対象から外す
fn is_glossary_file(path: &Path) -> bool {
    let relative = path.strip_prefix(source_root()).unwrap_or(path);
    relative == Path::new("ui/labels.rs") || relative.starts_with("ui/labels")
}

/// 閉じ引用符の位置。文字リテラルでない(ライフタイム)場合は None
fn char_literal_end(source: &str, index: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    match bytes.get(index + 1) {
        Some(b'\\') => {
            let mut cursor = index + 2;
            while cursor < bytes.len() && bytes[cursor] != b'\'' && bytes[cursor] != b'\n' {
                cursor += 1;
            }
            (bytes.get(cursor) == Some(&b'\'')).then_some(cursor + 1)
        }
        Some(_) => (bytes.get(index + 2) == Some(&b'\'')).then_some(index + 3),
        None => None,
    }
}

fn read_string(source: &str, index: usize) -> (String, usize) {
    let bytes = source.as_bytes();
    let mut value = String::new();
    let mut cursor = index + 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => {
                if let Some(character) = source[cursor + 1..].chars().next() {
                    value.push(character);
                    cursor += 1 + character.len_utf8();
                } else {
                    cursor += 1;
                }
            }
            b'"' => return (value, cursor + 1),
            _ => {
                let character = source[cursor..].chars().next().unwrap();
                value.push(character);
                cursor += character.len_utf8();
            }
        }
    }
    (value, cursor)
}

/// ソースから文字列リテラルの中身と開始行を取り出す。
/// コメントを読み飛ばすので、画面に出る文言だけを検査できる
fn string_literals_with_lines(source: &str) -> Vec<(String, usize)> {
    let bytes = source.as_bytes();
    let mut literals = Vec::new();
    let mut index = 0;
    let mut line = 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                line += 1;
                index += 1;
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/')
                {
                    if bytes[index] == b'\n' {
                        line += 1;
                    }
                    index += 1;
                }
                index = (index + 2).min(bytes.len());
            }
            b'r' => {
                let mut cursor = index + 1;
                while bytes.get(cursor) == Some(&b'#') {
                    cursor += 1;
                }
                if bytes.get(cursor) != Some(&b'"') {
                    index += 1;
                    continue;
                }
                let terminator = format!("\"{}", "#".repeat(cursor - index - 1));
                let rest = &source[cursor + 1..];
                match rest.find(&terminator) {
                    Some(end) => {
                        let value = rest[..end].to_string();
                        let start = line;
                        line += value.matches('\n').count();
                        literals.push((value, start));
                        index = cursor + 1 + end + terminator.len();
                    }
                    None => index = bytes.len(),
                }
            }
            b'"' => {
                let (value, next) = read_string(source, index);
                let start = line;
                line += value.matches('\n').count();
                literals.push((value, start));
                index = next;
            }
            b'\'' => match char_literal_end(source, index) {
                Some(end) => index = end,
                None => index += 1,
            },
            _ => index += 1,
        }
    }
    literals
}

fn string_literals(source: &str) -> Vec<String> {
    string_literals_with_lines(source)
        .into_iter()
        .map(|(value, _)| value)
        .collect()
}

fn contains_japanese(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c, '\u{3000}'..='\u{303F}' | '\u{3040}'..='\u{30FF}' | '\u{4E00}'..='\u{9FFF}' | '\u{FF00}'..='\u{FFEF}')
    })
}

fn is_test_code_path(relative: &Path) -> bool {
    let text = relative.to_string_lossy().replace('\\', "/");
    text.ends_with("/tests.rs")
        || text.contains("/tests/")
        || text == "testing.rs"
        || text.starts_with("testing/")
}

fn is_delivery_excluded_line(line: &str) -> bool {
    const MARKERS: &[&str] = &[
        "console::",
        "JsValue::from_str",
        "log!",
        "tracing::",
        "eprintln!",
        "println!",
        "dbg!",
        "expect(",
        "panic!(",
        "unreachable!(",
        "unimplemented!(",
    ];
    MARKERS.iter().any(|marker| line.contains(marker))
}

fn blank_range(source: &mut [u8], start: usize, end: usize) {
    let end = end.min(source.len());
    for byte in &mut source[start..end] {
        if *byte != b'\n' {
            *byte = b' ';
        }
    }
}

fn blank_cfg_test_regions(source: &str) -> String {
    let mut blanked = source.as_bytes().to_vec();
    let mut search_from = 0;
    while let Some(offset) = source[search_from..].find("#[cfg(test)]") {
        let attribute = search_from + offset;
        let mut cursor = attribute + "#[cfg(test)]".len();
        while cursor < source.len() && source.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if source[cursor..].starts_with("mod")
            && source[cursor + 3..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_')
        {
            let mut rest = cursor + 3;
            while rest < source.len() && source.as_bytes()[rest].is_ascii_whitespace() {
                rest += 1;
            }
            while rest < source.len()
                && (source.as_bytes()[rest].is_ascii_alphanumeric()
                    || source.as_bytes()[rest] == b'_')
            {
                rest += 1;
            }
            while rest < source.len() && source.as_bytes()[rest].is_ascii_whitespace() {
                rest += 1;
            }
            if source[rest..].starts_with('{') {
                let mut depth = 0;
                let mut end = rest;
                let mut in_string = false;
                while end < source.len() {
                    let byte = source.as_bytes()[end];
                    if in_string {
                        if byte == b'\\' {
                            end += 1;
                        } else if byte == b'"' {
                            in_string = false;
                        }
                    } else if byte == b'"' {
                        in_string = true;
                    } else if byte == b'{' {
                        depth += 1;
                    } else if byte == b'}' {
                        depth -= 1;
                        if depth == 0 {
                            end += 1;
                            break;
                        }
                    }
                    end += 1;
                }
                blank_range(&mut blanked, attribute, end);
                search_from = end;
                continue;
            }
            search_from = rest;
            continue;
        }
        if let Some(body) = source[cursor..].find('{') {
            let semi = source[cursor..].find(';');
            if semi.is_some_and(|position| position < body) {
                search_from = cursor + semi.unwrap() + 1;
                continue;
            }
            let mut depth = 0;
            let mut end = cursor + body;
            while end < source.len() {
                match source.as_bytes()[end] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            end += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                end += 1;
            }
            blank_range(&mut blanked, attribute, end);
            search_from = end;
            continue;
        }
        search_from = cursor;
    }
    String::from_utf8(blanked).expect("空白化は改行を保つ")
}

fn escape_baseline_literal(literal: &str) -> String {
    literal
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn baseline_path() -> PathBuf {
    source_root().join("ui/labels/baseline.txt")
}

fn collect_japanese_entries() -> Vec<String> {
    let mut files = Vec::new();
    collect_rs_files(&source_root(), &mut files);
    files.sort();
    let mut entries = Vec::new();
    for file in files {
        if is_glossary_file(&file) {
            continue;
        }
        let relative = file.strip_prefix(source_root()).unwrap_or(&file);
        if is_test_code_path(relative) {
            continue;
        }
        let source = std::fs::read_to_string(&file).expect("ソースを読める");
        let lines: Vec<&str> = source.lines().collect();
        let blanked = blank_cfg_test_regions(&source);
        for (literal, line) in string_literals_with_lines(&blanked) {
            if !contains_japanese(&literal) {
                continue;
            }
            if lines
                .get(line.saturating_sub(1))
                .is_some_and(|text| is_delivery_excluded_line(text))
            {
                continue;
            }
            entries.push(format!(
                "{}\t{}",
                relative.to_string_lossy().replace('\\', "/"),
                escape_baseline_literal(&literal)
            ));
        }
    }
    entries.sort();
    entries
}

fn load_baseline_entries(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).expect("基準ファイルを読める");
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn diff_entries(baseline: &[String], actual: &[String]) -> (Vec<String>, Vec<String>) {
    let mut counts: std::collections::HashMap<&str, (usize, usize)> =
        std::collections::HashMap::new();
    for entry in baseline {
        counts.entry(entry.as_str()).or_default().0 += 1;
    }
    for entry in actual {
        counts.entry(entry.as_str()).or_default().1 += 1;
    }
    let mut added = Vec::new();
    let mut removed = Vec::new();
    for (entry, (base, now)) in &counts {
        for _ in 0..now.saturating_sub(*base) {
            added.push((*entry).to_string());
        }
        for _ in 0..base.saturating_sub(*now) {
            removed.push((*entry).to_string());
        }
    }
    added.sort();
    removed.sort();
    (added, removed)
}

#[test]
fn forbidden_terms_are_not_used_outside_the_glossary() {
    let mut files = Vec::new();
    collect_rs_files(&source_root(), &mut files);
    let mut found = Vec::new();
    for file in files {
        if is_glossary_file(&file) {
            continue;
        }
        let source = std::fs::read_to_string(&file).expect("ソースを読める");
        for literal in string_literals(&source) {
            for (term, current) in FORBIDDEN_TERMS {
                if literal.contains(term) {
                    found.push(format!(
                        "{}: 「{term}」→ 現在の用語は「{current}」/ 該当: {literal}",
                        file.display()
                    ));
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "統一前の語が用語集の外に残っています:\n{}",
        found.join("\n")
    );
}

#[test]
fn glossary_builds_counts_and_placeholders() {
    assert_eq!(count(4), "4件");
    assert_eq!(count(0), "0件");
    assert_eq!(count(1_000_000), "1000000件");
    assert_eq!(search_placeholder("銘柄名・コード"), "銘柄名・コードで検索");
    assert_eq!(group_count("2026年10月", 3), "2026年10月 · 3件");
    assert_eq!(price_as_of("2026-10-21"), "価格基準日 10/21");
}

#[test]
fn japanese_literals_match_baseline() {
    let path = baseline_path();
    let mut actual = collect_japanese_entries();
    actual.sort();
    if std::env::var("UPDATE_LABELS_BASELINE").is_ok() {
        let mut text = String::from(
            "# 画面の直書き(用語集の外の日本語リテラル)の基準。1行1件で `相対パス<TAB>文言`。\n\
             # 新しい直書きは検査で落とす。減ったらこのファイルを更新する(減る方向だけ許す)。\n\
             # 更新は UPDATE_LABELS_BASELINE=1 cargo test japanese_literals_match_baseline で作り直す。\n",
        );
        for entry in &actual {
            text.push_str(entry);
            text.push('\n');
        }
        std::fs::write(&path, text).expect("基準ファイルを書ける");
        return;
    }
    let baseline = load_baseline_entries(&path);
    let (added, removed) = diff_entries(&baseline, &actual);
    assert!(
        added.is_empty(),
        "用語集の外に新しい日本語の直書きがあります。ui/labels.rs に移してください:\n{}",
        added.join("\n")
    );
    assert!(
        removed.is_empty(),
        "直書きが減りました。UPDATE_LABELS_BASELINE=1 cargo test japanese_literals_match_baseline で基準ファイルを更新してください:\n{}",
        removed.join("\n")
    );
}

#[test]
fn detects_japanese_string_literals() {
    let source = "//! コメントの日本語は数えない\nlet title = \"設定\";\nlet code = 'a';\nlet lifetime = \"x\";\nlet raw = r#\"検索\"#;\nlet plain = \"plain\";\n";
    let found: Vec<String> = string_literals_with_lines(source)
        .into_iter()
        .filter(|(value, _)| contains_japanese(value))
        .map(|(value, _)| value)
        .collect();
    assert_eq!(found, vec!["設定".to_string(), "検索".to_string()]);
    assert!(!contains_japanese("— … ⋯ ›"));
    assert!(!contains_japanese("plain"));
}

#[test]
fn excludes_test_and_delivery_noise() {
    assert!(is_test_code_path(Path::new("ui/badge/tests.rs")));
    assert!(is_test_code_path(Path::new("features/home/view/tests.rs")));
    assert!(is_test_code_path(Path::new(
        "features/receipts/tests/suite.rs"
    )));
    assert!(is_test_code_path(Path::new("testing/receipts.rs")));
    assert!(!is_test_code_path(Path::new("features/settings.rs")));
    assert!(is_delivery_excluded_line(
        "web_sys::console::warn_2(&JsValue::from_str(\"失敗\"), &error);"
    ));
    assert!(is_delivery_excluded_line(
        "let name = find(role).expect(\"列が設定に存在する\");"
    ));
    assert!(!is_delivery_excluded_line("let title = \"設定\";"));
    let source = "#[cfg(test)]\nmod tests {\n    let title = \"設定\";\n}\nlet body = \"表示\";\n";
    let blanked = blank_cfg_test_regions(source);
    let found: Vec<String> = string_literals_with_lines(&blanked)
        .into_iter()
        .filter(|(value, _)| contains_japanese(value))
        .map(|(value, _)| value)
        .collect();
    assert_eq!(found, vec!["表示".to_string()]);
}

#[test]
fn baseline_diff_reports_added_and_removed() {
    let baseline = vec!["a\t表示".to_string(), "b\t設定".to_string()];
    let same = baseline.clone();
    assert_eq!(diff_entries(&baseline, &same), (Vec::new(), Vec::new()));
    let (added, removed) = diff_entries(&baseline, &["a\t表示".to_string(), "c\t検索".to_string()]);
    assert_eq!(added, vec!["c\t検索".to_string()]);
    assert_eq!(removed, vec!["b\t設定".to_string()]);
    let (_, removed_only) = diff_entries(&baseline, &["a\t表示".to_string()]);
    assert_eq!(removed_only, vec!["b\t設定".to_string()]);
}
