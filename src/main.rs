use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{bail, Context};
use clap::Parser;

use cidr_aggregator::aggregator;
use cidr_aggregator::aggregator::Aggregator;
use cidr_aggregator::parser::parse_cidrs;

#[derive(Parser, Debug)]
#[command(
    name = "cidr-aggregator",
    version,
    about = "Aggregate, normalize, reverse, and difference CIDR IP ranges."
)]
struct Opt {
    /// Process IPv4 only
    #[arg(short = '4', long)]
    v4only: bool,

    /// Process IPv6 only
    #[arg(short = '6', long)]
    v6only: bool,

    /// Reverse ranges (compute complement)
    #[arg(short, long)]
    reverse: bool,

    /// Filter out reserved IPs for special purposes (RFC 5735, RFC 6890)
    #[arg(short = 'x', long)]
    exclude_reserved: bool,

    /// Ignore unrecognized lines instead of failing
    #[arg(short = 'i', long)]
    ignore_invalid: bool,

    /// Input file (reads from stdin if not provided)
    #[arg(short = 'f', long, value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output file (writes to stdout if not provided)
    #[arg(short = 'o', long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Print address and line counts to stderr
    #[arg(short = 's', long)]
    stats: bool,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let (v4, v6) = if !(opt.v4only ^ opt.v6only) {
        (true, true)
    } else {
        (opt.v4only, opt.v6only)
    };

    // Read input
    let input = match &opt.input {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read input file: {}", path.display()))?,
        None => {
            let mut buf = String::new();
            io::stdin()
                .lock()
                .read_to_string(&mut buf)
                .context("Failed to read from stdin")?;
            buf
        }
    };

    let (v4ranges, v6ranges, invalid_entries) = parse_cidrs(&input);

    // Handle invalid entries
    if !opt.ignore_invalid && !invalid_entries.is_empty() {
        eprintln!("The following lines are not valid CIDRs, IPs, or \"#\"-prefixed comments:\n");
        for entry in &invalid_entries {
            eprintln!("{}", entry);
        }
        eprintln!();
        bail!("Some lines are invalid");
    }

    // Process
    let v4ranges = aggregator::process(v4ranges, opt.reverse, opt.exclude_reserved);
    let v6ranges = aggregator::process(v6ranges, opt.reverse, opt.exclude_reserved);

    // Statistics
    if opt.stats {
        let v4_addr_count = v4ranges.count_address();
        let v6_addr_count = v6ranges.count_address();
        eprintln!(
            "IPv4: {} range(s), {} address(es)",
            v4ranges.len(),
            v4_addr_count
        );
        eprintln!(
            "IPv6: {} range(s), {} address(es)",
            v6ranges.len(),
            v6_addr_count
        );
    }

    // Write output
    let mut output: Box<dyn Write> = match &opt.output {
        Some(path) => Box::new(
            File::create(path)
                .with_context(|| format!("Failed to create output file: {}", path.display()))?,
        ),
        None => Box::new(io::stdout()),
    };

    if v4 && !v4ranges.is_empty() {
        writeln!(output, "{}", v4ranges.export())?;
    }
    if v6 && !v6ranges.is_empty() {
        // Add blank line separator if v4 was also printed
        if v4 && !v4ranges.is_empty() {
            writeln!(output)?;
        }
        writeln!(output, "{}", v6ranges.export())?;
    }

    Ok(())
}
