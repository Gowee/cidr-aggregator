//! Math helpers and reserved IP address lists.

use std::fmt::Display;
use std::mem;
use std::net::IpAddr;
use std::sync::LazyLock;

use num_traits::PrimInt;

use crate::{aggregator::Aggregator, EitherIpRange, Ipv4Range, Ipv6Range};

/// Integer logarithm base 2, complementing the standard library's `leading_zeros`.
pub trait MathLog2 {
    /// Returns `floor(log2(self))`.
    fn log2(self) -> u32;

    /// Returns `Some(log2(self))` if `self` is a power of two, `None` otherwise.
    fn checked_log2(self) -> Option<u32>;
}

impl<T: PrimInt> MathLog2 for T {
    fn log2(self) -> u32 {
        std::mem::size_of::<Self>() as u32 * 8 - self.leading_zeros() - 1
    }

    fn checked_log2(self) -> Option<u32> {
        if self.count_ones() == 1 {
            Some(self.log2())
        } else {
            None
        }
    }
}

/// Format a number as a string, with special handling for the overflow sentinel.
///
/// When `zero_as_overflow` is `true` and the value is `0`, returns the string
/// representation of `max_value + 1`. This is needed because the full IP space
/// has `2^32` (or `2^128`) addresses — one more than fits in the address type.
#[allow(dead_code)]
pub fn to_string_overflow<T: PrimInt + Display>(num: T, zero_as_overflow: bool) -> String {
    if zero_as_overflow && num == T::zero() {
        if mem::size_of::<T>() * 8 == 32 {
            String::from("4294967296")
        } else if mem::size_of::<T>() * 8 == 128 {
            String::from("340282366920938463463374607431768211456")
        } else {
            unimplemented!()
        }
    } else {
        num.to_string()
    }
}

/// Returns the bit-length of an IP address: 32 for IPv4, 128 for IPv6.
pub fn ip_addr_to_bit_length(ipa: IpAddr) -> u32 {
    if ipa.is_ipv4() {
        32
    } else if ipa.is_ipv6() {
        128
    } else {
        unimplemented!()
    }
}

/// Returns the number of trailing zero bits in an IP address.
pub fn ip_addr_trailing_zeros(ipa: IpAddr) -> u32 {
    match ipa {
        IpAddr::V4(ip) => u32::from(ip).trailing_zeros(),
        IpAddr::V6(ip) => u128::from(ip).trailing_zeros(),
    }
}

/// Reserved IPv4 address blocks (RFC 5735, RFC 6890), pre-aggregated.
pub static IPV4_RESERVED: LazyLock<Vec<Ipv4Range>> = LazyLock::new(|| {
    [
        "0.0.0.0/8",
        "10.0.0.0/8",
        "100.64.0.0/10",
        "127.0.0.0/8",
        "169.254.0.0/16",
        "172.16.0.0/12",
        "192.0.0.0/24",
        "192.0.2.0/24",
        "192.88.99.0/24",
        "192.168.0.0/16",
        "198.18.0.0/15",
        "198.51.100.0/24",
        "203.0.113.0/24",
        "224.0.0.0/4",
        "233.252.0.0/24",
        "240.0.0.0/4",
        "255.255.255.255/32",
    ]
    .iter()
    .cloned()
    .map(|s| s.parse::<EitherIpRange>().unwrap().into_v4().unwrap())
    .collect::<Vec<Ipv4Range>>()
    .aggregated()
});

/// Reserved IPv6 address blocks (RFC 6890), pre-aggregated.
pub static IPV6_RESERVED: LazyLock<Vec<Ipv6Range>> = LazyLock::new(|| {
    [
        "::/128",
        "::1/128",
        "::ffff:0:0/96",
        "::ffff:0:0:0/96",
        "64:ff9b::/96",
        "100::/64",
        "2001:0000::/32",
        "2001:20::/28",
        "2001:db8::/32",
        "2002::/16",
        "fc00::/7",
        "fe80::/10",
        "ff00::/8",
    ]
    .iter()
    .cloned()
    .map(|s| s.parse::<EitherIpRange>().unwrap().into_v6().unwrap())
    .collect::<Vec<Ipv6Range>>()
    .aggregated()
});

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn log2_powers_of_two() {
        assert_eq!(1u32.log2(), 0);
        assert_eq!(2u32.log2(), 1);
        assert_eq!(4u32.log2(), 2);
        assert_eq!(8u32.log2(), 3);
        assert_eq!(256u32.log2(), 8);
        assert_eq!(65536u64.log2(), 16);
    }

    #[test]
    fn log2_non_power_of_two() {
        assert_eq!(3u32.log2(), 1); // floor(log2(3)) = 1
        assert_eq!(5u32.log2(), 2);
        assert_eq!(7u32.log2(), 2);
    }

    #[test]
    fn checked_log2_power_of_two() {
        assert_eq!(1u32.checked_log2(), Some(0));
        assert_eq!(2u32.checked_log2(), Some(1));
        assert_eq!(256u32.checked_log2(), Some(8));
    }

    #[test]
    fn checked_log2_non_power_of_two() {
        assert_eq!(3u32.checked_log2(), None);
        assert_eq!(6u32.checked_log2(), None);
        assert_eq!(0u32.checked_log2(), None);
    }

    #[test]
    fn ip_addr_to_bit_length_v4() {
        assert_eq!(
            ip_addr_to_bit_length(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            32
        );
    }

    #[test]
    fn ip_addr_to_bit_length_v6() {
        assert_eq!(ip_addr_to_bit_length(IpAddr::V6(Ipv6Addr::LOCALHOST)), 128);
    }

    #[test]
    fn trailing_zeros_v4() {
        // 192.168.1.0 = 0xc0a80100 = ...0001_0000_0000, trailing zeros = 8
        assert_eq!(
            ip_addr_trailing_zeros(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0))),
            8
        );
        // 10.0.0.0 = 0x0a000000 = 00001010...0, trailing zeros = 25
        assert_eq!(
            ip_addr_trailing_zeros(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0))),
            25
        );
    }

    #[test]
    fn trailing_zeros_v6() {
        // ::1 = 1, trailing zeros = 0
        assert_eq!(
            ip_addr_trailing_zeros(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))),
            0
        );
        // :: = 0, trailing zeros = 128
        assert_eq!(
            ip_addr_trailing_zeros(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0))),
            128
        );
    }

    #[test]
    fn to_string_overflow_normal() {
        assert_eq!(to_string_overflow(256u32, false), "256");
        assert_eq!(to_string_overflow(0u32, false), "0");
    }

    #[test]
    fn to_string_overflow_v4_full() {
        assert_eq!(to_string_overflow(0u32, true), "4294967296");
    }

    #[test]
    fn to_string_overflow_v6_full() {
        assert_eq!(
            to_string_overflow(0u128, true),
            "340282366920938463463374607431768211456"
        );
    }
}
