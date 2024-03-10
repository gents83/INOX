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
    fn mark_as_dirty(&self, render_context: &RenderContext)
    where
        Self: Sized,
    {
        render_context
            .binding_data_buffer()
            .mark_buffer_as_changed(self.id());
    }
    fn size(&self) -> u64;
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef);
}

impl<T> AsBinding for Buffer<T>
where
    T: Sized + Clone + 'static,
{
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
        }

        impl Default for $VecType {
            fn default() -> Self {
                Self {
                    data: Vec::default(),
                }
            }
        }

        impl $crate::AsBinding for $VecType {
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
            pub fn set(
                &mut self,
                render_context: &$crate::RenderContext,
                data: Vec<$Type>,
            ) -> &mut Self {
                self.data = data;
                self.mark_as_dirty(render_context);
                self
            }
        }
    };
}

#[macro_export]
macro_rules! declare_as_dirty_binding {
    ($Type:ident) => {
        impl $crate::AsBinding for $Type {
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
