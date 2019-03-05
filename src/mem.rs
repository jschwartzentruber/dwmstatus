use std::io::prelude::*;
use std::{fs, io};
use std::str::FromStr;
use crate::prelude::*;

static ICON: &str= "💻";
static FREE_WARN: u64 = 1 * 1024 * 1024 * 1024;  // 1Gb
static FREE_BAD: u64 = 100 * 1024 * 1024;  // 100Mb

pub fn status() -> String {
    let fd = io::BufReader::new(fs::File::open("/proc/meminfo").unwrap());

    for line in fd.lines() {
        let line = line.unwrap();
        let columns: Vec<&str> = line.splitn(2, ":").collect();
        assert_eq!(columns.len(), 2);
        if columns[0] == "MemAvailable" {
            let columns: Vec<&str> = columns[1].trim().splitn(2, " ").collect();
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[1], "kB");
            let mem = u64::from_str(columns[0]).unwrap() * 1024;
            let mut result = if mem < FREE_BAD { BAD } else if mem < FREE_WARN { WARN } else { "" }.to_string();
            result = result + ICON + " " + &prefixed(mem as f64) + "B";
            return result;
        }
    }
    BAD.to_string() + ICON + " ?"
}
