use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::aggregator::Aggregator;
use crate::parser::parse_cidrs;
use crate::utils::to_string_overflow;
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

#[allow(unused_macros)]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub v4: OutputTriple,
    pub v6: OutputTriple,
    pub invalid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTriple {
    pub ranges: String,
    // TODO: use String for now to avoid exceeding MAX_SAFE_INTEGER
    //  ref: https://github.com/rustwasm/wasm-bindgen/issues/1156
    pub line_count_before: usize,
    pub line_count_after: usize,
    pub address_count_before: String,
    pub address_count_after: String,
}

pub fn _aggregated<R: IpRange>(mut ranges: Vec<R>, reverse: bool) -> OutputTriple {
    let line_count_before = ranges.len();
    console_log!("{:?}", ranges);
    ranges.aggregate();
    console_log!("{:?}", ranges);
    let address_count_before = to_string_overflow(ranges.count_address(), !ranges.is_empty());
    if reverse {
        ranges.reverse();
    }
    console_log!("{:?}", ranges);
    ranges.normalize();
    console_log!("{:?}", ranges);

    let line_count_after = ranges.len();
    let address_count_after = to_string_overflow(ranges.count_address(), !ranges.is_empty());

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
    JsValue::from_serde(&Output {
        v4: _aggregated(v4ranges, reverse),
        v6: _aggregated(v6ranges, reverse),
        invalid: invalid_entries.join("\n"),
    })
    .unwrap()
}
