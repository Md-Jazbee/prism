//! Regenerate the markdown golden fixture:
//! `cargo run -p prism-extract-markdown --example gen_golden`

use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/languages/markdown");
    let src = std::fs::read(root.join("sample.md"))?;
    let mut bundle = prism_extract_markdown::extract("sample.md", &src)?;
    bundle.normalize();
    let json = serde_json::to_string_pretty(&bundle)?;
    std::fs::write(root.join("expected.json"), json + "\n")?;
    println!("wrote {}", root.join("expected.json").display());
    Ok(())
}
