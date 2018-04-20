extern crate libz_sys;

pub mod jsapi {
    include!(concat!(env!("OUT_DIR"), "/jsapi.rs"));

    pub use self::root::*;
}
