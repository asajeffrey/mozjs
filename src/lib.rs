extern crate libz_sys;

mod generated { include!(concat!(env!("OUT_DIR"), "/jsapi.rs")); }
mod jsglue;

pub use generated::root as jsapi;
