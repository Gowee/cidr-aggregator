//! Integration tests exercising the full pipeline from parse → aggregate → normalize → export.

use cidr_aggregator::aggregator::Aggregator;
use cidr_aggregator::{parse_cidrs, IpRange, Ipv4Range};

#[test]
fn full_pipeline_single_v4() {
    let (mut v4, v6, invalid) = parse_cidrs("10.0.0.0/24");
    assert!(v6.is_empty());
    assert!(invalid.is_empty());

    v4.aggregate();
    v4.normalize();
    assert_eq!(v4.export(), "10.0.0.0/24");
}

#[test]
fn full_pipeline_merge_adjacent() {
    let (mut v4, _, _) = parse_cidrs("10.0.0.0/24\n10.0.1.0/24");
    v4.aggregate();
    v4.normalize();
    assert_eq!(v4.export(), "10.0.0.0/23");
}

#[test]
fn full_pipeline_with_comments_and_invalid() {
    let input = "# block list\n10.0.0.0/8\n# internal\n192.168.0.0/16\nbad-line\n172.16.0.0/12";
    let (mut v4, _v6, invalid) = parse_cidrs(input);
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0], "bad-line");

    v4.aggregate();
    v4.normalize();
    let output = v4.export();
    assert!(output.contains("10.0.0.0/8"));
    assert!(output.contains("172.16.0.0/12"));
    assert!(output.contains("192.168.0.0/16"));
}

#[test]
fn full_pipeline_reverse() {
    let (mut v4, _, _) = parse_cidrs("10.0.0.0/8");
    v4.aggregate();
    v4.reverse();
    v4.normalize();
    // The complement of 10.0.0.0/8 should contain 0.0.0.0/5 (or similar),
    // but not contain 10.0.0.0/8
    let output = v4.export();
    assert!(!output.contains("10.0.0.0/8"));
    assert!(v4.count_address() > 0u32);
}

#[test]
fn full_pipeline_exclude_reserved() {
    let (mut v4, _, _) = parse_cidrs("10.0.0.0/8");
    v4.aggregate();
    v4.difference(Ipv4Range::reserved());
    // 10.0.0.0/8 is itself in the reserved list, so it should be removed
    assert!(v4.is_empty());
}

#[test]
fn full_pipeline_ipv6() {
    let (_, mut v6, _) = parse_cidrs("2001:db8::/32\n2001:db8:1::/48");
    v6.aggregate();
    v6.normalize();
    // The /48 is contained in /32
    assert_eq!(v6.export(), "2001:db8::/32");
}

#[test]
fn full_pipeline_mixed() {
    let (mut v4, mut v6, invalid) =
        parse_cidrs("10.0.0.0/24\n10.0.1.0/24\n2001:db8::/32\n2001:db8:1::/48");
    assert!(invalid.is_empty());

    v4.aggregate();
    v4.normalize();
    v6.aggregate();
    v6.normalize();

    assert_eq!(v4.export(), "10.0.0.0/23");
    assert_eq!(v6.export(), "2001:db8::/32");
}

#[test]
fn aggregate_idempotent() {
    let (mut v4, _, _) = parse_cidrs("10.0.2.0/24\n10.0.0.0/24\n10.0.1.0/24");
    v4.aggregate();
    let once = v4.clone();
    v4.aggregate();
    // Aggregate twice should produce the same result as aggregate once
    assert_eq!(once, v4);
}

#[test]
fn normalize_idempotent() {
    let (mut v4, _, _) = parse_cidrs("10.0.0.0/24\n10.0.1.0/24");
    v4.aggregate();
    v4.normalize();
    let once = v4.export();
    v4.normalize();
    let twice = v4.export();
    assert_eq!(once, twice);
}

#[test]
fn empty_input() {
    let (v4, v6, invalid) = parse_cidrs("");
    assert!(v4.is_empty());
    assert!(v6.is_empty());
    assert!(invalid.is_empty());
}

#[test]
fn bare_ip_treated_as_host() {
    let (mut v4, _, _) = parse_cidrs("10.0.0.1");
    assert_eq!(v4.len(), 1);
    v4.normalize();
    assert_eq!(v4.export(), "10.0.0.1/32");
}

#[test]
fn full_range() {
    let (mut v4, _, _) = parse_cidrs("0.0.0.0/0");
    v4.aggregate();
    v4.normalize();
    assert_eq!(v4.export(), "0.0.0.0/0");
    assert_eq!(v4.len(), 1);
}
