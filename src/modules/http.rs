use std::ffi::CString;
use super::{MhyContext, MhyModule, ModuleType};
use crate::marshal;
use anyhow::Result;
use ilhook::x64::Registers;
use crate::util;
use  tracing::Level;

const ALPHA_WEB_REQUEST_UTILS_MAKE_INITIAL_URL: &str = "48 89 5C 24 ? 48 89 74 24 ? 48 89 4C 24 ? 57 41 56 41 57 48 83 EC ? 48 8B DA 48 8B F9 80 3D ? ? ? ? ? 75";
const BETA_WEB_REQUEST_UTILS_MAKE_INITIAL_URL: &str = "48 89 5C 24 ? 48 89 74 24 ? 48 89 4C 24 ? 57 41 56 41 57 48 81 EC ? ? ? ? 48 8B DA 48 8B F9";
const ALPHA_WEB_REQUEST_UTILS_MAKE_INITIAL_URL_OFFSET: usize = 0;
const BETA_WEB_REQUEST_UTILS_MAKE_INITIAL_URL_OFFSET: usize = 0;

/*const ALPHA_BROWSER_LOAD_URL: &str = "";
const BETA_BROWSER_LOAD_URL: &str = "";
const ALPHA_BROWSER_LOAD_URL_OFFSET: usize = 0;
const BETA_BROWSER_LOAD_URL_OFFSET: usize = 0;*/

pub struct Http;

impl MhyModule for MhyContext<Http> {
    unsafe fn init(&mut self) -> Result<()> {

        let is_beta = self.exe_name == "Endfield_TBeta_OS.exe";
        let sig = if is_beta { BETA_WEB_REQUEST_UTILS_MAKE_INITIAL_URL } else { ALPHA_WEB_REQUEST_UTILS_MAKE_INITIAL_URL };
        let offset = if is_beta { BETA_WEB_REQUEST_UTILS_MAKE_INITIAL_URL_OFFSET } else { ALPHA_WEB_REQUEST_UTILS_MAKE_INITIAL_URL_OFFSET };
        let web_request_utils_make_initial_url = util::pattern_scan_code(self.assembly_name, sig);
        if let Some(addr) = web_request_utils_make_initial_url {
            let target_addr = addr as usize + offset;
            tracing::debug!("web_request_utils_make_initial_url: {:x}", target_addr);
            self.interceptor.attach(
                target_addr,
                on_make_initial_url,
            )?;
        }
        else
        {
            tracing::warn!("Failed to find web_request_utils_make_initial_url");
        }

        /*let is_beta = self.exe_name == "Endfield_TBeta_OS.exe";
        let sig = if is_beta { BETA_BROWSER_LOAD_URL } else { ALPHA_BROWSER_LOAD_URL };
        let offset = if is_beta { BETA_BROWSER_LOAD_URL_OFFSET } else { ALPHA_BROWSER_LOAD_URL_OFFSET };
        let browser_load_url = util::pattern_scan_il2cpp(self.assembly_name, sig);
        if let Some(addr) = browser_load_url {
            let target_addr = addr as usize + offset;
            tracing::debug!("browser_load_url: {:x}", target_addr);
            self.interceptor.attach(
                target_addr,
                on_browser_load_url,
            )?;
        }
        else
        {
            tracing::warn!("Failed to find browser_load_url");
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
        tracing::error!("MakeInitialUrl: rcx is null, skipping");
        return;
    }

    let str_length = *((*reg).rcx.wrapping_add(16) as *const u32);
    let str_ptr = (*reg).rcx.wrapping_add(20) as *const u8;

    if str_length == 0 || str_ptr.is_null() {
        tracing::warn!("MakeInitialUrl: Invalid string length or pointer, skipping");
        return;
    }

    let slice = std::slice::from_raw_parts(str_ptr, (str_length * 2) as usize);
    let url = match String::from_utf16le(slice) {
        Ok(url) => url,
        Err(e) => {
            tracing::warn!("MakeInitialUrl: UTF-16 conversion failed: {:?}", e);
            return;
        }
    };

    tracing::debug!("MakeInitialUrl: Original URL: {}", url);

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

        tracing::debug!("MakeInitialUrl Redirect: {} -> {}", url, new_url);

        match CString::new(new_url.as_str()) {
            Ok(cstring) => (*reg).rcx = marshal::ptr_to_string_ansi(cstring.as_c_str()) as u64,
            Err(e) => tracing::error!("MakeInitialUrl: Failed to create CString: {:?}", e),
        }
    } else {
        tracing::info!("MakeInitialUrl: Skipping redirection");
    }
}

unsafe extern "win64" fn on_browser_load_url(reg: *mut Registers, _: usize) {
    if (*reg).rdx == 0 {
        tracing::error!("Browser::LoadURL: rdx is null, skipping");
        return;
    }

    let str_length = *((*reg).rdx.wrapping_add(16) as *const u32);
    let str_ptr = (*reg).rdx.wrapping_add(20) as *const u8;

    if str_length == 0 || str_ptr.is_null() {
        tracing::error!("Browser::LoadURL: Invalid string length or pointer, skipping");
        return;
    }

    let slice = std::slice::from_raw_parts(str_ptr, (str_length * 2) as usize);
    let url = match String::from_utf16le(slice) {
        Ok(url) => url,
        Err(e) => {
            tracing::warn!("Browser::LoadURL: UTF-16 conversion failed: {:?}", e);
            return;
        }
    };

    tracing::debug!("Browser::LoadURL: Original URL: {}", url);

    let mut new_url = String::from("http://127.0.0.1:21000");
    url.split('/').skip(3).for_each(|s| {
        new_url.push_str("/");
        new_url.push_str(s);
    });

    tracing::debug!("Browser::LoadURL: {} -> {}", url, new_url);

    match CString::new(new_url.as_str()) {
        Ok(cstring) => (*reg).rdx = marshal::ptr_to_string_ansi(cstring.as_c_str()) as u64,
        Err(e) => tracing::error!("Browser::LoadURL: Failed to create CString: {:?}", e),
    }
}