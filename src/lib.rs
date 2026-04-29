//! Parse CIDR strings, aggregate, reverse, and difference IP ranges, then
//! normalize back to CIDR notation.
//!
//! Supports both IPv4 and IPv6. The core abstraction is the [`IpRange`] trait,
//! implemented by [`Ipv4Range`] and [`Ipv6Range`]. Operations like
//! `.aggregate()`, `.reverse()`, `.normalize()`, and `.export()` are provided
//! on `Vec` of either range type via the [`Aggregator`] trait.
//!
//! # Quick start
//!
//! Aggregate overlapping and adjacent CIDR blocks into a minimal set:
//!
//! ```
//! use cidr_aggregator::{parse_cidrs, Aggregator};
//!
//! let (mut v4_ranges, _, _) = parse_cidrs("10.0.0.0/24\n10.0.1.0/24\n10.0.0.128/25");
//! // 10.0.0.0/24 and 10.0.1.0/24 are adjacent → merge to 10.0.0.0/23
//! // 10.0.0.128/25 is already covered by the /23 → absorbed
//! v4_ranges.aggregate();
//! // Aggregate produces a minimal set but not necessarily canonical CIDR
//! // blocks — normalize() is required before export().
//! v4_ranges.normalize();
//! assert_eq!(v4_ranges.export(), "10.0.0.0/23");
//! ```
//!
//! Chain operations in a pipeline — filter reserved addresses, then reverse:
//!
//! ```
//! use cidr_aggregator::{parse_cidrs, Aggregator, IpRange, Ipv6Range};
//!
//! let (_, v6_ranges, _) = parse_cidrs("2001:db8::/32\n64:ff9b::/96");
//!
//! println!(
//!     "{}",
//!     v6_ranges
//!         .aggregated()
//!         .differenced(Ipv6Range::reserved()) // strip RFC 6890 reserved blocks
//!         // Normalize is required to produce valid CIDR blocks after aggregation
//!         // and difference, which may leave non-canonical ranges.
//!         .normalized()
//!         .reversed()
//!         .export()
//! );
//! ```
//!
//! A WASM build of this crate powers the web app at
//! <https://cidr-aggregator.pages.dev>.

use std::fmt::{self, Debug, Display};
use std::hash::Hash;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use crate::utils::{
    ip_addr_to_bit_length, ip_addr_trailing_zeros, MathLog2, IPV4_RESERVED, IPV6_RESERVED,
};
use num_traits::{Bounded, NumAssignOps, NumCast, PrimInt, WrappingAdd, Zero};

/// An inclusive IPv4 range `[first, last]` stored as `u32`.
///
/// Ranges can be created from CIDR notation via [`EitherIpRange::from_str`]
/// or from a `(first_address, last_address)` pair via `from_cidr_pair_decimal`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ipv4Range(u32, u32);

/// An inclusive IPv6 range `[first, last]` stored as `u128`.
///
/// Ranges can be created from CIDR notation via [`EitherIpRange::from_str`]
/// or from a `(first_address, last_address)` pair via `from_cidr_pair_decimal`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ipv6Range(u128, u128);

/// Either an IPv4 or IPv6 range, used for parsing CIDR strings.
///
/// Use [`into_v4`](EitherIpRange::into_v4) / [`into_v6`](EitherIpRange::into_v6)
/// or match to extract the inner range.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EitherIpRange {
    V4(Ipv4Range),
    V6(Ipv6Range),
}

impl EitherIpRange {
    /// Extract the `Ipv4Range`, or `None` if this is an IPv6 range.
    pub fn into_v4(self) -> Option<Ipv4Range> {
        match self {
            EitherIpRange::V4(r) => Some(r),
            _ => None,
        }
    }

    /// Extract the `Ipv6Range`, or `None` if this is an IPv4 range.
    pub fn into_v6(self) -> Option<Ipv6Range> {
        match self {
            EitherIpRange::V6(r) => Some(r),
            _ => None,
        }
    }

    /// Returns `true` if this is an IPv4 range.
    pub fn is_v4(self) -> bool {
        matches!(self, EitherIpRange::V4(_))
    }

    /// Returns `true` if this is an IPv6 range.
    pub fn is_v6(self) -> bool {
        matches!(self, EitherIpRange::V6(_))
    }
}

impl From<Ipv4Range> for EitherIpRange {
    fn from(r: Ipv4Range) -> Self {
        EitherIpRange::V4(r)
    }
}

impl From<Ipv6Range> for EitherIpRange {
    fn from(r: Ipv6Range) -> Self {
        EitherIpRange::V6(r)
    }
}

impl FromStr for EitherIpRange {
    type Err = ();

    fn from_str(s: &str) -> Result<EitherIpRange, Self::Err> {
        if let Some((ip, cidr)) = s.split_once("/").or(Some((s, ""))) {
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

/// Core abstraction for an IP range.
///
/// Implemented by [`Ipv4Range`] and [`Ipv6Range`] via the `impl_ip_range!` macro.
/// This trait enables generic algorithms in the [`aggregator`] module that work
/// identically for both address families.
///
/// Note: [`Display`] (and therefore [`export`](Aggregator::export)) **panics**
/// if the range has not been normalized — its length must be a power of two.
pub trait IpRange: Copy + Eq + Ord + Display + Debug + Hash + 'static {
    type Address;
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

    /// The full IP space — from `0` to `max_value`.
    fn full() -> Self {
        Self::from_cidr_pair_decimal((
            Self::AddressDecimal::zero(),
            Self::AddressDecimal::max_value(),
        ))
    }

    /// Reserved / special-purpose address blocks (RFC 5735, RFC 6890).
    fn reserved() -> &'static [Self];
}

macro_rules! impl_ip_range {
    ($ip_range: ident, $address_type: ident, $decimal_type: ident, $reserved: ident) => {
        #[allow(clippy::legacy_numeric_constants)]
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

        /// Formats as CIDR notation (e.g. `192.168.1.0/24`).
        ///
        /// **Panics** if the range has not been normalized (its length must be a
        /// power of two, since that's what defines a valid CIDR prefix length).
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

pub use aggregator::Aggregator;
pub use parser::parse_cidrs;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
mod wasm;

#[cfg(test)]
mod tests;
