use std::{
    cmp::{max, min},
    mem,
};

use itertools::Itertools;

use crate::{utils::MathLog2, IpRange};

// use cidr::Cidr;
use num_traits::{Bounded, NumCast, One, PrimInt, WrappingAdd, Zero};

pub trait Aggregator<R: IpRange> {
    fn aggregated(self) -> Vec<R>;

    fn reversed(self) -> Vec<R>;

    fn differenced(self, other: &[R]) -> Vec<R>;

    fn normalized(self) -> Vec<R>;

    fn aggregate(&mut self);

    fn reverse(&mut self);

    fn difference(&mut self, other: &[R]);

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

    #[must_use = "for in-place modification, there is `difference`"]
    fn differenced(self, other: &[R]) -> Vec<R> {
        difference(self, other)
    }

    #[must_use = "for in-place modification, there is `normalize`"]
    fn normalized(self) -> Vec<R> {
        normalized(self)
    }

    // fn with_discarded(self, ranges: &[IpRange]) {}

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

#[inline(always)]
fn aggregated<R: IpRange>(mut ranges: Vec<R>) -> Vec<R> {
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
    if aggregated_ranges.last().is_none() // TODO:
        || *aggregated_ranges.last().unwrap() != R::from_cidr_pair_decimal(last_range)
    {
        aggregated_ranges.push(R::from_cidr_pair_decimal(last_range));
    }
    aggregated_ranges
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

#[inline(always)]
fn normalized<R: IpRange>(ranges: Vec<R>) -> Vec<R> {
    let mut normalized_ranges = Vec::new();
    // dbg!(&ranges);
    for range in ranges.into_iter() {
        let mut first = range.first_address_as_decimal();
        let mut length = range.length();
        // while length != R::AddressDecimal::zero() {
        if first == R::AddressDecimal::zero() && length == R::AddressDecimal::zero() {
            normalized_ranges.push(R::full());
            break;
        }
        loop {
            // dbg!(length);
            // let b = <R::AddressDecimal as NumCast>::from(2).unwrap().pow((first ^ last).trailing_zeros());
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

/// Difference a with b. TODO: doc
///
/// Both a and b are expected to be sorted ascendently and aggregated.
#[inline(always)]
fn difference<R: IpRange>(mut a: Vec<R>, b: &[R]) -> Vec<R> {
    if b.is_empty() {
        return a;
    }
    let mut ds = Vec::new();
    if a.is_empty() {
        return ds;
    }
    let mut i = 0;
    let mut j = 0;
    dbg!(b);
    while i < a.len() && j < b.len() {
        dbg!(a[i], b[j]);
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
                // set a[i].start = b[j].start
                ds.push(R::from_cidr_pair_decimal((
                    a[i].first_address_as_decimal(),
                    b[j].first_address_as_decimal() - R::AddressDecimal::one(),
                )));
                a[i] = R::from_cidr_pair_decimal((
                    b[j].last_address_as_decimal() + R::AddressDecimal::one(),
                    a[i].last_address_as_decimal(),
                ));
                j += 1;
            }
        } else
        /* if a[i].first_address_as_decimal() >= b[j].first_address_as_decimal() */
        {
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
        // else {
        dbg!(i, j, ds.len());
        // }
    }
    dbg!(i);
    if i != a.len() {
        // TODO; <; ... ... ..., .........................
        dbg!("bang");
        ds.extend_from_slice(&a[i..]);
    }
    ds
    // if a[0].first_address_as_decimal() < b[0].first_address_as_decimal() {
    //     let d = R::from_cidr_pair_decimal((
    //         a[i].first_address_as_decimal(),
    //         min(
    //             a[0].last_address_as_decimal(),
    //             b[0].first_address_as_decimal() - R::AddressDecimal::one(),
    //         ),
    //     ));
    //     // dbg!(d);
    //     ds.push(d);
    // }

    // while i < a.len() {
    //     while a[i].first_address_as_decimal() > b[j].last_address_as_decimal() {
    //         j += 1;
    //     }
    //     while a[i].first_address_as_decimal() < b[j].first_address_as_decimal() {
    //         let d = R::from_cidr_pair_decimal((
    //             a[i].first_address_as_decimal(),
    //             min(
    //                 a[i].last_address_as_decimal(),
    //                 b[j].first_address_as_decimal() - R::AddressDecimal::one(),
    //             ),
    //         ));
    //         dbg!(d);
    //         ds.push(d);

    //         if a[i].last_address_as_decimal() < b[j].first_address_as_decimal() {
    //             i += 1;
    //         } else {
    //             break;
    //         }
    //     }
    //     if a[i].last_address_as_decimal() < b[j].last_address_as_decimal() {
    //         i += 1;
    //     } else {
    //         if b[j].last_address_as_decimal() == R::AddressDecimal::max_value() {
    //             break;
    //         }
    //         let d = R::from_cidr_pair_decimal((
    //             b[j].last_address_as_decimal() + R::AddressDecimal::one(),
    //             min(
    //                 a[i].last_address_as_decimal(),
    //                 if j + 2 <= b.len() {
    //                     b[j + 1].first_address_as_decimal() - R::AddressDecimal::one()
    //                 } else {
    //                     R::AddressDecimal::max_value()
    //                 },
    //             ),
    //         ));
    //         ds.push(d);
    //         if a[i].last_address_as_decimal() <= d.last_address_as_decimal() {
    //             i += 1;
    //         }
    //         j += 1;
    //     }
    //     dbg!(i, j, &ds);
    // }
    // ds
}
