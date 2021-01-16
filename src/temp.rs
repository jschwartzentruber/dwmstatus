use crate::prelude::*;
use std::io::prelude::*;
use std::{fs, io};
use std::str::FromStr;
use glob::glob;

const ICON: &str = "🌡";

pub struct Temp {
    path: Option<String>,
}

impl Temp {
    pub fn new() -> Self {
        let mut result = None;
        for entry in glob("/sys/class/thermal/thermal_zone*/type").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let file = fs::File::open(&path).expect("Failed to open thermal type path");
                    for line in io::BufReader::new(file).lines() {
                        if line.expect("Failed to read line") == "x86_pkg_temp" {
                            let parent = path.parent().expect("Failed to get thermal type parent");
                            let temp = parent.join("temp");
                            let temp_str = temp.to_str().expect("Failed to join thermal temp path");
                            result = Some(temp_str.to_string());
                            break;
                        }
                    }
                    if result != None {
                        break;
                    }
                },
                Err(_err) => (),
            }
        }
        Temp {
            path: result
        }
    }

    pub fn status(&self) -> String {
        match &self.path {
            Some(path) => {
                let data = fs::read_to_string(path).expect("Failed to open thermal temp path");
                let temp = i32::from_str(&data.trim_end()).expect("Failed to parse thermal temp") / 1000;

                let mut ret = if temp > 75 {
                    BAD.to_string()
                } else {
                    "".to_string()
                };
                ret += &format!("{}{}°C", ICON, temp);
                ret
            },
            None => {
                let mut ret = BAD.to_string();
                ret += &format!("{}err", ICON);
                ret
            }
        }
    }
}
