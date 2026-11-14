#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), String> {
    use backend::ApiDoc;
    use utoipa::OpenApi;

    let doc = ApiDoc::openapi()
        .to_pretty_json()
        .map_err(|e| format!("OpenAPI JSONの生成に失敗しました: {e}"))?;
    std::fs::write("../docs/openapi.json", doc)
        .map_err(|e| format!("openapi.json の書き込みに失敗しました: {e}"))?;
    println!("docs/openapi.json を生成しました");
    Ok(())
}

// worker-build の wasm ビルドでは bin も対象になるため、ネイティブ専用コードを cfg で閉じて空の main を置く
#[cfg(target_arch = "wasm32")]
fn main() {}
