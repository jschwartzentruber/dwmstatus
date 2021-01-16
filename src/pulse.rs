use libc::c_void;
use std::{ffi, ptr, time};
use crate::prelude::*;

use libpulse_sys::context::*;
use libpulse_sys::error::pa_strerror;
use libpulse_sys::mainloop::api::pa_mainloop_api;
use libpulse_sys::mainloop::standard::*;
use libpulse_sys::volume::*;

static VOL_MUTE: &'static str = "🔇";
static VOL_UNMUTE: &'static str = "🔈";
static VOL_LOW: &'static str = "🔉";
static VOL_HIGH: &'static str = "🔊";

pub struct Pulse {
    mainloop: *mut pa_mainloop,
    mainloop_api: *const pa_mainloop_api,
    context: *mut pa_context,
    default_sink_name: Option<String>,
    mute: bool,
    level: Option<u8>,
}

extern "C" fn _state_cb(_context: *mut pa_context, data: *mut c_void) {
    let data: &mut Pulse = unsafe { (data as *mut Pulse).as_mut() }.unwrap();
    data.on_state_event()
}

extern "C" fn _server_info_cb(_context: *mut pa_context, i: *const pa_server_info, data: *mut c_void) {
    let data: &mut Pulse = unsafe { &mut *(data as *mut Pulse) };
    if !i.is_null() {
        let i = unsafe { i.as_ref() }.unwrap();
        data.on_server_info_event(i)
    }
}

extern "C" fn _sink_info_cb(_context: *mut pa_context, i: *const pa_sink_info, eol: i32, data: *mut c_void) {
    if eol <= 0 || ! i.is_null() {
        let data: &mut Pulse = unsafe { &mut *(data as *mut Pulse) };
        let i = unsafe { i.as_ref() }.unwrap();
        data.on_sink_info_event(i)
    }
}

extern "C" fn _subscribe_cb(_context: *mut pa_context, t: pa_subscription_event_type_t, idx: u32, data: *mut c_void) {
    let data: &mut Pulse = unsafe { &mut *(data as *mut Pulse) };
    data.on_subscribe_event(t, idx)
}

impl Drop for Pulse {
    fn drop(&mut self) {
        if !self.context.is_null() {
            unsafe { pa_context_unref(self.context) };
        }
        unsafe { pa_mainloop_quit(self.mainloop, 0) };
        unsafe { pa_mainloop_run(self.mainloop, ptr::null_mut()) };
        unsafe { pa_mainloop_free(self.mainloop) };
    }
}

impl Pulse {
    pub fn new() -> Self {
        let mainloop = unsafe { pa_mainloop_new() };
        if mainloop.is_null() {
            panic!("pa_mainloop_new failed");
        }

        let mainloop_api = unsafe { pa_mainloop_get_api(mainloop) };
        if mainloop_api.is_null() {
            panic!("pa_mainloop_get_api failed");
        }

        Self { mainloop, mainloop_api, context: ptr::null_mut(), default_sink_name: None, mute: false, level: None }
    }

    pub fn sleep(&mut self, period: time::Duration) {
        let period = period.as_micros() as i32;
        let ret = unsafe { pa_mainloop_prepare(self.mainloop, period) };
        if ret < 0 {
            let err_str = unsafe { ffi::CStr::from_ptr(pa_strerror(ret)) }.to_str().unwrap();
            panic!("pa_mainloop_prepare failed: {}", err_str);
        }
        let ret = unsafe { pa_mainloop_poll(self.mainloop) };
        if ret < 0 {
            let err_str = unsafe { ffi::CStr::from_ptr(pa_strerror(ret)) }.to_str().unwrap();
            panic!("pa_mainloop_poll failed: {}", err_str);
        }
        let ret = unsafe { pa_mainloop_dispatch(self.mainloop) };
        if ret < 0 {
            let err_str = unsafe { ffi::CStr::from_ptr(pa_strerror(ret)) }.to_str().unwrap();
            panic!("pa_mainloop_dispatch failed: {}", err_str);
        }
    }

    pub fn status(&mut self) -> Option<String> {
        if self.context.is_null() {
            let name_c = ffi::CString::new("dwmstatus").unwrap();
            let context = unsafe { pa_context_new(self.mainloop_api, name_c.as_ptr()) };
            if context.is_null() {
                panic!("pa_context_new failed");
            }

            let ret = unsafe{ pa_context_connect(context, ptr::null_mut(), PA_CONTEXT_NOAUTOSPAWN, ptr::null_mut()) };
            if ret < 0 {
                let err_str = unsafe { ffi::CStr::from_ptr(pa_strerror(ret)) }.to_str().unwrap();
                panic!("pa_context_connect failed: {}", err_str);
            }

            self.context = context;
            unsafe { pa_context_set_state_callback(self.context, Some(_state_cb), self as *mut _ as *mut c_void) };
        }
        loop {
            let ret = unsafe { pa_mainloop_iterate(self.mainloop, 0, ptr::null_mut()) };
            if ret < 0 {
                let err_str = unsafe { ffi::CStr::from_ptr(pa_strerror(ret)) }.to_str().unwrap();
                panic!("pa_mainloop_iterate failed: {}", err_str);
            }
            if ret == 0 {
                break;
            }
        }

        match self.level {
            Some(level) => {
                let mut result = "".to_string();
                // Check for mute
                if !self.mute {
                    if level < 10 {
                        result += VOL_UNMUTE;
                    } else if level > 75 {
                        result += VOL_HIGH;
                    } else {
                        result += VOL_LOW;
                    }
                } else {
                    result += WARN;
                    result += VOL_MUTE;
                }
                result += &format!(" {}%", level);
                Some(result)
            },
            None => None,
        }
    }

    fn on_state_event(&mut self) {
        assert!(!self.context.is_null());
        match unsafe { pa_context_get_state(self.context) } {
            PA_CONTEXT_AUTHORIZING | PA_CONTEXT_CONNECTING | PA_CONTEXT_SETTING_NAME => {},
            PA_CONTEXT_READY => {
                unsafe { pa_context_get_server_info(self.context, Some(_server_info_cb), self as *mut _ as *mut c_void) };
                // Subscribe to sink events from the server. This is how we get
                // volume change notifications from the server.
                unsafe { pa_context_set_subscribe_callback(self.context, Some(_subscribe_cb), self as *mut _ as *mut c_void) };
                unsafe { pa_context_subscribe(self.context, PA_SUBSCRIPTION_MASK_SINK | PA_SUBSCRIPTION_MASK_SERVER, None, self as *mut _ as *mut c_void) };
            },
            PA_CONTEXT_UNCONNECTED => {
                eprintln!("unconnected");
                unsafe { pa_context_unref(self.context) };
                self.context = ptr::null_mut();
            },
            PA_CONTEXT_TERMINATED => {
                eprintln!("terminated");
                unsafe { pa_context_unref(self.context) };
                self.context = ptr::null_mut();
            },
            PA_CONTEXT_FAILED => {
                let errno = unsafe { pa_context_errno(self.context) };
                let err_str = unsafe { ffi::CStr::from_ptr(pa_strerror(errno)) }.to_str().unwrap();
                eprintln!("pa_context_get_state failed: {}", err_str);
                unsafe { pa_context_unref(self.context) };
                self.context = ptr::null_mut();
            },
        };
    }

    fn on_server_info_event(&mut self, i: &pa_server_info) {
        assert!(!self.context.is_null());
        let sink_name = unsafe { ffi::CStr::from_ptr(i.default_sink_name) }.to_str().unwrap();
        self.default_sink_name = Some(sink_name.to_string());
        unsafe { pa_context_get_sink_info_by_name(self.context, i.default_sink_name, Some(_sink_info_cb), self as *mut _ as *mut c_void) };
    }

    fn on_sink_info_event(&mut self, i: &pa_sink_info) {
        if self.default_sink_name.is_some() {
            let sink_name = unsafe { ffi::CStr::from_ptr(i.name) }.to_str().unwrap();
            if self.default_sink_name.as_ref().unwrap() == sink_name {
                let volume = 100.0 * unsafe { pa_cvolume_avg(&i.volume as *const pa_cvolume) } as f64 / PA_VOLUME_NORM as f64;
                self.level = Some(f64::round(volume) as u8);
                self.mute = i.mute != 0;
            }
        }
    }

    fn on_subscribe_event(&mut self, t: pa_subscription_event_type_t, idx: u32) {
        assert!(!self.context.is_null());
        let facility = t & PA_SUBSCRIPTION_EVENT_FACILITY_MASK;

        match facility {
            PA_SUBSCRIPTION_EVENT_SINK => {
                unsafe { pa_context_get_sink_info_by_index(self.context, idx, Some(_sink_info_cb), self as *mut _ as *mut c_void) };
            },
            PA_SUBSCRIPTION_EVENT_SERVER => {
                unsafe { pa_context_get_server_info(self.context, Some(_server_info_cb), self as *mut _ as *mut c_void) };
            },
            _ => {
                panic!("Unexpected event: facility={}", t);
            },
        };
    }
}
