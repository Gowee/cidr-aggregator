//! CIDR string parser.

use crate::{EitherIpRange, Ipv4Range, Ipv6Range};

/// Parse a string of CIDR entries (one per line).
///
/// Lines starting with `#` are treated as comments and skipped.
/// Empty lines and whitespace-only lines are also skipped.
///
/// Returns a triple of `(IPv4 ranges, IPv6 ranges, invalid lines)`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        let (v4, v6, invalid) = parse_cidrs("");
        assert!(v4.is_empty());
        assert!(v6.is_empty());
        assert!(invalid.is_empty());
    }

    #[test]
    fn single_v4() {
        let (v4, v6, invalid) = parse_cidrs("10.0.0.0/8");
        assert_eq!(v4.len(), 1);
        assert!(v6.is_empty());
        assert!(invalid.is_empty());
    }

    #[test]
    fn single_v6() {
        let (v4, v6, invalid) = parse_cidrs("2001:db8::/32");
        assert!(v4.is_empty());
        assert_eq!(v6.len(), 1);
        assert!(invalid.is_empty());
    }

    #[test]
    fn mixed_v4_v6() {
        let (v4, v6, invalid) = parse_cidrs("10.0.0.0/8\n2001:db8::/32");
        assert_eq!(v4.len(), 1);
        assert_eq!(v6.len(), 1);
        assert!(invalid.is_empty());
    }

    #[test]
    fn skips_comment_lines() {
        let (v4, v6, invalid) = parse_cidrs("# this is a comment\n10.0.0.0/8\n# another comment");
        assert_eq!(v4.len(), 1);
        assert!(v6.is_empty());
        assert!(invalid.is_empty());
    }

    #[test]
    fn skips_empty_lines() {
        let (v4, _v6, invalid) = parse_cidrs("\n\n10.0.0.0/8\n\n\n");
        assert_eq!(v4.len(), 1);
        assert!(invalid.is_empty());
    }

    #[test]
    fn trims_whitespace() {
        let (v4, _v6, invalid) = parse_cidrs("  10.0.0.0/8  ");
        assert_eq!(v4.len(), 1);
        assert!(invalid.is_empty());
    }

    #[test]
    fn collects_invalid_lines() {
        let (v4, _v6, invalid) = parse_cidrs("not-a-cidr\n10.0.0.0/8\nalso-not-valid");
        assert_eq!(v4.len(), 1);
        assert_eq!(invalid.len(), 2);
        assert_eq!(invalid[0], "not-a-cidr");
        assert_eq!(invalid[1], "also-not-valid");
    }

    #[test]
    fn all_invalid() {
        let (v4, v6, invalid) = parse_cidrs("garbage\n192.168.1.5/24");
        assert!(v4.is_empty());
        assert!(v6.is_empty());
        assert_eq!(invalid.len(), 2);
    }
}
