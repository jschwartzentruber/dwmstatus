#![macro_use]

macro_rules! perror_check {
    ($f: ident ( $($x: expr),* ) ) => {
        let err = unsafe { $f( $($x),* ) };
        if err < 0 {
            use std::ffi;
            use libc;
            let errno = unsafe { *libc::__errno_location() };
            let err_str = unsafe { ffi::CStr::from_ptr(libc::strerror(errno)) }.to_str().expect("Failed to convert strerror result to string");
            panic!(format!("{}: {}", stringify!($f), err_str));
        }
    }
}

pub static NORM: &'static str = "\x01";
pub static GOOD: &'static str = "\x03";
pub static WARN: &'static str = "\x04";
pub static BAD: &'static str = "\x05";

static PREFIXES: &str = "kMGT";

pub fn prefixed0(num: f64) -> String {
    let mut prefix_idx = 0;
    let mut nv = num;
    while nv > 1024.0 && prefix_idx < PREFIXES.len() - 1 {
         prefix_idx += 1;
         nv /= 1024.0;
    }
    let mut result = format!("{:0.0}", nv);
    if prefix_idx > 0 {
        result += &PREFIXES[prefix_idx - 1..prefix_idx];
    }
    result
}

pub fn prefixed(num: f64) -> String {
    let mut prefix_idx = 0;
    let mut nv = num;
    while nv > 1024.0 && prefix_idx < PREFIXES.len() - 1 {
         prefix_idx += 1;
         nv /= 1024.0;
    }
    let mut result = format!("{:0.1}", nv);
    if prefix_idx > 0 {
        result += &PREFIXES[prefix_idx - 1..prefix_idx];
    }
    result
}
