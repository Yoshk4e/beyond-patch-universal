// Edited main.rs
#![feature(str_from_utf16_endian)]

use std::{sync::RwLock};

use lazy_static::lazy_static;
use modules::{CcpBlocker, Misc};
use windows::Win32::System::Console;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::{Foundation::HINSTANCE, System::LibraryLoader::GetModuleFileNameA};
use std::ffi::CStr;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

mod interceptor;
mod marshal;
mod modules;
mod util;

use crate::modules::{Http, MhyContext, ModuleManager};

unsafe fn thread_func() {
    Console::AllocConsole().unwrap();
    let mut module_manager = MODULE_MANAGER.write().unwrap();

    util::disable_memprotect_guard();

    println!("Endfield redirection patch\nMade by xeondev(base) & oureveryday(pattern scan idea)\n");

    sleep(Duration::from_secs(10));
    let mut buffer = [0u8; 260];
    GetModuleFileNameA(None, &mut buffer);
    let exe_path = CStr::from_ptr(buffer.as_ptr() as *const i8).to_str().unwrap();
    let exe_name = Path::new(exe_path).file_name().unwrap().to_str().unwrap();
    let dll = "GameAssembly.dll";
    println!("Current executable name: {}", exe_name);

    if exe_name != "Endfield_TBeta_OS.exe" && exe_name != "Endfield_TAlpha.exe" {
        println!("Executable is not endfield. Skipping initialization.");
        return;
    }

    module_manager.enable(MhyContext::<CcpBlocker>::new("", exe_name.to_string()));

    println!("Initializing modules...");

    /*module_manager.enable(MhyContext::<Security>::new(exe_name.to_string()));*/
    marshal::find();
    module_manager.enable(MhyContext::<Http>::new(&dll, exe_name.to_string()));
    module_manager.enable(MhyContext::<Misc>::new(&dll, exe_name.to_string()));

    println!("Successfully initialized!");
}

lazy_static! {
    static ref MODULE_MANAGER: RwLock<ModuleManager> = RwLock::new(ModuleManager::default());
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "system" fn DllMain(_: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(|| {
            thread_func();
        });
    }
    true
}