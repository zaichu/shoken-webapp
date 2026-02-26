use backend::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let doc = ApiDoc::openapi()
        .to_pretty_json()
        .expect("OpenAPI JSONの生成に失敗しました");
    std::fs::write("../docs/openapi.json", doc).expect("openapi.json の書き込みに失敗しました");
    println!("docs/openapi.json を生成しました");
}
