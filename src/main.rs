use std::error::Error;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// First file containing traces to unify
    trace_file_1: std::path::PathBuf,
    /// Second file containing traces to unify
    trace_file_2: std::path::PathBuf,
    /// File providing all valid basic blocks which are used to unify trace files
    valid_bb_file: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    // TODO: Read and parse all files

    // TODO: Auto-detect trace format ((mmio?), bb, (ram?))

    // TODO: Rewrite by removing all invalid basic blocks

    Ok(())
}
