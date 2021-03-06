use std::{
    fmt::{self, Display},
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;

#[derive(Debug)]
struct BasicBlockEntry {
    id: usize,
    program_counter: usize,
    hit_counter: usize,
}

impl Display for BasicBlockEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            // Non-stripped version
            write!(
                f,
                "{:04x} {:x} {}",
                self.id, self.program_counter, self.hit_counter
            )
        } else {
            // Stripped version
            write!(f, "{:x} {}", self.program_counter, self.hit_counter)
        }
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// File providing all valid basic blocks which are used to unify trace files
    valid_bb_file: PathBuf,
    /// File(s) containing traces to unify
    trace_files: Vec<PathBuf>,
    /// Output path
    #[clap(short)]
    output_path: Option<PathBuf>,
    /// Flag to enable verbose output
    #[clap(long, short)]
    verbose: bool,
    /// Flag to enable id stripping
    #[clap(long)]
    strip: bool,
}

fn parse_bb_trace_file(
    path: &PathBuf,
    valid_bb: &[usize],
    verbose: bool,
) -> Result<Vec<BasicBlockEntry>> {
    let trace_file = File::open(path).with_context(|| format!("Could not read file {:?}", path))?;

    let mut entries = Vec::new();

    let reader = BufReader::new(trace_file);
    let mut ids = Vec::new();
    let mut program_counters = Vec::new();
    let mut hit_counters = Vec::new();

    let mut id_offset = 0;

    for line in reader.lines().map(|l| l.unwrap()) {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        let id = usize::from_str_radix(parts[0], 16)?;
        let pc = usize::from_str_radix(parts[1], 16)?;
        let hit_count = parts[2].parse::<usize>()?;

        if valid_bb.contains(&pc) {
            ids.push(id - id_offset);
            program_counters.push(pc);
            hit_counters.push(hit_count);
        } else {
            id_offset += 1;
        }
    }

    // Truncate IDs
    ids.truncate(program_counters.len());

    // Ensure integrity
    assert_eq!(ids.len(), program_counters.len());
    assert_eq!(program_counters.len(), hit_counters.len());

    for i in 0..ids.len() {
        entries.push(BasicBlockEntry {
            id: ids[i],
            program_counter: program_counters[i],
            hit_counter: hit_counters[i],
        });
    }

    if verbose {
        println!(
            "{} basic block entries deleted for file: '{}'",
            id_offset,
            path.to_string_lossy()
        );
    }

    Ok(entries)
}

fn write_trace_file<T: Display>(traces: &[T], file_path: PathBuf, strip: bool) -> Result<()> {
    let mut unified_trace_file = File::create(&file_path)
        .with_context(|| format!("Unable to create output file {:?}", &file_path))?;

    for trace in traces.iter() {
        if !strip {
            // Write non-stripped version
            writeln!(unified_trace_file, "{}", trace)?;
        } else {
            // Write stripped version
            writeln!(unified_trace_file, "{:#}", trace)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // Read and parse bb file
    let valid_bb_file = File::open(&args.valid_bb_file)
        .with_context(|| format!("Could not read file {:?}", &args.valid_bb_file))?;
    let valid_bb: Vec<usize> = BufReader::new(valid_bb_file)
        .lines()
        .map(|l| l.unwrap())
        .filter_map(|l| usize::from_str_radix(&l, 16).ok())
        .collect();

    // Create output directory beforehand if needed
    let output_path = if let Some(output_path) = args.output_path {
        fs::create_dir_all(&output_path).with_context(|| "Unable to create output dir")?;
        output_path
    } else {
        PathBuf::new()
    };

    // Collect all trace file paths
    let trace_file_paths: Vec<PathBuf> = args
        .trace_files
        .iter()
        .flat_map(|path| {
            if let Ok(dir_entries) = fs::read_dir(path) {
                dir_entries
                    .into_iter()
                    .filter_map(|d| d.ok())
                    .map(|e| e.path())
                    .collect::<Vec<PathBuf>>()
            } else {
                // Either error happened or the trace file isn't a directory,
                // will handle error case later
                vec![path.clone()]
            }
        })
        .collect();

    // TODO: Auto-detect trace format ((mmio?), bb, (ram?))
    // Handle all trace files
    trace_file_paths.par_iter().for_each(|path| {
        // Only read valid traces from valid BBs (unification)
        let traces = parse_bb_trace_file(path, &valid_bb, args.verbose)
            .with_context(|| format!("Error while parsing trace file {:?}", path))
            .unwrap();

        // Write back unified traces
        let mut unified_trace_file_path = output_path.clone();

        unified_trace_file_path.push(path.file_name().unwrap());

        if !args.strip {
            unified_trace_file_path.set_extension("unified");
        } else {
            unified_trace_file_path.set_extension("stripped");
        }

        write_trace_file(&traces, unified_trace_file_path, args.strip).unwrap();
    });

    Ok(())
}
