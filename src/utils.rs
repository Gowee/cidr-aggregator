use num_traits::PrimInt;
use std::fmt::Display;
use std::mem;
use std::net::IpAddr;

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
