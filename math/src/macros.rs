#[macro_export]
macro_rules! implement_method_with_return_value {
    ($( $method:ident = $ret:expr, )*)
        => {$(
            #[inline]
            fn $method() -> Self {
                $ret
            }
        )*};
}

#[macro_export]
macro_rules! implement_method_with_base {
    ($( Self :: $method:ident ( self $( , $arg:ident : $ty:ty )* ) -> $ret:ty ; )*)
        => {$(
            #[inline]
            fn $method(self $( , $arg : $ty )* ) -> $ret {
                Self::$method(self $( , $arg )* )
            }
        )*};
    ($( $base:ident :: $method:ident ( self $( , $arg:ident : $ty:ty )* ) -> $ret:ty ; )*)
        => {$(
            #[inline]
            fn $method(self $( , $arg : $ty )* ) -> $ret {
                <Self as $base>::$method(self $( , $arg )* )
            }
        )*};
    ($( $base:ident :: $method:ident ( $( $arg:ident : $ty:ty ),* ) -> $ret:ty ; )*)
        => {$(
            #[inline]
            fn $method( $( $arg : $ty ),* ) -> $ret {
                <Self as $base>::$method( $( $arg ),* )
            }
        )*}
}