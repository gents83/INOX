use inox_resources::Buffer;

use crate::{BufferRef, RenderContext};
pub type BufferId = u64;

#[inline]
pub fn generate_id_from_address<T>(v: &T) -> BufferId {
    (v as *const T) as _
}

pub trait AsBinding {
    fn buffer_id(&self) -> BufferId
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
            .mark_buffer_as_changed(self.buffer_id());
    }
    fn count(&self) -> usize;
    fn size(&self) -> u64;
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef);
}

#[macro_export]
macro_rules! declare_as_binding {
    ($Type:ident) => {
        impl $crate::AsBinding for $Type {
            fn size(&self) -> u64 {
                std::mem::size_of::<$Type>() as u64
            }
            fn count(&self) -> usize {
                1
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

impl<T> AsBinding for Buffer<T>
where
    T: Sized + Clone + 'static,
{
    fn count(&self) -> usize {
        self.total_len()
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
    fn count(&self) -> usize {
        ARRAY_SIZE
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

//Please note that first usize is the length of the vector - then vector is offsetted
impl<T> AsBinding for Vec<T> {
    fn count(&self) -> usize {
        self.len()
    }
    fn size(&self) -> u64 {
        self.len() as u64 * std::mem::size_of::<T>() as u64
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef) {
        buffer.add_to_gpu_buffer(render_context, self.as_slice());
    }
}

//Please note that first usize is the length of the vector - then vector is offsetted
impl<T> AsBinding for [T]
where
    T: Sized + Clone + 'static,
{
    fn count(&self) -> usize {
        self.len()
    }
    fn size(&self) -> u64 {
        self.len() as u64 * std::mem::size_of::<T>() as u64
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut crate::BufferRef) {
        buffer.add_to_gpu_buffer(render_context, self);
    }
}

declare_as_binding!(i32);
declare_as_binding!(u32);
declare_as_binding!(usize);
declare_as_binding!(u64);
declare_as_binding!(u128);
declare_as_binding!(f32);
declare_as_binding!(f64);
