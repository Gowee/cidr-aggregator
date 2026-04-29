//! Operations on `Vec<R: IpRange>`: aggregate, reverse, difference, normalize.
//!
//! Each operation exists in two forms:
//! - **Consuming** (`aggregated()`, `reversed()`, etc.) — takes `self`, returns a new `Vec`.
//! - **In-place** (`aggregate()`, `reverse()`, etc.) — modifies the vector in-place.

use std::{
    cmp::{max, min},
    mem,
};

use itertools::Itertools;
use num_traits::{Bounded, NumCast, One, PrimInt, WrappingAdd, Zero};

use crate::{utils::MathLog2, IpRange};

/// Operations on a collection of IP ranges.
pub trait Aggregator<R: IpRange> {
    /// Merge overlapping and adjacent ranges. Returns sorted output.
    #[must_use = "for in-place modification, use `aggregate`"]
    fn aggregated(self) -> Vec<R>;

    /// Compute the complement — all IPs not covered by the given ranges.
    #[must_use = "for in-place modification, use `reverse`"]
    fn reversed(self) -> Vec<R>;

    /// Subtract `other` from `self`. Both inputs must be sorted and aggregated.
    #[must_use = "for in-place modification, use `difference`"]
    fn differenced(self, other: &[R]) -> Vec<R>;

    /// Split ranges into canonical CIDR blocks (power-of-two aligned).
    #[must_use = "for in-place modification, use `normalize`"]
    fn normalized(self) -> Vec<R>;

    fn aggregate(&mut self);

    fn reverse(&mut self);

    fn difference(&mut self, other: &[R]);

    fn normalize(&mut self);

    /// Sum of address counts across all ranges.
    fn count_address(&self) -> R::AddressDecimal;

    /// Serialize each range to CIDR notation, joined by newlines.
    fn export(&self) -> String;
}

impl<R: IpRange> Aggregator<R> for Vec<R> {
    fn aggregated(self) -> Vec<R> {
        aggregated(self)
    }

    fn reversed(self) -> Vec<R> {
        reversed(self)
    }

    fn differenced(self, other: &[R]) -> Vec<R> {
        difference(self, other)
    }

    fn normalized(self) -> Vec<R> {
        normalized(self)
    }

    #[inline(always)]
    fn aggregate(&mut self) {
        *self = mem::take(self).aggregated();
    }

    #[inline(always)]
    fn reverse(&mut self) {
        *self = mem::take(self).reversed();
    }

    #[inline(always)]
    fn difference(&mut self, other: &[R]) {
        *self = mem::take(self).differenced(other);
    }

    #[inline(always)]
    fn normalize(&mut self) {
        *self = mem::take(self).normalized();
    }

    fn count_address(&self) -> R::AddressDecimal {
        let mut count = R::AddressDecimal::zero();
        for range in self.iter() {
            count += range.length()
        }
        count
    }

    fn export(&self) -> String {
        self.iter().join("\n")
    }
}

/// Merge overlapping and adjacent ranges into a minimal set.
#[inline(always)]
pub(crate) fn aggregated<R: IpRange>(mut ranges: Vec<R>) -> Vec<R> {
    if ranges.is_empty() {
        return ranges;
    }
    ranges.sort();
    let mut ranges_iter = ranges.into_iter().map(|range| {
        (
            range.first_address_as_decimal(),
            range.last_address_as_decimal(),
        )
    });
    let mut aggregated_ranges = Vec::<R>::new();
    let mut last_range = ranges_iter.next().unwrap();
    for range in ranges_iter {
        if max(range.0, R::AddressDecimal::one()) - R::AddressDecimal::one() <= last_range.1 {
            // let length = (range.0 - last_range.0).wrapping_add(&range.1);
            last_range = (last_range.0, max(range.1, last_range.1))
            // if length == R::AddressDecimal::zero() {
            //     last_range = (last_range.0, R::AddressDecimal::zero());
            // } else {
            //     last_range = (last_range.0, max(length, last_range.1));
            // }
        } else {
            aggregated_ranges.push(R::from_cidr_pair_decimal(last_range));
            last_range = range;
        }
    }
    if aggregated_ranges.last().is_none()
        || *aggregated_ranges.last().unwrap() != R::from_cidr_pair_decimal(last_range)
    {
        aggregated_ranges.push(R::from_cidr_pair_decimal(last_range));
    }
    aggregated_ranges
}

/// Compute the complement: fill all gaps between ranges.
#[inline(always)]
pub(crate) fn reversed<R: IpRange>(ranges: Vec<R>) -> Vec<R> {
    if ranges.is_empty() {
        return vec![R::full()];
    }
    let mut reverse_ranges = Vec::new();
    let mut last_decimal = R::AddressDecimal::zero();
    for range in ranges.into_iter() {
        if range.first_address_as_decimal() > last_decimal {
            reverse_ranges.push(R::from_cidr_pair_decimal((
                last_decimal,
                range.first_address_as_decimal() - R::AddressDecimal::one(),
            )));
        }
        last_decimal = range
            .last_address_as_decimal()
            .wrapping_add(&R::AddressDecimal::one());
    }
    if last_decimal != R::AddressDecimal::zero()
    /* R::AddressDecimal::max_value().wrapping_add(&R::AddressDecimal::one()) */
    {
        reverse_ranges.push(R::from_cidr_pair_decimal((
            last_decimal,
            R::AddressDecimal::max_value(), // (R::AddressDecimal::max_value() - last_decimal).wrapping_add(&R::AddressDecimal::one()),
        )));
    }
    reverse_ranges
}

/// Split each range into canonical CIDR blocks (power-of-two aligned sub-ranges).
#[inline(always)]
pub(crate) fn normalized<R: IpRange>(ranges: Vec<R>) -> Vec<R> {
    let mut normalized_ranges = Vec::new();
    for range in ranges.into_iter() {
        let mut first = range.first_address_as_decimal();
        let mut length = range.length();
        // while length != R::AddressDecimal::zero() {
        if first == R::AddressDecimal::zero() && length == R::AddressDecimal::zero() {
            normalized_ranges.push(R::full());
            break;
        }
        loop {
            let b = <R::AddressDecimal as NumCast>::from(2).unwrap().pow(min(
                length.log2(),
                if first == R::AddressDecimal::zero() {
                    (mem::size_of::<R::AddressDecimal>() * 8) as u32
                } else {
                    first.trailing_zeros()
                },
            ));
            normalized_ranges.push(R::from_cidr_pair_decimal((
                first,
                first + (b - R::AddressDecimal::one()),
            )));
            length -= b;
            if length == R::AddressDecimal::zero() {
                break;
            }
            first += b;
        }
    }
    normalized_ranges
}

/// Subtract `b` from `a`. Both inputs must be sorted and aggregated.
///
/// The implementation is inspired by this [StackOverflow answer](https://stackoverflow.com/a/11891418/5488616).
#[inline(always)]
pub(crate) fn difference<R: IpRange>(mut a: Vec<R>, b: &[R]) -> Vec<R> {
    if b.is_empty() {
        return a;
    }
    let mut ds = Vec::new();
    if a.is_empty() {
        return ds;
    }
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        if a[i].first_address_as_decimal() < b[j].first_address_as_decimal() {
            if a[i].last_address_as_decimal() <= b[j].last_address_as_decimal() {
                let end = if a[i].last_address_as_decimal() < b[j].first_address_as_decimal() {
                    a[i].last_address_as_decimal()
                } else {
                    b[j].first_address_as_decimal() - R::AddressDecimal::one()
                };
                ds.push(R::from_cidr_pair_decimal((
                    a[i].first_address_as_decimal(),
                    end,
                )));
                i += 1;
            } else {
                ds.push(R::from_cidr_pair_decimal((
                    a[i].first_address_as_decimal(),
                    b[j].first_address_as_decimal() - R::AddressDecimal::one(),
                )));
                // set a[i].start = b[j].last
                a[i] = R::from_cidr_pair_decimal((
                    b[j].last_address_as_decimal() + R::AddressDecimal::one(),
                    a[i].last_address_as_decimal(),
                ));
                j += 1;
            }
        } else {
            /* if a[i].first_address_as_decimal() >= b[j].first_address_as_decimal() */
            if a[i].last_address_as_decimal() <= b[j].last_address_as_decimal() {
                i += 1;
            } else {
                if a[i].first_address_as_decimal() <= b[j].last_address_as_decimal() {
                    a[i] = R::from_cidr_pair_decimal((
                        b[j].last_address_as_decimal() + R::AddressDecimal::one(),
                        a[i].last_address_as_decimal(),
                    ));
                }
                j += 1;
            }
        }
    }
    if i != a.len() {
        ds.extend_from_slice(&a[i..]);
    }
    ds
}

/// Run the full pipeline: aggregate, optionally reverse, optionally exclude
/// reserved addresses, then normalize.
///
/// This is the standard workflow used by both the CLI and the WASM web app.
#[doc(hidden)]
pub fn process<R: IpRange>(ranges: Vec<R>, reverse: bool, exclude_reserved: bool) -> Vec<R> {
    let mut ranges = ranges;
    ranges.aggregate();
    if reverse {
        ranges.reverse();
    }
    if exclude_reserved {
        ranges.difference(R::reserved());
    }
    ranges.normalize();
    ranges
}

#[cfg(test)]
mod tests {
    use std::net::Ipv6Addr;

    use crate::{
        aggregator::Aggregator,
        tests::{v4, v4s, v6s},
        IpRange, Ipv4Range, Ipv6Range,
    };

    // ---- aggregated ----

    #[test]
    fn aggregated_empty() {
        let ranges: Vec<crate::Ipv4Range> = vec![];
        assert_eq!(ranges.aggregated(), vec![]);
    }

    #[test]
    fn aggregated_single() {
        let ranges = v4s(&["10.0.0.0/24"]);
        let result = ranges.aggregated();
        assert_eq!(result, v4s(&["10.0.0.0/24"]));
    }

    #[test]
    fn aggregated_already_sorted_non_overlapping() {
        let result = v4s(&["10.0.0.0/24", "10.0.2.0/24"]).aggregated();
        assert_eq!(result, v4s(&["10.0.0.0/24", "10.0.2.0/24"]));
    }

    #[test]
    fn aggregated_sorts_unsorted() {
        let result = v4s(&["10.0.2.0/24", "10.0.0.0/24"]).aggregated();
        assert_eq!(result, v4s(&["10.0.0.0/24", "10.0.2.0/24"]));
    }

    #[test]
    fn aggregated_merges_adjacent() {
        // 10.0.0.0/24 (10.0.0.0 - 10.0.0.255) and 10.0.1.0/24 (10.0.1.0 - 10.0.1.255) merge to 10.0.0.0/23
        let result = v4s(&["10.0.0.0/24", "10.0.1.0/24"]).aggregated();
        assert_eq!(result, v4s(&["10.0.0.0/23"]));
    }

    #[test]
    fn aggregated_merges_overlapping() {
        // 10.0.0.0/24 (10.0.0.0 - 10.0.0.255) and 10.0.0.128/25 (10.0.0.128 - 10.0.0.255) merge
        let result = v4s(&["10.0.0.0/24", "10.0.0.128/25"]).aggregated();
        assert_eq!(result, v4s(&["10.0.0.0/24"]));
    }

    #[test]
    fn aggregated_one_contains_another() {
        let result = v4s(&["10.0.0.0/16", "10.0.1.0/24"]).aggregated();
        assert_eq!(result, v4s(&["10.0.0.0/16"]));
    }

    #[test]
    fn aggregated_chain() {
        // Three adjacent /24s merge to one non-canonical range (768 IPs)
        let result = v4s(&["10.0.0.0/24", "10.0.1.0/24", "10.0.2.0/24"]).aggregated();
        assert_eq!(result.len(), 1);
        // Verify it covers all 3 /24s: 10.0.0.0 - 10.0.2.255 = 768 IPs
        assert_eq!(
            result[0].first_address_as_decimal(),
            v4("10.0.0.0/24").first_address_as_decimal()
        );
        assert_eq!(
            result[0].last_address_as_decimal(),
            v4("10.0.2.0/24").last_address_as_decimal()
        );
        assert_eq!(result[0].length(), 768);
    }

    #[test]
    fn aggregated_ipv6() {
        let result = v6s(&["2001:db8::/32", "2001:db8:1::/48"]).aggregated();
        assert_eq!(result, v6s(&["2001:db8::/32"]));
    }

    #[test]
    fn aggregated_idempotent() {
        let ranges = v4s(&[
            "10.0.0.0/24",
            "10.0.2.0/24",
            "10.0.1.0/24",
            "192.168.0.0/16",
            "10.0.0.128/25",
        ]);
        let result1 = ranges.aggregated();
        let result2 = result1.clone().aggregated();
        assert_eq!(result1, result2);
    }

    // ---- reversed ----

    #[test]
    fn reversed_empty_returns_full() {
        let ranges: Vec<crate::Ipv4Range> = vec![];
        let result = ranges.reversed();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Ipv4Range::full());
    }

    #[test]
    fn reversed_full_returns_empty() {
        let result = v4s(&["0.0.0.0/0"]).reversed();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn reversed_single() {
        // Complement of 10.0.0.0/24 = [0.0.0.0/8... no, this is complex
        // Let's verify by checking reverse(reverse(x)) == aggregate(x) instead
        let original = v4s(&["10.0.0.0/24"]);
        let reversed_once = original.clone().reversed();
        let reversed_twice = reversed_once.reversed();
        assert_eq!(original.aggregated(), reversed_twice.aggregated());
    }

    #[test]
    fn reversed_double_is_identity() {
        // reversed() expects sorted, aggregated input
        let original = v4s(&["10.0.0.0/24", "172.16.0.0/12", "192.168.0.0/16"]).aggregated();
        let reversed_twice = original.clone().reversed().reversed();
        assert_eq!(original, reversed_twice.aggregated());
    }

    #[test]
    fn reversed_ipv6_double_is_identity() {
        let original = v6s(&["2001:db8::/32", "fc00::/7"]);
        let reversed_twice = original.clone().reversed().reversed();
        assert_eq!(original.aggregated(), reversed_twice.aggregated());
    }

    // ---- normalized ----

    #[test]
    fn normalized_already_canonical() {
        let result = v4s(&["10.0.0.0/24"]).normalized();
        assert_eq!(result, v4s(&["10.0.0.0/24"]));
    }

    #[test]
    fn normalized_single_ip() {
        // A /32 is already a single canonical block
        let r = Ipv4Range::from_cidr_pair_decimal((0x0a000001, 0x0a000001));
        let result = vec![r].normalized();
        assert_eq!(result, vec![r]);
    }

    #[test]
    fn normalized_full_range() {
        let r = Ipv4Range::full();
        let result = vec![r].normalized();
        assert_eq!(result, vec![Ipv4Range::full()]);
        assert_eq!(result[0].to_string(), "0.0.0.0/0");
    }

    #[test]
    fn normalized_splits_non_canonical() {
        // Range [10.0.0.0, 10.0.0.7] (8 IPs) normalizes to 10.0.0.0/29
        let r = Ipv4Range::from_cidr_pair_decimal((0x0a000000, 0x0a000007));
        let result = vec![r].normalized();
        assert_eq!(result, v4s(&["10.0.0.0/29"]));
    }

    #[test]
    fn normalized_splits_6_ips() {
        // Range [10.0.0.0, 10.0.0.5] (6 IPs, not a power of 2)
        let r = Ipv4Range::from_cidr_pair_decimal((0x0a000000, 0x0a000005));
        let result = vec![r].normalized();
        // Expected: 10.0.0.0/30 (4 IPs) + 10.0.0.4/31 (2 IPs)
        assert_eq!(result, v4s(&["10.0.0.0/30", "10.0.0.4/31"]));
    }

    #[test]
    fn normalized_ipv6() {
        let r = Ipv6Range::from_cidr_pair((Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32));
        let result = vec![r].normalized();
        assert_eq!(result, vec![r]);
    }

    #[test]
    fn normalized_idempotent() {
        let r = Ipv4Range::from_cidr_pair_decimal((0x0a000000, 0x0a000005));
        let result1 = vec![r].normalized();
        let result2 = result1.clone().normalized();
        assert_eq!(result1, result2);
    }

    // ---- difference ----

    #[test]
    fn difference_empty_a() {
        let a: Vec<crate::Ipv4Range> = vec![];
        let b = v4s(&["10.0.0.0/24"]);
        assert_eq!(a.differenced(&b), vec![]);
    }

    #[test]
    fn difference_empty_b() {
        let a = v4s(&["10.0.0.0/24"]);
        let b: Vec<crate::Ipv4Range> = vec![];
        assert_eq!(a.differenced(&b), v4s(&["10.0.0.0/24"]));
    }

    #[test]
    fn difference_no_overlap() {
        let a = v4s(&["10.0.0.0/24"]);
        let b = v4s(&["192.168.0.0/24"]);
        assert_eq!(a.differenced(&b), v4s(&["10.0.0.0/24"]));
    }

    #[test]
    fn difference_b_contains_a() {
        let a = v4s(&["10.0.0.0/24"]);
        let b = v4s(&["10.0.0.0/16"]);
        assert_eq!(a.differenced(&b), vec![]);
    }

    #[test]
    fn difference_a_contains_b() {
        // a = 10.0.0.0/23 (10.0.0.0 - 10.0.1.255)
        // b = 10.0.0.0/24 (10.0.0.0 - 10.0.0.255)
        // result = 10.0.1.0/24 (10.0.1.0 - 10.0.1.255)
        let a = v4s(&["10.0.0.0/23"]);
        let b = v4s(&["10.0.0.0/24"]);
        let result = a.differenced(&b);
        assert_eq!(result, v4s(&["10.0.1.0/24"]));
    }

    #[test]
    fn difference_partial_overlap() {
        // a = 10.0.0.0/24 (0 - 255)
        // b = 10.0.0.128/25 (128 - 255)
        // result = 10.0.0.0/25 (0 - 127)
        let a = v4s(&["10.0.0.0/24"]);
        let b = v4s(&["10.0.0.128/25"]);
        let result = a.differenced(&b);
        assert_eq!(result, v4s(&["10.0.0.0/25"]));
    }

    #[test]
    fn difference_a_spans_b() {
        // a = 10.0.0.0/23 (0.0 - 1.255)
        // b = 10.0.0.128/25 (0.128 - 0.255)
        // result = 10.0.0.0/25 + 10.0.1.0/24
        let a = v4s(&["10.0.0.0/23"]);
        let b = v4s(&["10.0.0.128/25"]);
        let result = a.differenced(&b);
        assert_eq!(result, v4s(&["10.0.0.0/25", "10.0.1.0/24"]));
    }

    #[test]
    fn difference_ipv6() {
        let a = v6s(&["2001:db8::/32"]);
        let b = v6s(&["2001:db8:1::/48"]);
        let result = a.differenced(&b);
        // 2001:db8::/32 minus 2001:db8:1::/48 = two ranges
        assert_eq!(result.len(), 2);
        // Verify no overlap with b
        let overlap = result.clone().differenced(&b);
        assert_eq!(overlap, result);
    }

    // ---- count_address ----

    #[test]
    fn count_address_single() {
        let ranges = v4s(&["10.0.0.0/24"]);
        assert_eq!(ranges.count_address(), 256u32);
    }

    #[test]
    fn count_address_multiple() {
        let ranges = v4s(&["10.0.0.0/24", "192.168.0.0/24"]);
        assert_eq!(ranges.count_address(), 512u32);
    }

    #[test]
    fn count_address_empty() {
        let ranges: Vec<crate::Ipv4Range> = vec![];
        assert_eq!(ranges.count_address(), 0u32);
    }

    // ---- export ----

    #[test]
    fn export_single() {
        let ranges = v4s(&["10.0.0.0/24"]);
        assert_eq!(ranges.export(), "10.0.0.0/24");
    }

    #[test]
    fn export_multiple() {
        let ranges = v4s(&["10.0.0.0/24", "192.168.0.0/16"]);
        assert_eq!(ranges.export(), "10.0.0.0/24\n192.168.0.0/16");
    }

    // ---- in-place operations ----

    #[test]
    fn aggregate_in_place() {
        let mut ranges = v4s(&["10.0.1.0/24", "10.0.0.0/24"]);
        ranges.aggregate();
        assert_eq!(ranges, v4s(&["10.0.0.0/23"]));
    }

    #[test]
    fn normalize_in_place() {
        let r = Ipv4Range::from_cidr_pair_decimal((0x0a000000, 0x0a000007));
        let mut ranges = vec![r];
        ranges.normalize();
        assert_eq!(ranges, v4s(&["10.0.0.0/29"]));
    }

    #[test]
    fn difference_in_place() {
        let mut ranges = v4s(&["10.0.0.0/23"]);
        let b = v4s(&["10.0.0.0/24"]);
        ranges.difference(&b);
        assert_eq!(ranges, v4s(&["10.0.1.0/24"]));
    }
}
