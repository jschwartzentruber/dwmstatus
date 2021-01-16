use std::{ffi, fs, mem, ptr, time};
use std::str::FromStr;
use libc::{freeifaddrs, getifaddrs, getnameinfo};
use crate::prelude::*;
use crate::wlaninfo;

static ICON_VPN: &str = "🔒";
static ICON_WLAN: &str = "📶";
static ICON_LAN: &str = "🖧";
static ICON_UP: &str = "↑";
static ICON_DOWN: &str = "↓";

pub struct Interfaces {
    lan: Option<IFAddr>,
    wlan: Option<IFAddr>,
    vpn: Option<IFAddr>,
    last: time::Instant,
    show_lan_down: bool,
    show_wlan_down: bool,
    show_vpn_down: bool,
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
                let name = unsafe { ffi::CStr::from_ptr(ifa.ifa_name) }.to_str().unwrap().to_string();

                let rx_bytes = {
                    match fs::read_to_string("/sys/class/net/".to_string() + &name + "/statistics/rx_bytes") {
                        Ok(contents) => {
                            //println!("rx bytes for {}: {}", &name, &contents.trim_end());
                            match u64::from_str(&contents.trim_end()) {
                                Ok(value) => Some(value),
                                Err(_err) => None,
                            }
                        },
                        Err(_err) => None,
                    }
                };

                let tx_bytes = {
                    match fs::read_to_string("/sys/class/net/".to_string() + &name + "/statistics/tx_bytes") {
                        Ok(contents) => {
                            //println!("tx bytes for {}: {}", &name, &contents.trim_end());
                            match u64::from_str(&contents.trim_end()) {
                                Ok(value) => Some(value),
                                Err(_err) => None,
                            }
                        },
                        Err(_err) => None,
                    }
                };

                let result = IFAddr {
                    addr: addr.to_string(),
                    flags: ifa.ifa_flags,
                    has_addr: has_addr,
                    name: name,
                    tx_bytes: tx_bytes,
                    rx_bytes: rx_bytes,
                    tx_speed: None,
                    rx_speed: None,
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

fn speed_to_str(speed: Option<f64>, icon: &str) -> String {
    match speed {
        Some(speed) => {
            let mut result = " ".to_string();
            result += icon;
            if speed < 1024. {
                result += &prefixed0(speed);
            } else {
                result += &prefixed(speed);
            }
            result += "B/s";
            result
        },
        None => "".to_string(),
    }
}

#[derive(Debug)]
struct IFAddr {
    addr: String,
    flags: libc::c_uint,
    has_addr: bool,
    name: String,
    rx_bytes: Option<u64>,
    tx_bytes: Option<u64>,
    rx_speed: Option<f64>,
    tx_speed: Option<f64>,
}

impl IFAddr {
    fn is_lan(&self) -> bool {
        self.name.starts_with("eth") || self.name.starts_with("en")
    }

    fn is_wlan(&self) -> bool {
        self.name.starts_with("wl")
    }

    fn is_vpn(&self) -> bool {
        self.name.starts_with("tun") || self.name.starts_with("vti")
    }

    fn calc_speeds(&mut self, last: &Option<IFAddr>, interval: time::Duration) {
        //println!("calc speeds for interval {}", interval.as_micros() as f64 / 1e6);
        match last {
            Some(last) => {
                self.tx_speed = self._calc_tx_speed(last, interval);
                self.rx_speed = self._calc_rx_speed(last, interval);
            },
            None => {
                //println!("last is None");
                ()
            },
        };
    }

    fn _calc_tx_speed(&self, last: &IFAddr, interval: time::Duration) -> Option<f64> {
        let last_bytes = match last.tx_bytes {
            Some(bytes) => bytes,
            None => {
                //println!("tx last is None");
                return None;
            },
        };
        let now_bytes = match self.tx_bytes {
            Some(bytes) => bytes,
            None => {
                //println!("tx now is None");
                return None;
            },
        };
        Some((now_bytes - last_bytes) as f64 / interval.as_micros() as f64 * 1e6)
    }

    fn _calc_rx_speed(&mut self, last: &IFAddr, interval: time::Duration) -> Option<f64> {
        let last_bytes = match last.rx_bytes {
            Some(bytes) => bytes,
            None => {
                //println!("rx last is None");
                return None;
            },
        };
        let now_bytes = match self.rx_bytes {
            Some(bytes) => bytes,
            None => {
                //println!("rx now is None");
                return None;
            },
        };
        Some((now_bytes - last_bytes) as f64 / interval.as_micros() as f64 * 1e6)
    }
}

impl Interfaces {
    pub fn new(show_lan_down: bool, show_wlan_down: bool, show_vpn_down: bool) -> Self {
        Self { lan: None, wlan: None, vpn: None, last: time::Instant::now() - time::Duration::new(5, 0), show_lan_down, show_wlan_down, show_vpn_down }
    }

    // Return the IP address for the given interface or "no IP" if the
    // interface is up and running but hasn't got an IP address yet
    pub fn update(&mut self) {
        let now = time::Instant::now();
        if now - self.last < time::Duration::new(1, 0) {
            return;
        }

        let interval = now - self.last;
        self.last = now;

        let lan_last = self.lan.take();
        let vpn_last = self.vpn.take();
        let wlan_last = self.wlan.take();

        let mut lan_has_addr = false;
        let mut vpn_has_addr = false;
        let mut wlan_has_addr = false;

        for mut iface in IFAddrs::new() {
            //println!("{:?} ({})", iface, iface.flags & (libc::IFF_RUNNING as u32));
            if iface.flags & (libc::IFF_RUNNING as u32) != 0 {
                if iface.is_lan() {
                    if lan_has_addr && !vpn_has_addr && iface.has_addr {
                        //println!("vpn -> {:?}", iface);
                        iface.calc_speeds(&vpn_last, interval);
                        self.vpn = Some(iface);
                        vpn_has_addr = self.vpn.as_ref().unwrap().has_addr;
                    } else if !lan_has_addr {
                        //println!("lan -> {:?}", iface);
                        iface.calc_speeds(&lan_last, interval);
                        self.lan = Some(iface);
                        lan_has_addr = self.lan.as_ref().unwrap().has_addr;
                    }
                } else if iface.is_vpn() && !vpn_has_addr {
                    //println!("vpn -> {:?}", iface);
                    iface.calc_speeds(&vpn_last, interval);
                    self.vpn = Some(iface);
                    vpn_has_addr = self.vpn.as_ref().unwrap().has_addr;
                } else if iface.is_wlan() && !wlan_has_addr {
                    //println!("wlan -> {:?}", iface);
                    iface.calc_speeds(&wlan_last, interval);
                    self.wlan = Some(iface);
                    wlan_has_addr = self.wlan.as_ref().unwrap().has_addr;
                }
            }
        }
    }

    pub fn status_vpn(&mut self) -> Option<String> {
        self.update();
        match &self.vpn {
            None => {
                if self.show_vpn_down {
                    Some(BAD.to_string() + ICON_VPN + " down")
                } else {
                    None
                }
            },
            Some(ifaddr) => Some((GOOD.to_string() + ICON_VPN + " " + &ifaddr.addr) + &speed_to_str(ifaddr.rx_speed, ICON_DOWN) + &speed_to_str(ifaddr.tx_speed, ICON_UP)),
        }
    }

    pub fn status_wlan(&mut self) -> Option<String> {
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
                        result = result + first + &format!("{:0.0}%", quality);
                        //first = " ";
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
                /*
                match info.bitrate {
                    Some(bitrate) => {
                        result = result + first + &prefixed(bitrate) + "b/s";
                        //first = " ";
                    },
                    None => (),
                };
                */
                Some(result + ")" + &speed_to_str(ifaddr.rx_speed, ICON_DOWN) + &speed_to_str(ifaddr.tx_speed, ICON_UP))
            },
            None => {
                if self.show_wlan_down {
                    Some(BAD.to_string() + ICON_WLAN + " down")
                } else {
                    None
                }
            },
        }
    }

    pub fn status_lan(&mut self) -> Option<String> {
        self.update();
        match &self.lan {
            None => {
                if self.show_lan_down {
                    Some(BAD.to_string() + ICON_LAN + " down")
                } else {
                    None
                }
            },
            Some(ifaddr) => Some(GOOD.to_string() + ICON_LAN + " " + &ifaddr.addr + &speed_to_str(ifaddr.rx_speed, ICON_DOWN) + &speed_to_str(ifaddr.tx_speed, ICON_UP)),
        }
    }
}
