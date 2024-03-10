use crate::{BufferRef, RenderContext};
use inox_resources::Buffer;
pub type BufferId = u64;

#[inline]
pub fn generate_id_from_address<T>(v: &T) -> BufferId {
    (v as *const T) as _
}

pub trait AsBinding {
    fn id(&self) -> BufferId
    where
        Self: Sized,
    {
        generate_id_from_address(self)
    }
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, is_dirty: bool);
    fn size(&self) -> u64;
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef);
}

impl<T> AsBinding for Buffer<T>
where
    T: Sized + Clone + 'static,
{
    fn is_dirty(&self) -> bool {
        self.is_changed()
    }

    fn set_dirty(&mut self, is_dirty: bool) {
        self.mark_as_changed(is_dirty);
    }

    fn size(&self) -> u64 {
        self.total_len() as u64 * std::mem::size_of::<T>() as u64
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut crate::BufferRef) {
        buffer.add_to_gpu_buffer(render_context, self.data());
    }
}

impl<T, const ARRAY_SIZE: usize> AsBinding for [Buffer<T>; ARRAY_SIZE]
where
    T: Sized + Clone + 'static,
{
    fn is_dirty(&self) -> bool {
        self.iter().any(|b| b.is_changed())
    }

    fn set_dirty(&mut self, is_dirty: bool) {
        self.iter_mut().for_each(|b| b.mark_as_changed(is_dirty));
    }

    fn size(&self) -> u64 {
        let mut len = 0;
        self.iter().for_each(|b| {
            len += b.total_len() as u64 * std::mem::size_of::<T>() as u64;
        });
        len
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut crate::BufferRef) {
        self.iter().for_each(|b| {
            buffer.add_to_gpu_buffer(render_context, b.data());
        });
    }
}

impl<T> AsBinding for Vec<T> {
    fn is_dirty(&self) -> bool {
        false
    }

    fn set_dirty(&mut self, _is_dirty: bool) {}

    fn size(&self) -> u64 {
        self.len() as u64 * std::mem::size_of::<T>() as u64
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef) {
        buffer.add_to_gpu_buffer(render_context, self.as_slice());
    }
}

#[macro_export]
macro_rules! declare_as_binding_vector {
    ($VecType:ident, $Type:ident) => {
        pub struct $VecType {
            data: Vec<$Type>,
            is_dirty: bool,
        }

        impl Default for $VecType {
            fn default() -> Self {
                Self {
                    data: Vec::default(),
                    is_dirty: true,
                }
            }
        }

        impl $crate::AsBinding for $VecType {
            fn is_dirty(&self) -> bool {
                self.is_dirty
            }

            fn set_dirty(&mut self, is_dirty: bool) {
                self.is_dirty = is_dirty;
            }

            fn size(&self) -> u64 {
                self.data.len() as u64 * std::mem::size_of::<$Type>() as u64
            }

            fn fill_buffer(
                &self,
                render_context: &$crate::RenderContext,
                buffer: &mut $crate::BufferRef,
            ) {
                buffer.add_to_gpu_buffer(render_context, self.data.as_slice());
            }
        }

        #[allow(dead_code)]
        impl $VecType {
            pub fn data(&self) -> &[$Type] {
                &self.data
            }
            pub fn data_mut(&mut self) -> &mut Vec<$Type> {
                &mut self.data
            }
            pub fn set(&mut self, data: Vec<$Type>) -> &mut Self {
                use $crate::AsBinding;

                self.data = data;
                self.set_dirty(true);
                self
            }
        }
    };
}

#[macro_export]
macro_rules! declare_as_dirty_binding {
    ($Type:ident) => {
        impl $crate::AsBinding for $Type {
            fn is_dirty(&self) -> bool {
                true
            }

            fn set_dirty(&mut self, _is_dirty: bool) {}

            fn size(&self) -> u64 {
                std::mem::size_of::<$Type>() as u64
            }

            fn fill_buffer(
                &self,
                render_context: &$crate::RenderContext,
                buffer: &mut $crate::BufferRef,
            ) {
                buffer.add_to_gpu_buffer(render_context, &[*self]);
            }
        }
    };
}
