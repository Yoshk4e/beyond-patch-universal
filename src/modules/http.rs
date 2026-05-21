use std::ffi::c_void;

use super::{HgContext, HgModule};
use crate::il2cpp::{self, Il2CppString};
use anyhow::Result;
use ilhook::x64::Registers;

type MakeInitialUrl =
    unsafe extern "C" fn(*mut Il2CppString, *mut Il2CppString) -> *mut Il2CppString;
type UwrInternalSetUrl = unsafe extern "C" fn(*mut c_void, *mut Il2CppString);

static mut MAKE_INITIAL_URL: Option<MakeInitialUrl> = None;
static mut UWR_INTERNAL_SET_URL: Option<UwrInternalSetUrl> = None;

const PORT: u16 = 21000;
const DISPATCH_PORT: u16 = 21041;

pub struct Http;

impl HgModule for HgContext<Http> {
    unsafe fn init(&mut self) -> Result<()> {
        unsafe {
            let domain = il2cpp::domain_get();
            let uwr_img =
                (*(*domain).assembly_open("UnityEngine.UnityWebRequestModule.dll")).get_image();

            let web_request_utils =
                (*uwr_img).class_from_name("UnityEngineInternal", "WebRequestUtils");
            let unity_web_request =
                (*uwr_img).class_from_name("UnityEngine.Networking", "UnityWebRequest");

            let make_initial_url = (*web_request_utils).get_method("MakeInitialUrl", 2);
            let internal_set_url = (*unity_web_request).get_method("InternalSetUrl", 1);
            let set_url = (*unity_web_request).get_method("set_url", 1);

            MAKE_INITIAL_URL = Some(std::mem::transmute::<usize, MakeInitialUrl>(
                (*make_initial_url).address,
            ));
            UWR_INTERNAL_SET_URL = Some(std::mem::transmute::<usize, UwrInternalSetUrl>(
                (*internal_set_url).address,
            ));

            self.interceptor
                .replace((*set_url).address, on_set_url_hook)?;
        }
        Ok(())
    }
}

unsafe extern "win64" fn on_set_url_hook(regs: *mut Registers, _esp: usize, _eip: usize) -> usize {
    unsafe {
        let uwr = (*regs).rcx as *mut c_void;
        let url_string = (*regs).rdx as *mut Il2CppString;
        on_set_url(uwr, url_string);
    }
    0
}

unsafe fn on_set_url(uwr: *mut c_void, url_string: *mut Il2CppString) {
    unsafe {
        let url = (*url_string).to_string();
        println!("[Http]: {url}");

        let Some(stripped) = url.strip_prefix("https://") else {
            UWR_INTERNAL_SET_URL.unwrap_unchecked()(uwr, url_string);
            return;
        };

        // This is so fucking dumb but i'll change it later
        let port = if stripped.contains("/remote_config") || stripped.contains("/get_server_list") {
            DISPATCH_PORT
        } else {
            PORT
        };

        let path = stripped.find('/').map_or("/", |i| &stripped[i..]);
        let replacement = format!("http://127.0.0.1:{port}{path}\0");

        println!("[Http] redirect -> {}", replacement.trim_end_matches('\0'));

        let new_url = il2cpp::string_new(replacement.as_ptr().cast());
        let localhost = il2cpp::string_new(c"http://localhost/".as_ptr());
        let final_url = MAKE_INITIAL_URL.unwrap()(new_url, localhost);

        UWR_INTERNAL_SET_URL.unwrap_unchecked()(uwr, final_url);
    }
}
