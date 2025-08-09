// Edited misc.rs
use super::{MhyContext, MhyModule, ModuleType};
use anyhow::Result;
use ilhook::x64::Registers;
use crate::util;

pub struct Misc;

const ALPHA_SET_FLOAT_ARRAY_SIG: &str = "48 89 5C 24 ? 48 89 6C 24 ? 48 89 74 24 ? 57 48 83 EC ? 80 3D ? ? ? ? ? 41 8B F9 49 8B D8 8B F2 48 8B E9 75 ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? C6 05 ? ? ? ? ? 48 8B 15 ? ? ? ? 48 8B CB E8 ? ? ? ? 84 C0 0F 84 ? ? ? ? 83 7B ? ? 0F 84 ? ? ? ? 39 7B ? 0F 8C ? ? ? ? 0F 10 03 48 8B 15 ? ? ? ? 48 8D 4C 24 ? 0F 29 44 24 ? E8 ? ? ? ? 48 8B 0D ? ? ? ? 48 8B D8 83 B9 ? ? ? ? ? 75 ? E8 ? ? ? ? 33 D2 48 8B CB E8 ? ? ? ? 48 8B D8 48 8B 05 ? ? ? ? 48 85 C0 75 ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 89 05 ? ? ? ? 4C 8B CB 89 7C 24 ? 45 33 C0";
const BETA_GET_VECTOR_IMPL_SIG: &str = "48 89 5C 24 08 48 89 74 24 10 57 48 83 EC 20 48 8B 05 4A 9E FD 02 0F 57 C0 41 8B F8 48 8B F2 48 8B D9 0F 11 01 48 85 C0 75 18 48 8D 0D 1F 24 70 00 E8 3A 31 36 F4 48 85 C0 74 24 48 89 05 1E 9E FD 02 4C 8B C3 8B D7 48 8B CE FF D0 48 8B 74 24 38 48 8B C3 48 8B 5C 24 30 48 83 C4 20 5F C3 48 8D 0D EA 23 70 00 E8 75 2D 36 F4 48 8B C8 33 D2 E8 CB 0F 36 F4 CC";
const ALPHA_SET_FLOAT_OFFSET: usize = 0x200;
const BETA_SET_FLOAT_OFFSET: usize = 0x80;

impl MhyModule for MhyContext<Misc> {
    unsafe fn init(&mut self) -> Result<()> {
        // Dither
        let is_beta = self.exe_name == "Endfield_TBeta_OS.exe";
        let sig = if is_beta { BETA_GET_VECTOR_IMPL_SIG } else { ALPHA_SET_FLOAT_ARRAY_SIG };
        let offset = if is_beta { BETA_SET_FLOAT_OFFSET } else { ALPHA_SET_FLOAT_OFFSET };
        let set_float_array_addr = util::pattern_scan_code(self.assembly_name, sig);
        if let Some(addr) = set_float_array_addr {
            let target_addr = addr as usize + offset;
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