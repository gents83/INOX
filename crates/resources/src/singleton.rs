use std::any::Any;

use crate::SharedDataRc;

pub trait Singleton: Any + Send + Sync + 'static {
    #[allow(clippy::mut_from_ref)]
    fn get(shared_data_rc: &SharedDataRc) -> &mut Self
    where
        Self: Sized + Default;
}

#[macro_export]
macro_rules! implement_singleton {
    ($Type:ident) => {
        impl $crate::Singleton for $Type {
            fn get(shared_data_rc: &$crate::SharedDataRc) -> &mut $Type
            where
                Self: Sized + Default,
            {
                shared_data_rc
                    .get_singleton_mut::<$Type>()
                    .unwrap_or_else(|| {
                        shared_data_rc.register_singleton($Type::default());
                        shared_data_rc.get_singleton_mut::<$Type>().unwrap()
                    })
            }
        }
    };
}
