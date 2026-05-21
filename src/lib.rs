#![allow(dead_code, unused_unsafe)]

use std::sync::{LazyLock, RwLock};

use windows::Win32::{
    Foundation::HINSTANCE,
    System::{Console::AllocConsole, SystemServices::DLL_PROCESS_ATTACH},
};

mod il2cpp;
mod interceptor;
mod modules;

use modules::{CcpBlocker, HgContext, Http, ModuleManager};

static MODULE_MANAGER: LazyLock<RwLock<ModuleManager>> =
    LazyLock::new(|| RwLock::new(ModuleManager::new()));

#[macro_export]
macro_rules! wait {
    ($($module:literal),+) => {
        loop {
            if unsafe { true $( && ::windows::Win32::System::LibraryLoader::GetModuleHandleA(s!($module)).is_ok() )+ } {
                break;
            }
            ::std::thread::sleep(::std::time::Duration::from_secs(2));
        }
    };
}

macro_rules! load_module {
    ($module:ty) => {
        unsafe { MODULE_MANAGER.write().unwrap_unchecked().enable(HgContext::<$module>::new()) };
    };
    ($($module:ty),+) => { $(load_module!($module);)+ }
}

unsafe fn on_attach() {
    unsafe { AllocConsole().unwrap_unchecked() };

    std::thread::sleep(std::time::Duration::from_secs(15));
    load_module!(CcpBlocker);
    println!("Endfield patch injected");

    il2cpp::init().expect("Failed to init IL2CPP");

    println!("Initializing modules...");
    load_module!(Http);
    println!("Done.");
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
unsafe extern "system" fn DllMain(_: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(|| unsafe { on_attach() });
    }
    true
}
