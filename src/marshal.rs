use std::ffi::CStr;
use crate::util;
use windows::core::s;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use tracing::Level;

const PTR_TO_STRING_ANSI: &str = "40 53 48 83 EC ? 80 3D ? ? ? ? ? 48 8B D9 75 ? 48 8D 0D ? ? ? ? E8 ? ? ? ? C6 05 ? ? ? ? ? 48 8B 0D ? ? ? ? 83 B9 ? ? ? ? ? 75 ? E8 ? ? ? ? 48 8B CB E8";
const PTR_TO_STRING_ANSI_OFFSET: usize = 0x140;
type MarshalPtrToStringAnsi = unsafe extern "fastcall" fn(*const u8) -> *const u8;
static mut PTR_TO_STRING_ANSI_ADDR: Option<usize> = None;

pub unsafe fn ptr_to_string_ansi(content: &CStr) -> *const u8 {
    if PTR_TO_STRING_ANSI_ADDR.is_none() {
        find();
    }

    let func = std::mem::transmute::<usize, MarshalPtrToStringAnsi>(PTR_TO_STRING_ANSI_ADDR.unwrap());
    func(content.to_bytes_with_nul().as_ptr())
}

pub unsafe fn find() {
    let ptr_to_string_ansi = util::pattern_scan_code(module(), PTR_TO_STRING_ANSI);
    if let Some(addr) = ptr_to_string_ansi {
        let addr_offset = addr as usize + PTR_TO_STRING_ANSI_OFFSET;
        PTR_TO_STRING_ANSI_ADDR = Some(addr_offset);
        tracing::debug!("ptr_to_string_ansi: {:x}", addr_offset);
    } else {
        tracing::warn!("Failed to find ptr_to_string_ansi");
    }
}

unsafe fn module() -> &'static str {
    if GetModuleHandleA(s!("Endfield_TBeta_OS.exe")).is_ok() {
        "GameAssembly.dll"
    } else if GetModuleHandleA(s!("Endfield_TAlpha.exe")).is_ok() {
        "GameAssembly.dll"
    } else {
        panic!("Neither 'Endfield_TBeta_OS.exe' nor 'Endfield_TAlpha.exe' is loaded");
    }
}