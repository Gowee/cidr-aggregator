use std::io::{self, Read};

use structopt::StructOpt;

use ::cidr_aggregator::aggregator::Aggregator;
use ::cidr_aggregator::parser::parse_cidrs;
use ::cidr_aggregator::{EitherIpRange, IpRange, Ipv4Range, Ipv6Range};
// use crate::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "cidr-aggregator")]
struct Opt {
    /// Process IPv4 only; by default, both IPv4 and IPv6 are accepted
    #[structopt(short = "4", long)]
    v4only: bool,

    /// Process IPv6 only; by default, both IPv4 and IPv6 are accepted
    #[structopt(short = "6", long)]
    v6only: bool,

    /// Reverse ranges
    #[structopt(short, long)]
    reverse: bool,

    /// Filter out all reserved IPs for special purposes (RFC 5735 and RFC 6890), incl. private IPs
    #[structopt(short = "x", long)]
    exclude_reserved: bool,

    /// Ignore unrecognized lines; by default, it rejects with error
    #[structopt(short = "i", long)]
    ignore_invalid: bool,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    let (v4, v6) = if !(opt.v4only ^ opt.v6only) {
        (true, true)
    } else {
        (opt.v4only, opt.v6only)
    };
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;
    let (mut v4ranges, mut v6ranges, invalid_entries) = parse_cidrs(&input);

    v4ranges.aggregate();
    v6ranges.aggregate();
    // dbg!(&v4ranges.len());
    if opt.reverse {
        v4ranges.reverse();
        v6ranges.reverse();
    }
    // dbg!(&v4ranges);
    // v4ranges.difference(&[
    //     "10.0.0.0/8"
    //         .parse::<EitherIpRange>()
    //         .unwrap()
    //         .into_v4()
    //         .unwrap(),
    //     "192.168.0.0/16"
    //         .parse::<EitherIpRange>()
    //         .unwrap()
    //         .into_v4()
    //         .unwrap(),
    // ]);
    if opt.exclude_reserved {
        v4ranges.difference(Ipv4Range::reserved());
        v6ranges.difference(Ipv6Range::reserved());
    }
    // dbg!(&v4ranges);
    v4ranges.normalize();
    v6ranges.normalize();
    if !opt.ignore_invalid && !invalid_entries.is_empty() {
        eprintln!("The following lines are not valid CIDRs, IPs or \"#\"-prefixed comments:\n");
        for entry in invalid_entries.iter() {
            eprintln!("{}", entry);
        }
        eprintln!();
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Some lines are invalid",
        ));
    } else {
        if v4 && !v4ranges.is_empty() {
            println!("{}", v4ranges.export());
            if v6 && !v6ranges.is_empty() {
                println!();
            }
        }
        if v6 && !v6ranges.is_empty() {
            println!("{}", &v6ranges.export());
        }
    }
    Ok(())
}
