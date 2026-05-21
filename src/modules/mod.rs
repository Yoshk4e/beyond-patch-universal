use crate::interceptor::Interceptor;
use anyhow::Result;
use std::any::{Any, TypeId};

mod ccp_blocker;
mod http;
mod misc;

pub use ccp_blocker::CcpBlocker;
pub use http::Http;

pub struct Module(Box<dyn HgModule + 'static>);

impl Module {
    #[inline]
    pub fn new(mut value: impl HgModule + 'static) -> Self {
        unsafe { value.init().unwrap() };
        Self(Box::new(value))
    }

    #[inline]
    pub fn deinit(&mut self) {
        unsafe { self.0.deinit().unwrap() };
    }

    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        (*self.0).type_id() == TypeId::of::<T>()
    }
}

pub struct ModuleManager {
    modules: Vec<Module>,
}

unsafe impl Sync for ModuleManager {}
unsafe impl Send for ModuleManager {}

impl ModuleManager {
    #[inline]
    pub fn new() -> Self {
        Self {
            modules: Vec::with_capacity(8),
        }
    }

    pub unsafe fn enable(&mut self, module: impl HgModule + 'static) {
        self.modules.push(Module::new(module));
    }

    #[allow(dead_code)]
    pub unsafe fn disable<T: HgModule + 'static>(&mut self) {
        self.modules
            .iter_mut()
            .filter(|x| x.is::<T>())
            .for_each(|x| x.deinit());
    }
}

pub trait HgModule: Any {
    unsafe fn init(&mut self) -> Result<()> {
        Ok(())
    }
    unsafe fn deinit(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct HgContext<T> {
    pub interceptor: Interceptor,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> HgContext<T> {
    pub const fn new() -> Self {
        Self {
            interceptor: Interceptor::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}
