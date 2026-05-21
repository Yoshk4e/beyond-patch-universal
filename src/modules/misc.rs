//I'll disable it later on for now that isn't the right thing DO NOT uncomment it will crash your game

/*use super::{HgContext, HgModule};
use crate::il2cpp;
use anyhow::Result;
use ilhook::x64::Registers;

pub struct Misc;

impl HgModule for HgContext<Misc> {
    unsafe fn init(&mut self) -> Result<()> {
        unsafe {
            let domain = il2cpp::domain_get();
            let game_img = (*(*domain).assembly_open("Assembly-CSharp.dll")).get_image();

            let dither_ctrl = (*game_img)
                .class_from_name("Beyond.Gameplay.View", "EntityRenderAlphaDitherController");
            let set_dither_alpha = (*dither_ctrl).get_method("SetDitherAlpha", 1);

            self.interceptor
                .replace((*set_dither_alpha).address, noop)?;
        }
        Ok(())
    }
}

unsafe extern "win64" fn noop(_: *mut Registers, _: usize, _: usize) -> usize {
    0
}*/
