use schemars::schema::{InstanceType, RootSchema, SchemaObject};
use std::path::PathBuf;
use typify::{TypeSpace, TypeSpaceSettings};

const WRAPPER: &str = "PaginatedSearchResponse_DomesticStock_DomesticStockSummary_SearchFacets";
const DEFINITIONS: &[&str] = &[
    "DomesticStock",
    "DomesticStockSummary",
    "SearchFacets",
    "FacetOption",
];

fn rewrite_refs(text: &str) -> String {
    text.replace("#/components/schemas/", "#/definitions/")
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let spec_path = manifest.join("../docs/openapi.json");
    println!("cargo::rerun-if-changed={}", spec_path.display());
    println!("cargo::rerun-if-changed=build.rs");

    let spec: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&spec_path).expect("openapi.json"))
            .expect("valid openapi.json");
    let components = spec
        .pointer("/components/schemas")
        .and_then(serde_json::Value::as_object)
        .expect("components/schemas");

    let mut root = components[WRAPPER].clone();
    root["title"] = serde_json::Value::String("DomesticStockListResponse".to_string());
    root["properties"]["data"]["items"] =
        serde_json::json!({ "$ref": "#/definitions/DomesticStock" });
    root["properties"]["summary"] =
        serde_json::json!({ "$ref": "#/definitions/DomesticStockSummary" });
    root["properties"]["facets"] = serde_json::json!({ "$ref": "#/definitions/SearchFacets" });
    let mut root: serde_json::Value =
        serde_json::from_str(&rewrite_refs(&serde_json::to_string(&root).expect("wrap")))
            .expect("rewrite wrapper refs");

    let mut definitions = serde_json::Map::new();
    for name in DEFINITIONS {
        let text =
            rewrite_refs(&serde_json::to_string(&components[*name]).expect("serialize definition"));
        definitions.insert(
            (*name).to_string(),
            serde_json::from_str(&text).expect("parse definition"),
        );
    }
    root["definitions"] = serde_json::Value::Object(definitions);

    let root_schema: RootSchema = serde_json::from_value(root).expect("RootSchema");

    let mut settings = TypeSpaceSettings::default();
    settings.with_conversion(
        SchemaObject {
            instance_type: Some(InstanceType::Number.into()),
            format: Some("double".to_string()),
            ..Default::default()
        },
        "::rust_decimal::Decimal",
        std::iter::empty(),
    );
    let mut type_space = TypeSpace::new(&settings);
    type_space
        .add_root_schema(root_schema)
        .expect("typify generation");

    eprintln!(
        "typify uses chrono={} uuid={} serde_json={}",
        type_space.uses_chrono(),
        type_space.uses_uuid(),
        type_space.uses_serde_json()
    );

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    std::fs::write(out.join("dto_gen.rs"), type_space.to_stream().to_string())
        .expect("write dto_gen.rs");
}
