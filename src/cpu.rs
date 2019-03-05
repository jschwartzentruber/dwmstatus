use std::io::prelude::*;
use std::{fs, io};
use std::str::FromStr;
use crate::prelude::*;

pub fn status() -> String {
    let fd = io::BufReader::new(fs::File::open("/proc/cpuinfo").unwrap());

    for line in fd.lines() {
        let line = line.unwrap();
        if line.starts_with("cpu MHz") {
            let columns: Vec<&str> = line.splitn(2, ":").collect();
            assert_eq!(columns.len(), 2);
            let hz = f64::from_str(columns[1].trim()).unwrap() * 1024.0 * 1024.0;
            return prefixed(hz) + "Hz";
        }
    }
    (WARN.to_string() + "?")
}

