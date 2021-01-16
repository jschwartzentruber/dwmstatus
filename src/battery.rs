use crate::prelude::*;
use std::io::prelude::*;
use std::str::FromStr;
use std::{fs, io};

const CHARGE: &str = "🗲";
const DISCHARGE: &str = "🔋";
const FULL: &str = "🔌";
const UNKNOWN: &str = "???";

#[derive(PartialEq)]
enum BatteryState {
    Discharge,
    Charge,
    Full,
    Unknown,
}

fn match_consume(string: &mut &str, prefix: &str) -> bool {
    if string.starts_with(prefix) {
        *string = &string[prefix.len()..];
        true
    } else {
        false
    }
}

pub fn status() -> Option<String> {
    let fd = match fs::File::open("/sys/class/power_supply/BAT0/uevent") {
        Ok(fd) => io::BufReader::new(fd),
        Err(_err) => {
            return None;
        },
    };
    let mut state = BatteryState::Unknown;
    let mut voltage = -1;
    let mut full_design = 0;  // in uAh
    let mut full_last = 0;  // in uAh
    let mut remaining = 0;  // in uAh
    let mut present_rate = 0;  // in uA, always non-negative
    let mut seconds_remaining = 0;
    let mut watt_as_unit = false;

    for line in fd.lines() {
        let line = line.expect("Failed to read battery line");
        let mut l = &line[..];

        if match_consume(&mut l, "POWER_SUPPLY_") {
            if match_consume(&mut l, "STATUS=") {
                state = match l {
                    "Discharging" => BatteryState::Discharge,
                    "Charging" => BatteryState::Charge,
                    "Full" => BatteryState::Full,
                    _ => BatteryState::Unknown,
                };
            } else if match_consume(&mut l, "ENERGY_NOW=") {
                watt_as_unit = true;
                remaining = i32::from_str(l).unwrap();
            } else if match_consume(&mut l, "CHARGE_NOW=") {
                watt_as_unit = false;
                remaining = i32::from_str(l).unwrap();
            } else if match_consume(&mut l, "CURRENT_NOW=") {
                present_rate = i32::from_str(l).unwrap().abs();
            } else if match_consume(&mut l, "VOLTAGE_NOW=") {
                voltage = i32::from_str(l).unwrap().abs();

            // on some systems POWER_SUPPLY_POWER_NOW does not exist, but actually
            // it is the same as POWER_SUPPLY_CURRENT_NOW but with μWh as
            // unit instead of μAh. We will calculate it as we need it
            // later.
            } else if match_consume(&mut l, "POWER_NOW=") {
                present_rate = i32::from_str(l).unwrap().abs();
            } else if match_consume(&mut l, "CHARGE_FULL_DESIGN=") || match_consume(&mut l, "ENERGY_FULL_DESIGN=") {
                full_design = i32::from_str(l).unwrap();
            } else if match_consume(&mut l, "ENERGY_FULL=") || match_consume(&mut l, "CHARGE_FULL=") {
                full_last = i32::from_str(l).unwrap();
            }
        }
    }

    // the difference between POWER_SUPPLY_ENERGY_NOW and
    // POWER_SUPPLY_CHARGE_NOW is the unit of measurement. The energy is
    // given in mWh, the charge in mAh. So calculate every value given in
    // ampere to watt
    if !watt_as_unit && voltage != -1 {
        let voltage = voltage as f64;
        present_rate = ((voltage / 1000.0) * (present_rate as f64 / 1000.0)).floor() as i32;
        remaining = ((voltage / 1000.0) * (remaining as f64 / 1000.0)).floor() as i32;
        full_design = ((voltage / 1000.0) * (full_design as f64 / 1000.0)).floor() as i32;
        full_last = ((voltage / 1000.0) * (full_last as f64 / 1000.0)).floor() as i32;
    }

    let last_full_capacity = true;
    let full = if last_full_capacity { full_last } else { full_design };
    if full < 0 {
        // We have no physical measurements and no estimates. Nothing
        // much we can report, then.
        Some(BAD.to_string() + "No battery")
    } else {
        let mut percentage_remaining = 100.0 * remaining as f64 / full as f64;

        // Some batteries report POWER_SUPPLY_CHARGE_NOW=<full_design> when fully
        // charged, even though that’s plainly wrong. For people who chose to see
        // the percentage calculated based on the last full capacity, we clamp the
        // value to 100%, as that makes more sense.
        // See http://bugs.debian.org/785398
        if last_full_capacity && percentage_remaining > 100.0 {
            percentage_remaining = 100.0;
        }

        if present_rate > 0 && state != BatteryState::Full {
            seconds_remaining = if state == BatteryState::Charge {
                (3600.0 * (full - remaining) as f64 / present_rate as f64) as i32
            } else if state == BatteryState::Discharge {
                (3600.0 * remaining as f64 / present_rate as f64) as i32
            } else {
                0
            };
        }

        let mut ret = if state == BatteryState::Discharge && percentage_remaining < 25.0 {
            WARN.to_string()
        } else if state == BatteryState::Unknown || (state == BatteryState::Discharge && percentage_remaining < 5.0) {
            BAD.to_string()
        } else {
            "".to_string()
        };
        match state {
            BatteryState::Discharge | BatteryState::Charge | BatteryState::Unknown => {
                ret += if state == BatteryState::Charge { CHARGE } else if state == BatteryState::Discharge { DISCHARGE } else { UNKNOWN };
                ret += &format!(" {:0.0}% (", percentage_remaining);
                if seconds_remaining > 3600 {
                    ret += &format!("{}h{:02})", seconds_remaining / 3600, (seconds_remaining % 3600) / 60);
                } else {
                    ret += &format!("{}m{:02})", seconds_remaining / 60, seconds_remaining % 60);
                }
            },
            BatteryState::Full => { ret += FULL; ret += " 100%"; },
        };
        Some(ret)
    }
}
