use alsa_sys;
use libc;
use std::{ffi, ptr};
use crate::prelude::*;

static VOL_MUTE: &'static str = "🔇";
static VOL_UNMUTE: &'static str = "🔈";
static VOL_LOW: &'static str = "🔉";
static VOL_HIGH: &'static str = "🔊";

macro_rules! acheck {
    ($f: ident ( $($x: expr),* ) ) => {
        let err = unsafe { alsa_sys::$f( $($x),* ) };
        if err < 0 {
            let err_cstr = unsafe { alsa_sys::snd_strerror(err) };
            let err_str = unsafe { ffi::CStr::from_ptr(err_cstr) }.to_str().unwrap();
            panic!(format!("ALSA: {}: {}", stringify!($f), err_str));
        }
    }
}

struct Mixer {
    handle: *mut alsa_sys::snd_mixer_t
}

impl Mixer {
    fn new() -> Self {
        Mixer {
            handle: {
                let mut _alsa_mxr = ptr::null_mut();
                acheck!(snd_mixer_open(&mut _alsa_mxr, 0));
                _alsa_mxr
            }
        }
    }

    fn attach(&self, device: &str) {
        let device_cstr = ffi::CString::new(device).unwrap();
        acheck!(snd_mixer_attach(self.handle, device_cstr.as_ptr()));
    }

    fn register(&self) {
        acheck!(snd_mixer_selem_register(self.handle, ptr::null_mut(), ptr::null_mut()));
    }

    fn load(&self) {
        acheck!(snd_mixer_load(self.handle));
    }

    fn find(&self, sid: &Selem) -> Elem {
        let alsa_elem = unsafe { alsa_sys::snd_mixer_find_selem(self.handle, sid.handle) };
        if alsa_elem.is_null() {
            panic!(format!("ALSA: Cannot find mixer {} (index {})", sid.get_name(), sid.get_index()));
        }
        Elem { handle: alsa_elem }
    }

    fn handle_events(&self) {
        unsafe { alsa_sys::snd_mixer_handle_events(self.handle); }
    }
}

impl Drop for Mixer {
    fn drop(&mut self) {
        unsafe { alsa_sys::snd_mixer_close(self.handle); }
    }
}

struct Selem {
    handle: *mut alsa_sys::snd_mixer_selem_id_t
}

impl Selem {
    fn new() -> Self {
        let mut selem = ptr::null_mut();
        unsafe { alsa_sys::snd_mixer_selem_id_malloc(&mut selem); }
        if selem.is_null() {
            panic!("failed to allocate snd_mixer_selem_id_t");
        }
        Self { handle: selem }
    }

    fn set_index(&self, idx: libc::c_uint) {
        unsafe { alsa_sys::snd_mixer_selem_id_set_index(self.handle, idx); }
    }

    fn set_name(&self, name: &str) {
        let name_cstr = ffi::CString::new(name).unwrap();
        unsafe { alsa_sys::snd_mixer_selem_id_set_name(self.handle, name_cstr.as_ptr()); }
    }

    fn get_index(&self) -> libc::c_uint {
        unsafe { alsa_sys::snd_mixer_selem_id_get_index(self.handle) }
    }

    fn get_name(&self) -> &str {
        unsafe { ffi::CStr::from_ptr(alsa_sys::snd_mixer_selem_id_get_name(self.handle)).to_str().unwrap() }
    }
}

impl Drop for Selem {
    fn drop(&mut self) {
        unsafe { alsa_sys::snd_mixer_selem_id_free(self.handle); }
    }
}

struct Elem {
    handle: *mut alsa_sys::snd_mixer_elem_t
}

impl Elem {
    fn get_playback_volume_range(&self) -> (libc::c_long, libc::c_long) {
        let mut alsa_min : libc::c_long = 0;
        let mut alsa_max : libc::c_long = 0;
        unsafe { alsa_sys::snd_mixer_selem_get_playback_volume_range(self.handle, &mut alsa_min, &mut alsa_max); }
        (alsa_min, alsa_max)
    }

    fn get_playback_volume(&self) -> libc::c_long {
        let mut val: libc::c_long = 0;
        unsafe { alsa_sys::snd_mixer_selem_get_playback_volume(self.handle, 0, &mut val); }
        val
    }

    fn has_playback_switch(&self) -> bool {
        (unsafe { alsa_sys::snd_mixer_selem_has_playback_switch(self.handle) } != 0)
    }

    fn get_playback_switch(&self) -> bool {
        let mut val: libc::c_int = 0;
        acheck!(snd_mixer_selem_get_playback_switch(self.handle, 0, &mut val));
        (val != 0)
    }
}

impl Drop for Elem {
    fn drop(&mut self) {
        if self.handle.is_null() {
            unsafe { alsa_sys::snd_mixer_elem_free(self.handle); }
        }
    }
}

pub struct Alsa {
    mixer: Mixer,
    elem: Elem,
    min: libc::c_long,
    max: libc::c_long,
}

impl Alsa {
    pub fn new() -> Self {
        // Init mixer
        let mixer = Mixer::new();
        mixer.attach("default");
        mixer.register();
        mixer.load();

        let sid = Selem::new();

        // Find the given mixer
        sid.set_index(0);
        sid.set_name("Master");
        let elem = mixer.find(&sid);
 
        // Get the volume range to convert the volume later
        let (min, max) = elem.get_playback_volume_range();

        Self { mixer: mixer, elem: elem, min: min, max: max }
    }

    pub fn status(&self) -> String {
        let mut result = "".to_string();
        self.mixer.handle_events();

        let val = f64::round((self.elem.get_playback_volume() as f64 - self.min as f64) / (self.max - self.min) as f64 * 100.0) as u8;

        // Check for mute
        if !self.elem.has_playback_switch() || self.elem.get_playback_switch() {
            if val < 10 {
                result += VOL_UNMUTE;
            } else if val > 75 {
                result += VOL_HIGH;
            } else {
                result += VOL_LOW;
            }
        } else {
            result += WARN;
            result += VOL_MUTE;
        }

        result += &format!(" {}%", val);
        result
    }
}
