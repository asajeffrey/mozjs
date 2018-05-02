extern crate libz_sys;

mod generated { include!(concat!(env!("OUT_DIR"), "/jsapi.rs")); }
mod jsglue;
pub mod rooting;

pub mod jsapi {
  pub use rooting;
  pub use generated::root::*;
}
