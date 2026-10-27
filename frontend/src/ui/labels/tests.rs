use super::*;
use std::path::{Path, PathBuf};

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

/// ソースから文字列リテラルの中身を取り出す。
/// コメントを読み飛ばすので、画面に出る文言だけを検査できる
fn string_literals(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut literals = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/')
                {
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
                        literals.push(rest[..end].to_string());
                        index = cursor + 1 + end + terminator.len();
                    }
                    None => index = bytes.len(),
                }
            }
            b'"' => {
                let (value, next) = read_string(source, index);
                literals.push(value);
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
    assert_eq!(price_as_of("2026-10-21"), "10/21 時点");
}
