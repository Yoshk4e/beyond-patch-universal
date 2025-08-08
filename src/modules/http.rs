use std::ffi::CString;

use super::{MhyContext, MhyModule, ModuleType};
use crate::marshal;
use anyhow::Result;
use ilhook::x64::Registers;
use crate::util;

const WEB_REQUEST_UTILS_MAKE_INITIAL_URL: &str = "48 89 5C 24 ? 48 89 74 24 ? 48 89 4C 24 ? 57 41 56 41 57 48 83 EC ? 48 8B DA 48 8B F9 80 3D ? ? ? ? ? 75";
/*const BROWSER_LOAD_URL: &str = "41 B0 01 E9 08 00 00 00 0F 1F 84 00 00 00 00 00 56 57";
const BROWSER_LOAD_URL_OFFSET: usize = 0x10;*/

pub struct Http;

impl MhyModule for MhyContext<Http> {
    unsafe fn init(&mut self) -> Result<()> {

        let web_request_utils_make_initial_url = util::pattern_scan_code(self.assembly_name, WEB_REQUEST_UTILS_MAKE_INITIAL_URL);
        if let Some(addr) = web_request_utils_make_initial_url {
            println!("web_request_utils_make_initial_url: {:x}", addr as usize);
            self.interceptor.attach(
                addr as usize,
                on_make_initial_url,
            )?;
        }
        else
        {
            println!("Failed to find web_request_utils_make_initial_url");
        }

        /*let browser_load_url = util::pattern_scan_il2cpp(self.assembly_name, BROWSER_LOAD_URL);
        if let Some(addr) = browser_load_url {
            let addr_offset = addr as usize + BROWSER_LOAD_URL_OFFSET;
            println!("browser_load_url: {:x}", addr_offset);
            self.interceptor.attach(
                addr_offset,
                on_browser_load_url,
            )?;
        }*/
        /*else
        {
            println!("Failed to find browser_load_url");
        }*/

        Ok(())
    }

    unsafe fn de_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_module_type(&self) -> super::ModuleType {
        ModuleType::Http
    }
}

unsafe extern "win64" fn on_make_initial_url(reg: *mut Registers, _: usize) {
    if (*reg).rcx == 0 {
        println!("MakeInitialUrl: rcx is null, skipping");
        return;
    }

    let str_length = *((*reg).rcx.wrapping_add(16) as *const u32);
    let str_ptr = (*reg).rcx.wrapping_add(20) as *const u8;

    if str_length == 0 || str_ptr.is_null() {
        println!("MakeInitialUrl: Invalid string length or pointer, skipping");
        return;
    }

    let slice = std::slice::from_raw_parts(str_ptr, (str_length * 2) as usize);
    let url = match String::from_utf16le(slice) {
        Ok(url) => url,
        Err(e) => {
            println!("MakeInitialUrl: UTF-16 conversion failed: {:?}", e);
            return;
        }
    };

    println!("MakeInitialUrl: Original URL: {}", url);

    if !url.contains("/token_by_channel_token") && !url.contains("platform=Windows") && !url.contains("res_version") && !url.contains("asset") && !url.contains("StreamingAssets") {
        let mut new_url = if url.contains("/remote_config") || url.contains("/get_server_list") {
            String::from("http://127.0.0.1:21041")
        } else {
            String::from("http://127.0.0.1:21000")
        };

        url.split('/').skip(3).for_each(|s| {
            new_url.push_str("/");
            new_url.push_str(s);
        });

        println!("MakeInitialUrl Redirect: {} -> {}", url, new_url);

        match CString::new(new_url.as_str()) {
            Ok(cstring) => (*reg).rcx = marshal::ptr_to_string_ansi(cstring.as_c_str()) as u64,
            Err(e) => println!("MakeInitialUrl: Failed to create CString: {:?}", e),
        }
    } else {
        println!("MakeInitialUrl: Skipping redirection");
    }
}

unsafe extern "win64" fn on_browser_load_url(reg: *mut Registers, _: usize) {
    if (*reg).rdx == 0 {
        println!("Browser::LoadURL: rdx is null, skipping");
        return;
    }

    let str_length = *((*reg).rdx.wrapping_add(16) as *const u32);
    let str_ptr = (*reg).rdx.wrapping_add(20) as *const u8;

    if str_length == 0 || str_ptr.is_null() {
        println!("Browser::LoadURL: Invalid string length or pointer, skipping");
        return;
    }

    let slice = std::slice::from_raw_parts(str_ptr, (str_length * 2) as usize);
    let url = match String::from_utf16le(slice) {
        Ok(url) => url,
        Err(e) => {
            println!("Browser::LoadURL: UTF-16 conversion failed: {:?}", e);
            return;
        }
    };

    println!("Browser::LoadURL: Original URL: {}", url);

    let mut new_url = String::from("http://127.0.0.1:21000");
    url.split('/').skip(3).for_each(|s| {
        new_url.push_str("/");
        new_url.push_str(s);
    });

    println!("Browser::LoadURL: {} -> {}", url, new_url);

    match CString::new(new_url.as_str()) {
        Ok(cstring) => (*reg).rdx = marshal::ptr_to_string_ansi(cstring.as_c_str()) as u64,
        Err(e) => println!("Browser::LoadURL: Failed to create CString: {:?}", e),
    }
}