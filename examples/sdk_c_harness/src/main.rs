use libc::c_char;
use std::ffi::CStr;

#[link(name = "astraweave_sdk", kind = "static")]
extern "C" {
    fn aw_version() -> Version;
    fn aw_version_string(buf: *mut u8, len: usize) -> usize;
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

fn main() {
    unsafe {
        let v = aw_version();
        println!("SDK version: {}.{}.{}", v.major, v.minor, v.patch);
        let mut buf = vec![0u8; 32];
        let need = aw_version_string(buf.as_mut_ptr(), buf.len());
        println!("version string need={} bytes", need);
        let cstr = CStr::from_ptr(buf.as_ptr() as *const c_char);
        println!("version string: {}", cstr.to_string_lossy());
    }
}
