use std::{ffi, mem, ptr, time};
use libc::{freeifaddrs, getifaddrs, getnameinfo};
use crate::prelude::*;
use crate::wlaninfo;

static ICON_VPN: &str = "🔒";
static ICON_WLAN: &str = "📶";
static ICON_LAN: &str = "🖧";

pub struct Interfaces {
    lan: Option<IFAddr>,
    wlan: Option<IFAddr>,
    vpn: Option<IFAddr>,
    last: time::Instant,
}

struct IFAddrs {
    ifap: *mut libc::ifaddrs,
    cp: *const libc::ifaddrs,
}

impl IFAddrs {
    fn new() -> Self {
        let mut ifap = ptr::null_mut();
        perror_check!(getifaddrs(&mut ifap));
        IFAddrs { ifap: ifap, cp: ifap }
    }
}

impl Iterator for IFAddrs {
    type Item = IFAddr;

    fn next(&mut self) -> Option<Self::Item> {
        let cp = unsafe { self.cp.as_ref() };
        match cp {
            None => None,
            Some(ifa) => {
                let mut addr_raw: Vec<i8> = vec![0; 20];
                let len = mem::size_of::<libc::sockaddr_in>();
                let ret = if !ifa.ifa_addr.is_null() && unsafe { ifa.ifa_addr.as_ref() }.unwrap().sa_family as i32 == libc::AF_INET {
                    unsafe { getnameinfo(ifa.ifa_addr, len as u32, addr_raw.as_mut_ptr(), 20, ptr::null_mut(), 0, libc::NI_NUMERICHOST) }
                } else {
                    0
                };
                let (addr, has_addr) = if ret != 0 {
                    let err_str = unsafe { ffi::CStr::from_ptr(libc::gai_strerror(ret)) }.to_str().unwrap();
                    eprintln!("getnameinfo(): {}", err_str);
                    ("error", false)
                } else if addr_raw[0] == 0 {
                    ("no IP", false)
                } else {
                    (unsafe { ffi::CStr::from_ptr(addr_raw.as_ptr()) }.to_str().unwrap(), true)
                };
                let result = IFAddr {
                    addr: addr.to_string(),
                    flags: ifa.ifa_flags,
                    has_addr: has_addr,
                    name: unsafe { ffi::CStr::from_ptr(ifa.ifa_name) }.to_str().unwrap().to_string(),
                };
                self.cp = ifa.ifa_next;
                Some(result)
            },
        }
    }
}

impl Drop for IFAddrs {
    fn drop(&mut self) {
        if !self.ifap.is_null() {
            unsafe { freeifaddrs(self.ifap); }
        }
    }
}

#[derive(Debug)]
struct IFAddr {
    addr: String,
    flags: libc::c_uint,
    has_addr: bool,
    name: String,
}

impl IFAddr {
    fn is_lan(&self) -> bool {
        (self.name.starts_with("eth") || self.name.starts_with("enp"))
    }

    fn is_wlan(&self) -> bool {
        (self.name.starts_with("wlan") || self.name.starts_with("wlp"))
    }

    fn is_vpn(&self) -> bool {
        (self.name.starts_with("tun") || self.name.starts_with("vti"))
    }
}

impl Interfaces {
    pub fn new() -> Self {
        Self { lan: None, wlan: None, vpn: None, last: time::Instant::now() - time::Duration::new(5, 0) }
    }

    // Return the IP address for the given interface or "no IP" if the
    // interface is up and running but hasn't got an IP address yet
    pub fn update(&mut self) {
        let now = time::Instant::now();
        if now - self.last < time::Duration::new(1, 0) {
            return;
        }

        self.last = now;

        self.lan = None;
        self.vpn = None;
        self.wlan = None;

        let mut lan_has_addr = false;
        let mut vpn_has_addr = false;
        let mut wlan_has_addr = false;

        for iface in IFAddrs::new() {
            //println!("{:?} ({})", iface, iface.flags & (libc::IFF_RUNNING as u32));
            if iface.flags & (libc::IFF_RUNNING as u32) != 0 {
                if iface.is_lan() {
                    if lan_has_addr && !vpn_has_addr && iface.has_addr {
                        //println!("vpn -> {:?}", iface);
                        self.vpn = Some(iface);
                        vpn_has_addr = self.vpn.as_ref().unwrap().has_addr;
                    } else if !lan_has_addr {
                        //println!("lan -> {:?}", iface);
                        self.lan = Some(iface);
                        lan_has_addr = self.lan.as_ref().unwrap().has_addr;
                    }
                } else if iface.is_vpn() && !vpn_has_addr {
                    //println!("vpn -> {:?}", iface);
                    self.vpn = Some(iface);
                    vpn_has_addr = self.vpn.as_ref().unwrap().has_addr;
                } else if iface.is_wlan() && !wlan_has_addr {
                    //println!("wlan -> {:?}", iface);
                    self.wlan = Some(iface);
                    wlan_has_addr = self.wlan.as_ref().unwrap().has_addr;
                }
            }
        }
    }

    pub fn status_vpn(&mut self) -> String {
        self.update();
        match &self.vpn {
            None => (BAD.to_string() + ICON_VPN + " down"),
            Some(ifaddr) => (GOOD.to_string() + ICON_VPN + " " + &ifaddr.addr),
        }
    }

    pub fn status_wlan(&mut self) -> String {
        self.update();
        match &self.wlan {
            Some(ifaddr) => {
                let mut first = "";
                let info = wlaninfo::WLanInfo::new(&ifaddr.name);

                let mut result = match info.quality {
                    Some(quality) => if quality < 50.0 { WARN } else { GOOD },
                    None => GOOD,
                }.to_string() + ICON_WLAN + " " + &ifaddr.addr + " (";

                match info.essid {
                    Some(essid) => {
                        result = result + first + &essid;
                        first = " ";
                    },
                    None => (),
                };

                match info.quality {
                    Some(quality) => {
                        result = result + first + &format!("{:0.1}%", quality);
                        first = " ";
                    },
                    None => (),
                };

                /*
                match info.frequency {
                    Some(freq) => {
                        result = result + first + "freq " + &prefixed(freq) + "Hz";
                        first = " ";
                    },
                    None => (),
                };
                */

                match info.bitrate {
                    Some(bitrate) => {
                        result = result + first + &prefixed(bitrate) + "b/s";
                        //first = " ";
                    },
                    None => (),
                };
                (result + ")")
            },
            None => (BAD.to_string() + ICON_WLAN + " down"),
        }
    }

    pub fn status_lan(&mut self) -> String {
        self.update();
        match &self.lan {
            None => (BAD.to_string() + ICON_LAN + " down"),
            Some(ifaddr) => (GOOD.to_string() + ICON_LAN + " " + &ifaddr.addr),
        }
    }
}
