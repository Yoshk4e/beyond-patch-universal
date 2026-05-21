use std::ffi::c_void;
use std::sync::OnceLock;
use std::{fmt, ptr};
use windows::core::{s, PCSTR};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

static IL2CPP: OnceLock<Il2cpp> = OnceLock::new();

pub fn init() -> Result<(), &'static str> {
    let il2cpp = unsafe { Il2cpp::load() };
    IL2CPP.set(il2cpp).map_err(|_| "IL2CPP already initialized")
}

pub trait AsCStr {
    fn as_cstr(&self) -> *const i8;
}

impl AsCStr for str {
    fn as_cstr(&self) -> *const i8 {
        let bytes = format!("{self}\0");
        let ptr = bytes.as_ptr().cast();
        std::mem::forget(bytes);
        ptr
    }
}

impl AsCStr for &str {
    fn as_cstr(&self) -> *const i8 {
        (*self).as_cstr()
    }
}

macro_rules! il2cpp_struct {
    ($(fn $name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        pub struct Il2cpp {
            $(pub $name: unsafe extern "C" fn($($ty),*) -> $ret,)*
        }

        impl Il2cpp {
            pub unsafe fn load() -> Self {
                let module = unsafe { GetModuleHandleA(s!("GameAssembly.dll")).unwrap() };
                println!("GameAssembly: {:#x}", module.0 as usize);
                Self {
                    $($name: {
                        let sym = concat!("il2cpp_", stringify!($name), "\0");
                        let proc = unsafe { GetProcAddress(module, PCSTR(sym.as_ptr())) }
                            .expect(concat!("export not found: il2cpp_", stringify!($name)));
                        unsafe { std::mem::transmute::<unsafe extern "system" fn() -> isize,
                                                        unsafe extern "C" fn($($ty),*) -> $ret>(proc) }
                    },)*
                }
            }
        }

        $(pub unsafe fn $name($($arg: $ty),*) -> $ret {
            unsafe { (crate::il2cpp::IL2CPP.get().unwrap().$name)($($arg),*) }
        })*
    };
}

il2cpp_struct! {
    fn domain_get() -> *mut Il2CppDomain;
    fn domain_assembly_open(domain: *mut Il2CppDomain, name: *const i8) -> *const Il2CppAssembly;
    fn assembly_get_image(assembly: *const Il2CppAssembly) -> *const Il2CppImage;
    fn class_from_name(image: *const Il2CppImage, namespace: *const i8, name: *const i8) -> *mut Il2CppClass;
    fn class_get_method_from_name(klass: *mut Il2CppClass, name: *const i8, args_count: i32) -> *mut MethodInfo;
    fn class_get_field_from_name(klass: *mut Il2CppClass, name: *const i8) -> *mut Il2CppField;
    fn field_get_offset(field: *mut Il2CppField) -> usize;
    fn string_new(str: *const i8) -> *mut Il2CppString;
    fn object_new(klass: *mut Il2CppClass) -> *mut c_void;
}

#[repr(C)]
pub struct Il2CppString {
    pub header: u128,
    pub length: u32,
    pub first_char: u16,
}

impl fmt::Display for Il2CppString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chars = unsafe {
            std::slice::from_raw_parts(
                (ptr::from_ref(&self.first_char)).cast::<u16>(),
                self.length as usize,
            )
        };
        write!(f, "{}", String::from_utf16_lossy(chars))
    }
}

#[repr(C)]
pub struct Il2CppDomain;
#[repr(C)]
pub struct Il2CppAssembly;
#[repr(C)]
pub struct Il2CppImage;
#[repr(C)]
pub struct Il2CppClass;
#[repr(C)]
pub struct Il2CppField;

#[repr(C)]
pub struct MethodInfo {
    pub address: usize,
}

impl Il2CppDomain {
    pub unsafe fn assembly_open(&self, name: &str) -> *const Il2CppAssembly {
        unsafe {
            domain_assembly_open(
                ptr::from_ref::<Il2CppDomain>(self).cast_mut(),
                name.as_cstr(),
            )
        }
    }
}

impl Il2CppAssembly {
    pub unsafe fn get_image(&self) -> *const Il2CppImage {
        unsafe { assembly_get_image(ptr::from_ref(self)) }
    }
}

impl Il2CppImage {
    pub unsafe fn class_from_name(&self, namespace: &str, name: &str) -> *mut Il2CppClass {
        unsafe { class_from_name(ptr::from_ref(self), namespace.as_cstr(), name.as_cstr()) }
    }
}

impl Il2CppClass {
    pub unsafe fn get_method(&self, name: &str, args: i32) -> *mut MethodInfo {
        unsafe {
            class_get_method_from_name(
                ptr::from_ref::<Il2CppClass>(self).cast_mut(),
                name.as_cstr(),
                args,
            )
        }
    }

    pub unsafe fn get_field(&self, name: &str) -> *mut Il2CppField {
        unsafe {
            class_get_field_from_name(
                ptr::from_ref::<Il2CppClass>(self).cast_mut(),
                name.as_cstr(),
            )
        }
    }
}

impl Il2CppField {
    pub unsafe fn get_offset(&self) -> usize {
        unsafe { field_get_offset(ptr::from_ref::<Il2CppField>(self).cast_mut()) }
    }
}
