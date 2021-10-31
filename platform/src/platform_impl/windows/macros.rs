#[macro_export]
macro_rules! declare_handle {
    ($name:ident, $inner:ident) => {
        #[derive(Clone, Copy)]
        pub enum $inner {}
        pub type $name = *mut $inner;

        unsafe impl Send for $inner {}
        unsafe impl Sync for $inner {}
    };
}

#[macro_export]
macro_rules! declare_extern_function {
    (stdcall $func:ident($($t:ty,)*) -> $ret:ty) => (
        pub type $func = Option<unsafe extern "system" fn($($t,)*) -> $ret>;
    );
    (stdcall $func:ident($($p:ident: $t:ty,)*) -> $ret:ty) => (
        pub type $func = Option<unsafe extern "system" fn($($p: $t,)*) -> $ret>;
    );
    (cdecl $func:ident($($t:ty,)*) -> $ret:ty) => (
        pub type $func = Option<unsafe extern "C" fn($($t,)*) -> $ret>;
    );
    (cdecl $func:ident($($p:ident: $t:ty,)*) -> $ret:ty) => (
        pub type $func = Option<unsafe extern "C" fn($($p: $t,)*) -> $ret>;
    );
}
