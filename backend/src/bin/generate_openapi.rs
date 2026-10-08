#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use backend::ApiDoc;
    use utoipa::OpenApi;

    let doc = ApiDoc::openapi()
        .to_pretty_json()
        .expect("OpenAPI JSONの生成に失敗しました");
    std::fs::write("../docs/openapi.json", doc).expect("openapi.json の書き込みに失敗しました");
    println!("docs/openapi.json を生成しました");
}

// worker-build の wasm ビルドでは bin も対象になるため、ネイティブ専用コードを cfg で閉じて空の main を置く
#[cfg(target_arch = "wasm32")]
fn main() {}
