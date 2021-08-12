use std::{array, cmp::min, mem};

use super::IpRange;
use crate::utils::MathLog2;

// use cidr::Cidr;
use num_traits::{pow, Bounded, FromPrimitive, NumAssignOps, NumCast, NumOps, PrimInt, Zero, One, WrappingAdd};

pub trait Aggregator<R: IpRange> {
    fn aggregate(self) -> Vec<R>;

    fn reverse(self) -> Vec<R>;

    fn normalize(self) -> Vec<R>;

    fn aggregated(&mut self);

    fn reversed(&mut self);

    fn normalized(&mut self);

    fn count_address(&self) -> R::AddressDecimal;
}

impl<R: IpRange> Aggregator<R> for Vec<R> {
    #[must_use = "for in-place modification, there is `aggregated`"]
    fn aggregate(self) -> Vec<R> {
        aggregate(self)
    }

    #[must_use = "for in-place modification, there is `reversed`"]
    fn reverse(self) -> Vec<R> {
        reverse(self)
    }

    #[must_use = "for in-place modification, there is `normalzied`"]
    fn normalize(self) -> Vec<R> {
        normalize(self)
    }

    fn aggregated(&mut self) {
        *self = mem::take(self).aggregate();
    }

    fn reversed(&mut self) {
        *self = mem::take(self).reverse();
    }

    fn normalized(&mut self) {
        *self = mem::take(self).normalize();
    }

    fn count_address(&self) -> R::AddressDecimal {
        let mut count = R::AddressDecimal::zero();
        for range in self.iter() {
            count += range.length()
        }
        count
    }
}

fn aggregate<R: IpRange>(mut ranges: Vec<R>) -> Vec<R> {
    if ranges.is_empty() {
        return ranges;
    }
    ranges.sort();
    let mut ranges_iter = ranges.into_iter().map(|range| {
        (
            range.first_address_as_decimal(),
            range.length(), // pow(<R::AddressDecimal as NumCast>::from(2).unwrap(),
                            // range.network_length() as usize)
        )
    });
    let mut aggregated_ranges = Vec::<R>::new();
    let mut last_range = ranges_iter.next().unwrap();
    for range in ranges_iter {
        if range.0 - last_range.0 == last_range.1 {
            last_range = (last_range.0, last_range.1.wrapping_add(&range.1));
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

fn reverse<R: IpRange>(ranges: Vec<R>) -> Vec<R> {
    if ranges.is_empty() {
        return vec![R::full()];
    }
    let mut reversed_ranges = Vec::new();
    let mut last_decimal = R::AddressDecimal::zero();
    for range in ranges.into_iter() {
        if range.first_address_as_decimal() > last_decimal {
            reversed_ranges.push(R::from_cidr_pair_decimal((
                last_decimal,
                range.first_address_as_decimal() - last_decimal,
            )));
        }
        last_decimal = range.first_address_as_decimal() + range.length();
    }
    if last_decimal != R::AddressDecimal::zero() /* R::AddressDecimal::max_value().wrapping_add(&R::AddressDecimal::one()) */ {
        reversed_ranges.push(R::from_cidr_pair_decimal((last_decimal, (R::AddressDecimal::max_value() - last_decimal).wrapping_add(&R::AddressDecimal::one()))));
    }
    reversed_ranges
}

fn normalize<R: IpRange>(ranges: Vec<R>) -> Vec<R> {
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
            let b = <R::AddressDecimal as NumCast>::from(2)
                .unwrap()
                .pow(min(length.log2(), if first == R::AddressDecimal::zero() {32} else {first.trailing_zeros()}));
            // dbg!(first, length, b, R::from_cidr_pair_decimal((first, b)));
            normalized_ranges.push(R::from_cidr_pair_decimal((first, b)));
            length -= b;
            if length == R::AddressDecimal::zero() {
                break;
            }
            first += b;
        }
    }
    normalized_ranges
}

// fn to_cidrs<R: IpRange>(ranges: Vec<R>) -> Vec<R::Cidr> {
//     for (mut ip, mut count) in ranges.into_iter() {
//         while count != 0 {
//             let b = min(
//                 <R::AddressDecimal as NumCast>::from(2).unwrap().pow(count.log2()),
//                 <R::AddressDecimal as NumCast>::from(2).unwrap().pow(ip.trailing_zeros()),
//             );
//             write!(
//                 output,
//                 "{}/{}\n",
//                 $addr_type::from(ip),
//                 std::mem::size_of::<decimal_type!($addr_type)>() as u32 * 8 - b.log2()
//             )
//             .unwrap();
//             line += 1;
//             assert!(count > 0);
//             count -= b;
//             ip += b;
//         }
//     }
//     unimplemented!())
// }

// fn reverse

// impl IPv4Ranges {
//     fn aggregate(&mut self) {
//         let mut ranges = self.0;
//         ranges.sort();
//         let mut aggregated_ranges = Vec::new();
//         let mut last_range = *ranges.first().unwrap();
//         for range in ranges.into_iter().skip(1) {
//             if range.0 - last_range.0 == last_range.1 {
//                 last_range = (last_range.0, last_range.1 + range.1)
//             } else {
//                 aggregated_ranges.push(last_range);
//                 last_range = range;
//             }
//         }
//         if aggregated_ranges.last().is_none() || *aggregated_ranges.last().unwrap() != last_range {
//             aggregated_ranges.push(last_range);
//         }
//         self.0 = aggregated_ranges;
//         // let mut inverse_entries = Vec::new();
//         // let mut last_decimal = 0;
//         // for range in aggregated_ranges.into_iter().chain(array::IntoIter::new([(<decimal_type!($addr_type)>::MAX, 0)]).into_iter()) {
//         //     if range.0 > last_decimal {
//         //         inverse_entries.push((range.0, range.0 - last_decimal));
//         //     }
//         //     last_decimal = range.0 + range.1;
//         // }
//     }

//     fn reverse(&self) {

//     }

//     fn into_aggregated(self) -> Self {
//         self.aggregate();
//         self
//     }

//     fn into_reversed(self) -> Self {
//         self.reverse()
//         self
//     }
// }
