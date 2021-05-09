#![allow(improper_ctypes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::all)]
#![allow(rustdoc::broken_intra_doc_links)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const SCRIPTS_TAR_BZ2: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/scripts.tar.bz2"));

pub const SCRIPTS_FINGERPRINT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/scripts.sha256"));
