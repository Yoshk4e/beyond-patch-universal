use super::{MhyContext, MhyModule, ModuleType};
use anyhow::Result;
use ilhook::x64::Registers;
use crate::util;

pub struct Misc;


const SET_FLOAT_ARRAY_SIG: &str = "48 89 5C 24 ? 48 89 6C 24 ? 48 89 74 24 ? 57 48 83 EC ? 80 3D ? ? ? ? ? 41 8B F9 49 8B D8 8B F2 48 8B E9 75 ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? C6 05 ? ? ? ? ? 48 8B 15 ? ? ? ? 48 8B CB E8 ? ? ? ? 84 C0 0F 84 ? ? ? ? 83 7B ? ? 0F 84 ? ? ? ? 39 7B ? 0F 8C ? ? ? ? 0F 10 03 48 8B 15 ? ? ? ? 48 8D 4C 24 ? 0F 29 44 24 ? E8 ? ? ? ? 48 8B 0D ? ? ? ? 48 8B D8 83 B9 ? ? ? ? ? 75 ? E8 ? ? ? ? 33 D2 48 8B CB E8 ? ? ? ? 48 8B D8 48 8B 05 ? ? ? ? 48 85 C0 75 ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 89 05 ? ? ? ? 4C 8B CB 89 7C 24 ? 45 33 C0";
// Offset from SetFloatArray to SetFloat(int, float)
const SET_FLOAT_OFFSET: usize = 0x200;

impl MhyModule for MhyContext<Misc> {
    unsafe fn init(&mut self) -> Result<()> {
        // Dither
        let set_float_array_addr = util::pattern_scan_code(self.assembly_name, SET_FLOAT_ARRAY_SIG);
        if let Some(addr) = set_float_array_addr {
            let target_addr = addr as usize + SET_FLOAT_OFFSET;
            println!("set_float_target: {:x}", target_addr);
            self.interceptor.replace(
                target_addr,
                set_float_replacement,
            )?;
        } else {
            println!("Failed to find set_float_array");
        }

        Ok(())
    }

    unsafe fn de_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_module_type(&self) -> super::ModuleType {
        ModuleType::Misc
    }
}


unsafe extern "win64" fn set_float_replacement(
    _: *mut Registers,
    _: usize,
    _: usize,
) -> usize {
    0
}