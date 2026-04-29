use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::aggregator::{self, Aggregator};
use crate::parser::parse_cidrs;
use crate::utils::to_string_overflow;
use crate::IpRange;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
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
    pub line_count_before: usize,
    pub line_count_after: usize,
    pub address_count_before: String,
    pub address_count_after: String,
}

fn build_output<R: IpRange>(
    mut ranges: Vec<R>,
    reverse: bool,
    exclude_reserved: bool,
) -> OutputTriple {
    let line_count_before = ranges.len();
    ranges.aggregate();
    let address_count_before = to_string_overflow(ranges.count_address(), !ranges.is_empty());
    ranges = aggregator::process(ranges, reverse, exclude_reserved);
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
pub fn aggregate(cidrs: &str, reverse: bool, exclude_reserved: bool) -> JsValue {
    let (v4ranges, v6ranges, invalid_entries) = parse_cidrs(cidrs);
    JsValue::from_serde(&Output {
        v4: build_output(v4ranges, reverse, exclude_reserved),
        v6: build_output(v6ranges, reverse, exclude_reserved),
        invalid: invalid_entries.join("\n"),
    })
    .unwrap()
}
