# cidr-aggregator

[![Crates.io](https://img.shields.io/crates/v/cidr-aggregator)](https://crates.io/crates/cidr-aggregator)
[![docs.rs](https://img.shields.io/docsrs/cidr-aggregator)](https://docs.rs/cidr-aggregator)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/cidr-aggregator)](LICENSE)

Aggregate, normalize, reverse, and difference CIDR (IP range) entries. Both IPv4 and IPv6 are supported.

## Features

- **Aggregate** — merge overlapping and adjacent CIDR ranges into a minimal set
- **Reverse** — compute the complement (all IPs *not* in the given ranges)
- **Difference** — subtract one set of ranges from another
- **Normalize** — split non-canonical ranges into proper CIDR blocks
- **Exclude reserved** — filter out special-purpose addresses (RFC 5735, RFC 6890)
- **IPv4 + IPv6** — unified API via the `IpRange` trait

## Usage

### CLI

Install with Cargo:

```sh
cargo install cidr-aggregator --features cli
```

Pipe CIDRs from stdin:

```sh
cat ip-list.txt | cidr-aggregator
curl -s https://example.com/ip-list.txt | cidr-aggregator
```

Read from a file:

```sh
cidr-aggregator -f input.txt -o output.txt
```

Reverse (compute complement of all input ranges):

```sh
cidr-aggregator -r < ranges.txt
```

Exclude reserved/private IPs:

```sh
cidr-aggregator -x < ranges.txt
```

Show statistics (address count, line count):

```sh
cidr-aggregator -s < ranges.txt
```

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
cidr-aggregator = "0.1"
```

**Aggregate** overlapping and adjacent blocks into a minimal set:

```rust
use cidr_aggregator::{Aggregator, parse_cidrs};

let (mut v4_ranges, _, _) = parse_cidrs("10.0.0.0/24\n10.0.1.0/24\n10.0.0.128/25");
// Adjacent .0.0/24 + .1.0/24 → .0.0/23;  .0.128/25 is inside the /23
v4_ranges.aggregate();
// Aggregate produces minimal ranges but not necessarily canonical CIDR
// blocks — normalize() is required before export().
v4_ranges.normalize();
assert_eq!(v4_ranges.export(), "10.0.0.0/23");
```

**Chain** operations in a pipeline — filter reserved addresses, then reverse:

```rust
use cidr_aggregator::{Aggregator, parse_cidrs, IpRange, Ipv6Range};

let (_, v6_ranges, _) = parse_cidrs("2001:db8::/32\n64:ff9b::/96");
println!(
    "{}",
    v6_ranges
        .aggregated()
        .differenced(Ipv6Range::reserved()) // strip RFC 6890 reserved blocks
        // Normalize is required to produce valid CIDR blocks after aggregation
        // and difference, which may leave non-canonical ranges.
        .normalized()
        .reversed()
        .export());
```

See the [API documentation](https://docs.rs/cidr-aggregator) for all available operations.

### Web App

Also available as a WASM-powered web app at:

**[cidr-aggregator.pages.dev](https://cidr-aggregator.pages.dev)**

## License

MIT OR Apache-2.0
