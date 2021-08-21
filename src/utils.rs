use std::fmt::Display;
use std::mem;
use std::net::IpAddr;

use lazy_static::lazy_static;
use num_traits::PrimInt;

use crate::{aggregator::Aggregator, EitherIpRange, Ipv4Range, Ipv6Range};

pub trait MathLog2 {
    // We follow the return type convention used in `.leading_zeros`.
    // https://users.rust-lang.org/t/why-the-return-type-of-int-leading-zeros-is-u32-of-u8/
    fn log2(self) -> u32;

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

pub fn ip_addr_to_bit_length(ipa: IpAddr) -> u32 {
    if ipa.is_ipv4() {
        32
    } else if ipa.is_ipv6() {
        128
    } else {
        unimplemented!()
    }
}

pub fn ip_addr_trailing_zeros(ipa: IpAddr) -> u32 {
    match ipa {
        IpAddr::V4(ip) => u32::from(ip).trailing_zeros(),
        IpAddr::V6(ip) => u128::from(ip).trailing_zeros(),
    }
}

lazy_static! {
    pub static ref IPV4_RESERVED: Vec<Ipv4Range> = [
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
        "255.255.255.255/32"
    ]
    .iter()
    .cloned()
    .map(|s| s.parse::<EitherIpRange>().unwrap().into_v4().unwrap())
    .collect::<Vec<Ipv4Range>>()
    .aggregated();
    pub static ref IPV6_RESERVED: Vec<Ipv6Range> = [
        // "::/0",
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
        "ff00::/8"
    ]
    .iter()
    .cloned()
    .map(|s| s.parse::<EitherIpRange>().unwrap().into_v6().unwrap())
    .collect::<Vec<Ipv6Range>>()
    .aggregated();
}
