use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;

use structopt::StructOpt;

use ::cidr_aggregator::aggregator::Aggregator;
use ::cidr_aggregator::parser::parse_cidrs;
// use crate::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "cidr-aggregator")]
struct Opt {
    /// reversed the ranges
    #[structopt(short, long)]
    reversed: bool,

    /// When activating, the program will exit with failure if there is invalid lines in input
    #[structopt(short, long)]
    strict: bool,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    // let ParseResult {v4ranges, v6ranges, invalid_entries} = parse_cidrs(io::stdin().lock().lines());
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;
    let (mut v4ranges, mut v6ranges, invalid_entries) = parse_cidrs(&input);
    v4ranges = v4ranges.aggregated();
    v6ranges = v6ranges.aggregated();
    // dbg!(&v4ranges);
    if opt.reversed {
        v4ranges = v4ranges.reversed();
        v6ranges = v6ranges.reversed();
        // dbg!(&v4ranges);
    }
    v4ranges = v4ranges.normalized();
    // dbg!(&v4ranges);
    v6ranges = v6ranges.normalized();
    println!("{}", v4ranges.export());
    if !v4ranges.is_empty() && !v6ranges.is_empty() {
        println!();
    }
    println!("{}", &v6ranges.export());
    // writeln!(&mut std::io::stderr(), "{}", v4ranges)?;
    // writeln!(&mut std::io::stderr(), "{}", v6ranges)?;

    if opt.strict && !invalid_entries.is_empty() {
        eprintln!("The following lines contains invalid CIDR entries:");
        for entry in invalid_entries.iter() {
            eprintln!("{}", entry);
        }
        eprintln!();
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "One or more lines are invalid",
        ));
    }
    Ok(())
}
