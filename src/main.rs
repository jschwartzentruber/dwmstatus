use chrono;
mod prelude;
mod battery;
mod cpu;
mod disk;
mod ifaddrs;
mod load;
mod mem;
mod temp;
mod vol;
mod wlaninfo;

use std::{ffi, ptr, thread, time};
use chrono::prelude::*;
use crate::prelude::*;
use x11::xlib;

static ALLOW_EARLY: u64 = 1;  // seconds to allow Fields to update early

struct Field<T> {
    updater: Box<dyn Fn(Option<&mut T>) -> String>,
    value: String,
    period: time::Duration,
    next: time::Instant,
}

impl<T> Field<T> {
    fn new(updater: Box<dyn Fn(Option<&mut T>) -> String>, period: time::Duration) -> Self {
        Self { updater: updater, value: "".to_string(), period: period, next: time::Instant::now() }
    }
}

struct Status<T> {
    fields : Vec<Field<T>>,
    idx_by_time : Vec<usize>,
    data: Option<T>,
}

impl<T> Status<T> {
    fn new(data: Option<T>) -> Self {
        Self { fields: vec![], idx_by_time: vec![], data: data }
    }

    // ordering of tasks is by task.next. the 0th element has changed it's `next`,
    // so figure out where it needs to go, and put tasks back into sorted order
    fn reorder_first(&mut self) {
        if self.idx_by_time.len() > 1 {
            let mut l = 0;
            while l + 1 < self.idx_by_time.len() && self.fields[self.idx_by_time[0]].next >= self.fields[self.idx_by_time[l + 1]].next {
                l += 1;
            }
            self.idx_by_time[..l + 1].rotate_left(1)
        }
    }

    fn add(&mut self, updater: Box<dyn Fn(Option<&mut T>) -> String>, period: time::Duration) {
        let new_task = Field::new(updater, period);
        for i in 0..self.idx_by_time.len() {
            if new_task.next < self.fields[self.idx_by_time[i]].next {
                self.idx_by_time.insert(i, self.fields.len());
                self.fields.push(new_task);
                return;
            }
        }
        self.idx_by_time.push(self.fields.len());
        self.fields.push(new_task);
    }

    fn run(&mut self) {
        if self.fields.len() > 0 {
            let now = time::Instant::now();
            while self.fields[self.idx_by_time[0]].next <= (now + time::Duration::new(ALLOW_EARLY, 0)) {
                let task = &mut self.fields[self.idx_by_time[0]];
                let updater = &mut task.updater;
                task.value = updater(self.data.as_mut());
                task.next = now + task.period;
                self.reorder_first();
            }
        }
    }

    fn sleep(&self) {
        let now = time::Instant::now();
        if self.fields[self.idx_by_time[0]].next <= now {
            return;
        }
        thread::sleep(self.fields[self.idx_by_time[0]].next - now);

    }

    fn print(&self) {
        for t in &self.fields[..] {
            print!("{}", t.value);
        }
        println!("");
    }

    fn xsetroot(&self, dpy: *mut xlib::Display) {
        let result = ffi::CString::new({
            let mut result = "".to_string();
            for t in &self.fields[..] {
                result += &t.value;
            }
            result
        }).unwrap();

        unsafe {
            xlib::XStoreName(dpy, xlib::XDefaultRootWindow(dpy), result.as_ptr());
            xlib::XSync(dpy, false as i32);
        }
    }
}

fn status_volume(data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    data.unwrap().0.status()
}

fn status_load(_data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    load::status()
}

fn status_lan(data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    data.unwrap().1.status_lan()
}

fn status_vpn(data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    data.unwrap().1.status_vpn()
}

fn status_wlan(data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    data.unwrap().1.status_wlan()
}

fn status_battery(_data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    battery::status()
}

fn status_disk(_data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    disk::status()
}

fn status_ram(_data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    mem::status()
}

fn status_cpuspeed(_data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    cpu::status()
}

fn status_temp(_data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    temp::status()
}

fn status_time(_data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

fn sep(_data: Option<&mut (vol::Alsa, ifaddrs::Interfaces)>) -> String {
    (NORM.to_string() + " | ")
}

fn main() {
    let data = (vol::Alsa::new(), ifaddrs::Interfaces::new());
    let mut status = Status::new(Some(data));
    status.add(Box::new(status_vpn), time::Duration::new(5, 0));
    status.add(Box::new(sep), time::Duration::new(365 * 24 * 60 * 60, 0));
    status.add(Box::new(status_wlan), time::Duration::new(5, 0));
    status.add(Box::new(sep), time::Duration::new(365 * 24 * 60 * 60, 0));
    status.add(Box::new(status_lan), time::Duration::new(5, 0));
    status.add(Box::new(sep), time::Duration::new(365 * 24 * 60 * 60, 0));
    status.add(Box::new(status_battery), time::Duration::new(15, 0));
    status.add(Box::new(sep), time::Duration::new(365 * 24 * 60 * 60, 0));
    status.add(Box::new(status_disk), time::Duration::new(15, 0));
    status.add(Box::new(sep), time::Duration::new(365 * 24 * 60 * 60, 0));
    status.add(Box::new(status_ram), time::Duration::new(15, 0));
    status.add(Box::new(sep), time::Duration::new(365 * 24 * 60 * 60, 0));
    status.add(Box::new(status_cpuspeed), time::Duration::new(15, 0));
    status.add(Box::new(sep), time::Duration::new(365 * 24 * 60 * 60, 0));
    status.add(Box::new(status_load), time::Duration::new(5, 0));
    status.add(Box::new(sep), time::Duration::new(365 * 24 * 60 * 60, 0));
    status.add(Box::new(status_temp), time::Duration::new(5, 0));
    status.add(Box::new(sep), time::Duration::new(365 * 24 * 60 * 60, 0));
    status.add(Box::new(status_volume), time::Duration::new(5, 0));
    status.add(Box::new(sep), time::Duration::new(365 * 24 * 60 * 60, 0));
    status.add(Box::new(status_time), time::Duration::new(5, 0));

    let display = unsafe { xlib::XOpenDisplay(ptr::null_mut()) };
    if display.is_null() {
        panic!("dwmstatus: cannot open display.");
    }

    loop {
        status.run();
        status.xsetroot(display);
        status.sleep();
    }
}

#[cfg(test)]
mod tests {
    use std::time;
    use super::Status;

    #[test]
    fn run_reorder() {
        let mut status = Status::new(None);
        //let mut runs : Vec<usize> = vec![];
        status.add(Box::new(|_: Option<&mut ()>| { "A".to_string() }), time::Duration::new(30, 0));
        status.add(Box::new(|_: Option<&mut ()>| { "B".to_string() }), time::Duration::new(20, 0));
        status.add(Box::new(|_: Option<&mut ()>| { "C".to_string() }), time::Duration::new(10, 0));
        assert_eq!(status.idx_by_time, vec![0, 1, 2]);
        status.run();
        assert_eq!(status.fields[0].value, "A");
        assert_eq!(status.fields[1].value, "B");
        assert_eq!(status.fields[2].value, "C");
        assert_eq!(status.idx_by_time, vec![2, 1, 0]);
        status.add(Box::new(|_: Option<&mut ()>| { "D".to_string() }), time::Duration::new(15, 0));
        assert_eq!(status.idx_by_time, vec![3, 2, 1, 0]);
        status.run();
        assert_eq!(status.fields[3].value, "D");
        assert_eq!(status.idx_by_time, vec![2, 3, 1, 0]);
    }
}
