use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::fmt::{self, Write};

use crate::{EitherIpRange, Ipv4Range, Ipv6Range, IpRange};

// TODO: iterator as output

pub fn parse_cidrs(cidrs: &str) -> (Vec<Ipv4Range>, Vec<Ipv6Range>, Vec<String>) {
    let mut v4ranges = Vec::new();
    let mut v6ranges = Vec::new();
    let mut invalid_entries = Vec::new();
    for line in cidrs
        .lines()
        .map(str::trim)
        .filter(|&line| !line.is_empty() && !line.starts_with("#"))
    {
        if let Some(range) = line.parse::<EitherIpRange>().ok() {
            match range {
                EitherIpRange::V4(r) => v4ranges.push(r),
                EitherIpRange::V6(r) => v6ranges.push(r),
            }
        } else {
            invalid_entries.push(line.to_owned());
        }
    }

    (v4ranges, v6ranges, invalid_entries)
}

pub fn export<R: IpRange>(ranges: &Vec<R>) -> String {
    let mut output = String::new();
    for range in ranges {
        writeln!(output, "{}", range).unwrap();
    }
    output
}
