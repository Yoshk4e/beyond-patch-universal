#![feature(str_from_utf16_endian)]

use std::{sync::RwLock, thread::sleep, time::Duration};
use lazy_static::lazy_static;
use tracing::Level;
use windows::Win32::{
    Foundation::HINSTANCE,
    System::{
        Console::{AllocConsole, GetConsoleMode, SetConsoleMode, CONSOLE_MODE, ENABLE_VIRTUAL_TERMINAL_PROCESSING},
        LibraryLoader::GetModuleFileNameA,
        SystemServices::DLL_PROCESS_ATTACH,
    },
};
use std::ffi::CStr;
use std::path::Path;
use windows::Win32::System::Console;

mod interceptor;
mod marshal;
mod modules;
mod util;

use crate::modules::{CcpBlocker, Http, MhyContext, Misc, ModuleManager};

unsafe fn init_tracing(level: Level) {
    // Enable ANSI escape code support in the Windows console
    let mut console_mode = CONSOLE_MODE(0);
    let stdout = windows::Win32::System::Console::GetStdHandle(windows::Win32::System::Console::STD_OUTPUT_HANDLE).unwrap();
    GetConsoleMode(stdout, &mut console_mode).unwrap();
    SetConsoleMode(stdout, console_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING).unwrap();

    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();

    tracing::info!("Endfield Redirection Patch");
    tracing::info!("Made by: xeondev (base), oureveryday (pattern scan idea)");
}

unsafe fn thread_func() {
    Console::AllocConsole().unwrap();
    init_tracing(tracing::Level::DEBUG);

    let mut module_manager = MODULE_MANAGER.write().unwrap();

    util::disable_memprotect_guard();

    sleep(Duration::from_secs(10));

    let mut buffer = [0u8; 260];
    GetModuleFileNameA(None, &mut buffer);
    let exe_path = CStr::from_ptr(buffer.as_ptr() as *const i8).to_str().unwrap();
    let exe_name = Path::new(exe_path).file_name().unwrap().to_str().unwrap();
    let dll = "GameAssembly.dll";
    tracing::info!("Current executable name: {}", exe_name);

    if exe_name != "Endfield_TBeta_OS.exe" && exe_name != "Endfield_TAlpha.exe" {
        tracing::error!("Executable is not Endfield. Skipping initialization.");
        return;
    }

    tracing::debug!("Initializing modules...");
    module_manager.enable(MhyContext::<CcpBlocker>::new("", exe_name.to_string()));
    marshal::find();
    module_manager.enable(MhyContext::<Http>::new(&dll, exe_name.to_string()));
    module_manager.enable(MhyContext::<Misc>::new(&dll, exe_name.to_string()));

    tracing::debug!("Successfully initialized!");
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