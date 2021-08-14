use std::{
    array,
    cmp::{max, min},
    fmt::Write,
    mem,
};

use super::IpRange;
use crate::utils::MathLog2;
use itertools::Itertools;

// use cidr::Cidr;
use num_traits::{
    pow, Bounded, FromPrimitive, NumAssignOps, NumCast, NumOps, One, PrimInt, WrappingAdd, Zero,
};

pub trait Aggregator<R: IpRange> {
    fn aggregated(self) -> Vec<R>;

    fn reversed(self) -> Vec<R>;

    fn normalized(self) -> Vec<R>;

    fn aggregate(&mut self);

    fn reverse(&mut self);

    fn normalize(&mut self);

    fn count_address(&self) -> R::AddressDecimal;

    fn export(&self) -> String;
}

impl<R: IpRange> Aggregator<R> for Vec<R> {
    #[must_use = "for in-place modification, there is `aggregate`"]
    fn aggregated(self) -> Vec<R> {
        aggregated(self)
    }

    #[must_use = "for in-place modification, there is `reverse`"]
    fn reversed(self) -> Vec<R> {
        reversed(self)
    }

    #[must_use = "for in-place modification, there is `normalzied`"]
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

#[inline(always)]
fn aggregated<R: IpRange>(mut ranges: Vec<R>) -> Vec<R> {
    if ranges.is_empty() {
        return ranges;
    }
    ranges.sort();
    let mut ranges_iter = ranges
        .into_iter()
        .map(|range| (range.first_address_as_decimal(), range.length()));
    let mut aggregate_ranges = Vec::<R>::new();
    let mut last_range = ranges_iter.next().unwrap();
    for range in ranges_iter {
        if range.0 - last_range.0 <= last_range.1 {
            let length = (range.0 - last_range.0).wrapping_add(&range.1);
            if length == R::AddressDecimal::zero() {
                last_range = (last_range.0, R::AddressDecimal::zero());
            } else {
                last_range = (last_range.0, max(length, last_range.1));
            }
        } else {
            aggregate_ranges.push(R::from_cidr_pair_decimal(last_range));
            last_range = range;
        }
    }
    if aggregate_ranges.last().is_none()
        || *aggregate_ranges.last().unwrap() != R::from_cidr_pair_decimal(last_range)
    {
        aggregate_ranges.push(R::from_cidr_pair_decimal(last_range));
    }
    aggregate_ranges
}

#[inline(always)]
fn reversed<R: IpRange>(ranges: Vec<R>) -> Vec<R> {
    if ranges.is_empty() {
        return vec![R::full()];
    }
    let mut reverse_ranges = Vec::new();
    let mut last_decimal = R::AddressDecimal::zero();
    for range in ranges.into_iter() {
        if range.first_address_as_decimal() > last_decimal {
            reverse_ranges.push(R::from_cidr_pair_decimal((
                last_decimal,
                range.first_address_as_decimal() - last_decimal,
            )));
        }
        last_decimal = range
            .first_address_as_decimal()
            .wrapping_add(&range.length());
    }
    if last_decimal != R::AddressDecimal::zero()
    /* R::AddressDecimal::max_value().wrapping_add(&R::AddressDecimal::one()) */
    {
        reverse_ranges.push(R::from_cidr_pair_decimal((
            last_decimal,
            (R::AddressDecimal::max_value() - last_decimal).wrapping_add(&R::AddressDecimal::one()),
        )));
    }
    reverse_ranges
}

#[inline(always)]
fn normalized<R: IpRange>(ranges: Vec<R>) -> Vec<R> {
    let mut normalize_ranges = Vec::new();
    for range in ranges.into_iter() {
        let mut first = range.first_address_as_decimal();
        let mut length = range.length();
        // while length != R::AddressDecimal::zero() {
        if first == R::AddressDecimal::zero() && length == R::AddressDecimal::zero() {
            normalize_ranges.push(R::full());
            break;
        }
        loop {
            let b = <R::AddressDecimal as NumCast>::from(2).unwrap().pow(min(
                length.log2(),
                if first == R::AddressDecimal::zero() {
                    32
                } else {
                    first.trailing_zeros()
                },
            ));
            normalize_ranges.push(R::from_cidr_pair_decimal((first, b)));
            length -= b;
            if length == R::AddressDecimal::zero() {
                break;
            }
            first += b;
        }
    }
    normalize_ranges
}
