use crate::Serializable;


#[macro_export]
macro_rules! impl_boxed_trait {
    ($Type:ty) => {
        /*
        impl $crate::AsSerializable for $Type {
            fn into_type<T>(self: Box<Self>) -> Box<T>
            where
                Box<Self>: Serializable,
                T: Serializable + ?Sized,
            {
                unsafe {
                    let ptr = Box::into_raw(self);
                    Box::from_raw(ptr as _)
                }
            }
        }
        */

        impl $crate::Serializable for Box<$Type> {
            #[inline]
            fn type_name(&self) -> String {
                let str = format!("alloc::boxed::Box<{}>", self.as_ref().type_name().as_str());
                str
            }

            #[inline]
            fn as_serializable(&self) -> &dyn $crate::Serializable {
                self.as_ref().as_serializable()
            }

            #[inline]
            fn as_serializable_mut(&mut self) -> &mut dyn $crate::Serializable {
                self.as_mut().as_serializable_mut()
            }

            #[inline]
            fn any(&self) -> &dyn std::any::Any {
                self.as_ref().any()
            }

            #[inline]
            fn any_mut(&mut self) -> &mut dyn std::any::Any {
                self.as_mut().any_mut()
            }

            #[inline]
            fn set_from(
                &mut self,
                value: &dyn $crate::Serializable,
                registry: &$crate::SerializableRegistry,
            ) {
                self.as_mut().set_from(value, registry)
            }

            #[inline]
            fn serializable_ref(&self) -> $crate::SerializableRef {
                self.as_ref().serializable_ref()
            }

            #[inline]
            fn serializable_mut(&mut self) -> $crate::SerializableMut {
                self.as_mut().serializable_mut()
            }

            #[inline]
            fn duplicate(&self) -> Box<dyn $crate::Serializable> {
                self.as_ref().duplicate()
            }

            #[inline]
            fn compute_hash(&self) -> Option<u64> {
                self.as_ref().compute_hash()
            }

            #[inline]
            fn is_equal(&self, value: &dyn $crate::Serializable) -> Option<bool> {
                self.as_ref().is_equal(value)
            }

            #[inline]
            fn serializable_value(&self) -> Option<$crate::SerializableValue> {
                self.as_ref().serializable_value()
            }
        }

        impl $crate::FromSerializable for Box<$Type> {
            fn from_serializable(
                value: &dyn $crate::Serializable,
                registry: &$crate::SerializableRegistry,
            ) -> Option<Self> {
                Some(registry.create_value_from_trait::<$Type>(value))
            }
        }
    };
}

impl_boxed_trait!(dyn Serializable);
