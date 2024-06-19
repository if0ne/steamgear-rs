#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::useless_transmute)]

#[cfg(target_os = "windows")]
include!("windows_bindings.rs");

#[cfg(target_os = "macos")]
include!("macos_bindings.rs");

#[cfg(target_os = "linux")]
include!("linux_bindings.rs");
