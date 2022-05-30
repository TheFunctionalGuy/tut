use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use anyhow::{Context, Result};
use clap::Parser;

struct Trace {
    ids: Vec<usize>,
    program_counters: Vec<usize>,
    hit_counters: Vec<usize>,
}

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

fn parse_bb_trace(file: File, valid_bb: &[usize]) -> Result<Trace> {
    let reader = BufReader::new(file);
    let mut ids = Vec::new();
    let mut program_counters = Vec::new();
    let mut hit_counters = Vec::new();

    for line in reader.lines().map(|l| l.unwrap()) {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        let id = usize::from_str_radix(parts[0], 16)?;
        let pc = usize::from_str_radix(parts[1], 16)?;
        let hit_count = parts[2].parse::<usize>()?;

        ids.push(id);

        if valid_bb.contains(&pc) {
            program_counters.push(pc);
            hit_counters.push(hit_count);
        }
    }

    // Truncate IDs
    ids.truncate(program_counters.len());

    // Ensure integrity
    assert_eq!(ids.len(), program_counters.len());
    assert_eq!(program_counters.len(), hit_counters.len());

    Ok(Trace {
        ids,
        program_counters,
        hit_counters,
    })
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // TODO: Read and parse all files
    let valid_bb_file = File::open(&args.valid_bb_file)
        .with_context(|| format!("Could not read file {:?}", &args.valid_bb_file))?;
    let valid_bb: Vec<usize> = BufReader::new(valid_bb_file)
        .lines()
        .map(|l| l.unwrap())
        .filter_map(|l| usize::from_str_radix(&l, 16).ok())
        .collect();

    // Only Read valid traces from valid BBs
    let trace_file_1 = File::open(&args.trace_file_1)
        .with_context(|| format!("Could not read file {:?}", &args.trace_file_1))?;
    let traces_1 = parse_bb_trace(trace_file_1, &valid_bb)?;
    let trace_file_2 = File::open(&args.trace_file_2)
        .with_context(|| format!("Could not read file {:?}", &args.trace_file_2))?;
    let trace_2 = parse_bb_trace(trace_file_2, &valid_bb)?;

    // TODO: Auto-detect trace format ((mmio?), bb, (ram?))

    // TODO: Write back traces

    Ok(())
}
