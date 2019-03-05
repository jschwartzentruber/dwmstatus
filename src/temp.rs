use crate::prelude::*;
use std::fs;
use std::str::FromStr;

const ICON: &str = "🌡";
const INPUT: &str = "/sys/devices/platform/coretemp.0/hwmon/hwmon3/temp1_input";

pub fn status() -> String {
    let data = fs::read_to_string(INPUT).unwrap();
    let temp = i32::from_str(&data.trim_right()).unwrap() / 1000;

    let mut ret = if temp > 75 {
        BAD.to_string()
    } else {
        "".to_string()
    };
    ret += &format!("{}{}°C", ICON, temp);
    ret
}
