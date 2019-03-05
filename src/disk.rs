use crate::prelude::*;
use libc::{endmntent, getmntent, setmntent, statvfs};
use std::{ffi, mem};

static ICON: &str = "🖴";
static PATH: &str = "/";

pub fn status() -> String {
    let mut buf: statvfs = unsafe { mem::uninitialized() };
    let mut mounted = false;
    let path = ffi::CString::new(PATH).unwrap();

    if unsafe { statvfs(path.as_ptr(), &mut buf) } == -1 {
        // If statvfs errors, e.g., due to the path not existing,
        // we consider the device not mounted.
        mounted = false;
    } else {
        let mtab_cstr = ffi::CString::new("/etc/mtab").unwrap();
        let r_cstr = ffi::CString::new("r").unwrap();
        let mntentfile = unsafe { setmntent(mtab_cstr.as_ptr(), r_cstr.as_ptr()) };

        loop {
            let m = unsafe { getmntent(mntentfile) };
            if m.is_null() {
                break;
            }
            let mnt_dir = unsafe { ffi::CStr::from_ptr((*m).mnt_dir) };
            if mnt_dir.to_str().unwrap() == PATH {
                mounted = true;
                break;
            }
        }
        unsafe { endmntent(mntentfile) };
    }
    if !mounted {
        BAD.to_string() + ICON + " ?"
    } else {
        let percent_free = 100.0 * buf.f_bfree as f64 / buf.f_blocks as f64;
        let mut result = if percent_free < 10.0 { WARN } else { "" }.to_string() + ICON + " ";
        result += &prefixed(buf.f_bsize as f64 * buf.f_bavail as f64);
        result += "B";
        result
    }
}
