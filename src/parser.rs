use crate::{EitherIpRange, Ipv4Range, Ipv6Range};

// TODO: iterator as output

pub fn parse_cidrs(cidrs: &str) -> (Vec<Ipv4Range>, Vec<Ipv6Range>, Vec<String>) {
    let mut v4ranges = Vec::new();
    let mut v6ranges = Vec::new();
    let mut invalid_entries = Vec::new();
    for line in cidrs
        .lines()
        .map(str::trim)
        .filter(|&line| !line.is_empty() && !line.starts_with('#'))
    {
        if let Ok(range) = line.parse::<EitherIpRange>() {
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
