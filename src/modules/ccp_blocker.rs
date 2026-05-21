use std::ffi::CStr;

use super::{HgContext, HgModule};
use anyhow::Result;
use ilhook::x64::Registers;
use windows::{
    core::s,
    Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress},
};

pub struct CcpBlocker;

// i was a dumb nigger doing it on "game-config.gryphline.com", "as-stable.gryphline.com", for whatever reason
const BLOCKED: &[&str] = &[
    "event-log-api-ipv6.hypergryph.com",
    "event-log-api-data-lake-prod-cn.hypergryph.com",
    "pc.crashsight.qq.com",
    "crashsight.wetest.net",
];

impl HgModule for HgContext<CcpBlocker> {
    unsafe fn init(&mut self) -> Result<()> {
        unsafe {
            let ws2 = GetModuleHandleA(s!("Ws2_32.dll")).unwrap_unchecked();
            let getaddrinfo = GetProcAddress(ws2, s!("getaddrinfo")).unwrap_unchecked();
            self.interceptor
                .attach(getaddrinfo as usize, on_getaddrinfo)
        }
    }
}

unsafe extern "win64" fn on_getaddrinfo(reg: *mut Registers, _: usize) {
    let host = unsafe { CStr::from_ptr((*reg).rcx as *const i8).to_string_lossy() };
    if BLOCKED.contains(&&*host) {
        unsafe { std::ptr::copy_nonoverlapping(c"0.0.0.0".as_ptr(), (*reg).rcx as *mut i8, 9) };
    }
}
