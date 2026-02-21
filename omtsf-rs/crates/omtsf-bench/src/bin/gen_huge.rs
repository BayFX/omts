//! Generates the huge-tier benchmark fixture to disk.
//!
//! Run via `just gen-huge`. The resulting file is written to
//! `target/bench-fixtures/huge.omts.json` (~500 MB) and is loaded by
//! `benches/huge_file.rs` at benchmark time.

use std::error::Error;
use std::fs;
use std::io::BufWriter;

use omtsf_bench::{SizeTier, generate_supply_chain, huge_fixture_path};

fn main() -> Result<(), Box<dyn Error>> {
    let path = huge_fixture_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    eprintln!("Generating Huge tier (~737K nodes)...");
    let file = generate_supply_chain(&SizeTier::Huge.config(42));

    let node_count = file.nodes.len();
    let edge_count = file.edges.len();
    eprintln!("Generated {node_count} nodes, {edge_count} edges");

    eprintln!("Writing to {}...", path.display());
    let out = fs::File::create(&path)?;
    let writer = BufWriter::new(out);
    serde_json::to_writer(writer, &file)?;

    let meta = fs::metadata(&path)?;
    eprintln!("Done: {:.1} MB", meta.len() as f64 / (1024.0 * 1024.0));

    Ok(())
}
