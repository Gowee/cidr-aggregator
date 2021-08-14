use std::mem;

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use num_traits::Zero;

use crate::parser::parse_cidrs;
use crate::aggregator::Aggregator;
use crate::IpRange;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub v4: OutputTriple,
    pub v6: OutputTriple,
    pub invalid: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTriple {
    pub ranges: String,
    // TODO: use String for now to avoid exceeding MAX_SAFE_INTEGER
    //  ref: https://github.com/rustwasm/wasm-bindgen/issues/1156
    pub line_count_before: usize,
    pub line_count_after: usize,
    pub address_count_before: String,
    pub address_count_after: String
}

pub fn _aggregated<R: IpRange>(mut ranges: Vec<R>, reversed: bool) -> OutputTriple {
    ranges.aggregate();
    let line_count_before = ranges.len();
    let address_count_before = ranges.count_address().to_string();
    if reversed {
        ranges.reverse();
    }
    ranges.normalize();

    let line_count_after = ranges.len();
    let address_count_after = ranges.count_address();
    let address_count_after = if address_count_after == R::AddressDecimal::zero() && !ranges.is_empty() {
        if mem::size_of::<R::AddressDecimal>() * 8== 32  {
            String::from("4294967296")
        }
        else {
            String::from("340282366920938463463374607431768211456")
        }
    } else {
        address_count_after.to_string()
    };

    OutputTriple {
        ranges: ranges.export(),
        line_count_before,
        line_count_after,
        address_count_before,
        address_count_after,
    }
}

#[wasm_bindgen]
pub fn aggregate(cidrs: &str, reverse: bool) -> JsValue {
    let (v4ranges, v6ranges, invalid_entries) = parse_cidrs(cidrs);
    JsValue::from_serde( &Output {
        v4: _aggregated(v4ranges, reverse),
        v6: _aggregated(v6ranges, reverse),
        invalid: invalid_entries.join("\n")
    }).unwrap()
} 
