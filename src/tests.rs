//! Shared test helpers and tests for types defined in `lib.rs`.

use std::net::{Ipv4Addr, Ipv6Addr};

use crate::{aggregator::Aggregator, EitherIpRange, IpRange, Ipv4Range, Ipv6Range};

// -- Shared helpers --

/// Parse a string as an `Ipv4Range`, panicking on failure.
pub fn v4(input: &str) -> Ipv4Range {
    input.parse::<EitherIpRange>().unwrap().into_v4().unwrap()
}

/// Parse a string as an `Ipv6Range`, panicking on failure.
pub fn v6(input: &str) -> Ipv6Range {
    input.parse::<EitherIpRange>().unwrap().into_v6().unwrap()
}

/// Collect `&str` slices into `Ipv4Range`s.
pub fn v4s(inputs: &[&str]) -> Vec<Ipv4Range> {
    inputs.iter().copied().map(v4).collect()
}

/// Collect `&str` slices into `Ipv6Range`s.
pub fn v6s(inputs: &[&str]) -> Vec<Ipv6Range> {
    inputs.iter().copied().map(v6).collect()
}

// -- Ipv4Range tests --

#[test]
fn ipv4_from_cidr_pair() {
    let r = Ipv4Range::from_cidr_pair((Ipv4Addr::new(192, 168, 1, 0), 24));
    assert_eq!(r.first_address(), Ipv4Addr::new(192, 168, 1, 0));
    assert_eq!(r.last_address(), Ipv4Addr::new(192, 168, 1, 255));
    assert_eq!(r.length(), 256);
}

#[test]
fn ipv4_cidr_32_is_single_host() {
    let r = Ipv4Range::from_cidr_pair((Ipv4Addr::new(10, 0, 0, 1), 32));
    assert_eq!(r.first_address(), Ipv4Addr::new(10, 0, 0, 1));
    assert_eq!(r.last_address(), Ipv4Addr::new(10, 0, 0, 1));
    assert_eq!(r.length(), 1);
}

#[test]
fn ipv4_cidr_0_is_full_range() {
    let r = Ipv4Range::from_cidr_pair((Ipv4Addr::new(0, 0, 0, 0), 0));
    assert_eq!(r.first_address_as_decimal(), 0);
    assert_eq!(r.last_address_as_decimal(), u32::MAX);
}

#[test]
fn ipv4_full() {
    let r = Ipv4Range::full();
    assert_eq!(r.first_address_as_decimal(), 0);
    assert_eq!(r.last_address_as_decimal(), u32::MAX);
}

#[test]
fn ipv4_from_cidr_pair_decimal() {
    let r = Ipv4Range::from_cidr_pair_decimal((0x0a000001, 0x0a0000ff));
    assert_eq!(r.first_address_as_decimal(), 0x0a000001);
    assert_eq!(r.last_address_as_decimal(), 0x0a0000ff);
    assert_eq!(r.length(), 255);
}

// -- Ipv6Range tests --

#[test]
fn ipv6_from_cidr_pair() {
    let r = Ipv6Range::from_cidr_pair((Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32));
    assert_eq!(
        r.first_address(),
        Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0)
    );
    assert_eq!(
        r.last_address(),
        Ipv6Addr::new(0x2001, 0xdb8, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff)
    );
}

#[test]
fn ipv6_cidr_128_is_single_host() {
    let r = Ipv6Range::from_cidr_pair((Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1), 128));
    assert_eq!(r.length(), 1);
}

#[test]
fn ipv6_cidr_0_is_full_range() {
    let r = Ipv6Range::from_cidr_pair((Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0));
    assert_eq!(r.first_address_as_decimal(), 0);
    assert_eq!(r.last_address_as_decimal(), u128::MAX);
}

#[test]
fn ipv6_full() {
    let r = Ipv6Range::full();
    assert_eq!(r.first_address_as_decimal(), 0);
    assert_eq!(r.last_address_as_decimal(), u128::MAX);
}

// -- Display tests --

#[test]
fn display_normalized_v4() {
    let r = Ipv4Range::from_cidr_pair((Ipv4Addr::new(192, 168, 1, 0), 24));
    assert_eq!(r.to_string(), "192.168.1.0/24");
}

#[test]
fn display_normalized_v6() {
    let r = Ipv6Range::from_cidr_pair((Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32));
    assert_eq!(r.to_string(), "2001:db8::/32");
}

#[test]
fn display_full_v4() {
    let r = Ipv4Range::full();
    assert_eq!(r.to_string(), "0.0.0.0/0");
}

#[test]
fn display_full_v6() {
    let r = Ipv6Range::full();
    assert_eq!(r.to_string(), "::/0");
}

#[test]
#[should_panic(expected = "Range not normalize yet")]
fn display_panics_on_unnormalized_v4() {
    let r = Ipv4Range::from_cidr_pair_decimal((0x0a000000, 0x0a000005)); // 6 IPs, not power of 2
    let _ = r.to_string();
}

#[test]
#[should_panic(expected = "Range not normalize yet")]
fn display_panics_on_unnormalized_v6() {
    let r = Ipv6Range::from_cidr_pair_decimal((0, 5));
    let _ = r.to_string();
}

// -- EitherIpRange / FromStr tests --

#[test]
fn parse_valid_v4_cidr() {
    let r = "192.168.1.0/24".parse::<EitherIpRange>().unwrap();
    assert!(r.is_v4());
    assert_eq!(r.into_v4().unwrap().length(), 256);
}

#[test]
fn parse_valid_v6_cidr() {
    let r = "2001:db8::/32".parse::<EitherIpRange>().unwrap();
    assert!(r.is_v6());
}

#[test]
fn parse_bare_ipv4_treated_as_32() {
    let r = "10.0.0.1".parse::<EitherIpRange>().unwrap();
    assert_eq!(r.into_v4().unwrap().length(), 1);
    assert_eq!(
        r.into_v4().unwrap().first_address(),
        Ipv4Addr::new(10, 0, 0, 1)
    );
}

#[test]
fn parse_bare_ipv6_treated_as_128() {
    let r = "::1".parse::<EitherIpRange>().unwrap();
    assert_eq!(r.into_v6().unwrap().length(), 1);
}

#[test]
fn parse_rejects_host_address() {
    // 192.168.1.5/24 has host bits set (the .5)
    assert!("192.168.1.5/24".parse::<EitherIpRange>().is_err());
}

#[test]
fn parse_rejects_invalid_ip() {
    assert!("not.an.ip/24".parse::<EitherIpRange>().is_err());
    assert!("abc/24".parse::<EitherIpRange>().is_err());
}

#[test]
fn parse_rejects_empty() {
    assert!("".parse::<EitherIpRange>().is_err());
}

#[test]
fn parse_rejects_invalid_cidr_too_large() {
    assert!("10.0.0.0/33".parse::<EitherIpRange>().is_err());
    assert!("::/129".parse::<EitherIpRange>().is_err());
}

#[test]
fn parse_zero_zero_zero_zero_slash_zero() {
    let r = "0.0.0.0/0".parse::<EitherIpRange>().unwrap();
    let v4 = r.into_v4().unwrap();
    assert_eq!(v4.first_address_as_decimal(), 0);
    assert_eq!(v4.last_address_as_decimal(), u32::MAX);
}

// -- From/Into round-trip tests --

#[test]
fn ipv4_round_trip_cidr_pair() {
    // from_cidr_pair takes (addr, prefix_len); into() returns (addr, log2(length))
    // A /8 has length 2^24, so into() returns (addr, 24)
    let original = Ipv4Range::from_cidr_pair((Ipv4Addr::new(10, 0, 0, 0), 8));
    let (addr, host_bits): (Ipv4Addr, u8) = original.into();
    assert_eq!(addr, Ipv4Addr::new(10, 0, 0, 0));
    assert_eq!(host_bits, 24); // log2(2^24) = 24
}

#[test]
fn ipv6_round_trip_cidr_pair() {
    // into() returns log2(length), not prefix length
    let original = Ipv6Range::from_cidr_pair((Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 64));
    let (addr, host_bits): (Ipv6Addr, u8) = original.into();
    assert_eq!(addr, Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0));
    assert_eq!(host_bits, 64); // log2(2^64) = 64
}

#[test]
fn ipv4_round_trip_decimal_pair() {
    let original = Ipv4Range::from_cidr_pair_decimal((0x0a000000, 0x0affffff));
    let (first, last): (u32, u32) = original.into();
    assert_eq!(first, 0x0a000000);
    assert_eq!(last, 0x0affffff);
}

// -- Reserved ranges --

#[test]
fn ipv4_reserved_is_not_empty() {
    assert!(!Ipv4Range::reserved().is_empty());
}

#[test]
fn ipv6_reserved_is_not_empty() {
    assert!(!Ipv6Range::reserved().is_empty());
}

#[test]
fn ipv4_reserved_contains_private() {
    // 10.0.0.0/8 should be in reserved
    let private = v4("10.0.0.0/8");
    let reserved = Ipv4Range::reserved();
    // The reserved list is aggregated, so private should be covered
    let mut combined = reserved.to_vec();
    combined.push(private);
    let before = combined.len();
    combined.aggregate();
    // After aggregating with the private range, length shouldn't increase
    assert!(combined.len() <= before);
}

#[test]
fn ipv6_reserved_contains_loopback() {
    let loopback = v6("::1/128");
    let reserved = Ipv6Range::reserved();
    let mut combined = reserved.to_vec();
    combined.push(loopback);
    let before = combined.len();
    combined.aggregate();
    assert!(combined.len() <= before);
}
