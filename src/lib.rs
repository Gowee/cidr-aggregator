use std::fmt::{self, Debug, Display};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use crate::utils::{
    ip_addr_to_bit_length, ip_addr_trailing_zeros, MathLog2, IPV4_RESERVED, IPV6_RESERVED,
};
use num_traits::{Bounded, NumAssignOps, NumCast, PrimInt, WrappingAdd, Zero};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv4Range(u32, u32); // [start, end] inclusively

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv6Range(u128, u128);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EitherIpRange {
    V4(Ipv4Range),
    V6(Ipv6Range),
}

impl EitherIpRange {
    pub fn into_v4(self) -> Option<Ipv4Range> {
        match self {
            EitherIpRange::V4(r) => Some(r),
            _ => None,
        }
    }

    pub fn into_v6(self) -> Option<Ipv6Range> {
        match self {
            EitherIpRange::V6(r) => Some(r),
            _ => None,
        }
    }

    pub fn is_v4(self) -> bool {
        self.into_v4().is_some()
    }

    pub fn is_v6(self) -> bool {
        self.into_v6().is_some()
    }
}

impl FromStr for EitherIpRange {
    type Err = ();

    fn from_str(s: &str) -> Result<EitherIpRange, Self::Err> {
        if let Some((ip, cidr)) = s.split_once("/").or(Some((s, "")))
        // .and_then(|(ip, cidr)| Some((ip.parse::<IpAddr>().ok()?, cidr.parse::<u8>().ok()?)))
        {
            let ip = ip.parse::<IpAddr>().map_err(|_| ())?;
            let cidr = if cidr.is_empty() {
                ip_addr_to_bit_length(ip) as u8
            } else {
                cidr.parse::<u8>().map_err(|_| ())?
            };
            if cidr as u32 > ip_addr_to_bit_length(ip)
                || ip_addr_trailing_zeros(ip) < ip_addr_to_bit_length(ip) - cidr as u32
            {
                return Err(()); // a host instead of a range
            }
            Ok(match ip {
                IpAddr::V4(ip) => EitherIpRange::V4(Ipv4Range::from_cidr_pair((ip, cidr))),
                IpAddr::V6(ip) => EitherIpRange::V6(Ipv6Range::from_cidr_pair((ip, cidr))),
            })
        } else {
            Err(())
        }
    }
}

pub trait IpRange: Copy + Clone + Eq + Ord + Display + Debug + 'static {
    type Address; // TODO: how to associate Address with AddressDecimal somehow?
    type AddressDecimal: PrimInt + NumAssignOps + WrappingAdd + Bounded + Display + Debug;

    fn first_address(&self) -> Self::Address;
    fn first_address_as_decimal(&self) -> Self::AddressDecimal;
    fn last_address(&self) -> Self::Address;
    fn last_address_as_decimal(&self) -> Self::AddressDecimal;
    fn length(&self) -> Self::AddressDecimal;
    fn from_cidr_pair(first_address_and_cidr: (Self::Address, u8)) -> Self;
    fn into_cidr_pair(self) -> (Self::Address, u8);
    fn from_cidr_pair_decimal(
        first_and_last_address_decimal: (Self::AddressDecimal, Self::AddressDecimal),
    ) -> Self;
    fn into_cidr_pair_decimal(self) -> (Self::AddressDecimal, Self::AddressDecimal);
    fn full() -> Self {
        // inclusive interval
        Self::from_cidr_pair_decimal((
            Self::AddressDecimal::zero(),
            Self::AddressDecimal::max_value(),
        ))
    }
    fn reserved() -> &'static [Self];
    // fn format_cidr(&self) -> fmt::Arguments;
}

macro_rules! impl_ip_range {
    ($ip_range: ident, $address_type: ident, $decimal_type: ident, $reserved: ident) => {
        impl IpRange for $ip_range {
            type Address = $address_type;
            type AddressDecimal = $decimal_type;

            fn first_address(&self) -> Self::Address {
                self.0.into()
            }

            fn first_address_as_decimal(&self) -> Self::AddressDecimal {
                self.0
            }

            fn last_address(&self) -> Self::Address {
                self.1.into()
            }

            fn last_address_as_decimal(&self) -> Self::AddressDecimal {
                self.1
            }

            fn length(&self) -> Self::AddressDecimal {
                (self.1 - self.0).wrapping_add(1 as $decimal_type)
            }

            fn from_cidr_pair(first_address_and_cidr: (Self::Address, u8)) -> Self {
                let first: $decimal_type = first_address_and_cidr.0.into();
                let last = if first_address_and_cidr.1 == 0 {
                    Self::AddressDecimal::max_value()
                } else {
                    first
                        + (<Self::AddressDecimal as NumCast>::from(2).unwrap().pow(
                            std::mem::size_of::<$address_type>() as u32 * 8
                                - first_address_and_cidr.1 as u32,
                        ) - 1)
                };
                Self(first, last)
            }

            fn into_cidr_pair(self) -> (Self::Address, u8) {
                self.into()
            }

            fn from_cidr_pair_decimal(
                first_and_last_address_decimal: (Self::AddressDecimal, Self::AddressDecimal),
            ) -> Self {
                assert!(first_and_last_address_decimal.0 <= first_and_last_address_decimal.1);
                Self(
                    first_and_last_address_decimal.0,
                    first_and_last_address_decimal.1,
                )
            }

            fn into_cidr_pair_decimal(self) -> (Self::AddressDecimal, Self::AddressDecimal) {
                self.into()
            }

            fn reserved() -> &'static [Self] {
                &$reserved[..]
            }

            // fn format_cidr(&self) -> fmt::Arguments {
            //     let cidr: u32 = self.1.checked_log2().expect("Range not normalize yet");
            //     let a = 1;
            //     format_args!("{}/{}", a, a)
            // }
        }

        impl From<($address_type, u8)> for $ip_range {
            fn from(first_address_and_cidr: ($address_type, u8)) -> $ip_range {
                Self::from_cidr_pair(first_address_and_cidr)
            }
        }

        impl From<$ip_range> for ($address_type, u8) {
            fn from(range: $ip_range) -> ($address_type, u8) {
                (
                    $address_type::from(range.0),
                    (range.1 - range.0 + 1)
                        .checked_log2()
                        .expect("Range not normalize yet") as u8,
                )
            }
        }

        impl From<($decimal_type, $decimal_type)> for $ip_range {
            fn from(first_and_last_address_decimal: ($decimal_type, $decimal_type)) -> $ip_range {
                Self::from_cidr_pair_decimal(first_and_last_address_decimal)
            }
        }

        impl From<$ip_range> for ($decimal_type, $decimal_type) {
            fn from(range: $ip_range) -> ($decimal_type, $decimal_type) {
                (range.0.into(), range.1)
            }
        }

        impl fmt::Display for $ip_range {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let cidr = if self.first_address_as_decimal() == $decimal_type::zero()
                    && self.length() == 0
                {
                    0
                } else {
                    std::mem::size_of::<$decimal_type>() as u32 * 8
                        - self
                            .length()
                            .checked_log2()
                            .expect("Range not normalize yet")
                };
                write!(f, "{}/{}", self.first_address(), cidr)
            }
        }
    };
}

impl_ip_range!(Ipv4Range, Ipv4Addr, u32, IPV4_RESERVED);
impl_ip_range!(Ipv6Range, Ipv6Addr, u128, IPV6_RESERVED);

pub mod aggregator;
pub mod parser;
mod utils;

macro_rules! for_wasm {
    ($($item:item)*) => {$(
        #[cfg(target_arch = "wasm32")]
        $item
    )*}
}

for_wasm! {
    mod wasm;
}

#[cfg(test)]
mod tests;
