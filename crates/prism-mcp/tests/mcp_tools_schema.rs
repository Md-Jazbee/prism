//! schemas/mcp-tools/v1 must match the Rust allowlist (gap G-11).

use prism_mcp::ALLOWED_TOOLS;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn schema_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas/mcp-tools/v1")
}

#[test]
fn mcp_tool_schemas_match_allowlist() {
    let dir = schema_dir();
    assert!(
        dir.is_dir(),
        "missing schemas/mcp-tools/v1 at {}",
        dir.display()
    );

    let catalog: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("catalog.json")).expect("catalog.json"))
            .expect("catalog json");
    let catalog_tools: BTreeSet<String> = catalog["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|v| v.as_str().expect("tool name").to_string())
        .collect();

    let allow: BTreeSet<String> = ALLOWED_TOOLS.iter().map(|s| (*s).to_string()).collect();
    assert_eq!(
        catalog_tools, allow,
        "catalog.json tools must equal ALLOWED_TOOLS"
    );

    for name in &allow {
        let path = dir.join(format!("{name}.json"));
        assert!(path.is_file(), "missing schema file {}", path.display());
        let schema: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("schema")).expect("json");
        assert_eq!(
            schema["title"].as_str(),
            Some(name.as_str()),
            "title must match tool name in {}",
            path.display()
        );
        assert_eq!(schema["type"], "object");
        assert!(
            schema.get("properties").is_some(),
            "properties required in {}",
            path.display()
        );
    }
}
