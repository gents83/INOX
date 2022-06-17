use inox_resources::{Buffer, HashBuffer};
use inox_uid::Uid;

use crate::{GpuBuffer, RenderCoreContext};

pub trait AsBinding {
    fn id(&self) -> Uid
    where
        Self: Sized,
    {
        let address = unsafe { *(self as *const Self as *const u128) };
        Uid::from_u128(address)
    }
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, is_dirty: bool);
    fn size(&self) -> u64;
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer);
}

impl<Id, Data, const MAX_COUNT: usize> AsBinding for HashBuffer<Id, Data, MAX_COUNT>
where
    Id: Eq + std::hash::Hash + Copy,
    Data: Default,
{
    fn is_dirty(&self) -> bool {
        self.is_changed()
    }

    fn set_dirty(&mut self, is_dirty: bool) {
        if !is_dirty {
            self.mark_as_unchanged();
        }
    }

    fn size(&self) -> u64 {
        self.buffer_len() as u64 * std::mem::size_of::<Data>() as u64
    }

    fn fill_buffer(
        &self,
        render_core_context: &crate::RenderCoreContext,
        buffer: &mut crate::GpuBuffer,
    ) {
        buffer.add_to_gpu_buffer(render_core_context, self.data());
    }
}

impl<Id, Data, const MAX_COUNT: usize, const ARRAY_SIZE: usize> AsBinding
    for [HashBuffer<Id, Data, MAX_COUNT>; ARRAY_SIZE]
where
    Id: Eq + std::hash::Hash + Copy,
    Data: Default,
{
    fn is_dirty(&self) -> bool {
        self.iter().any(|b| b.is_changed())
    }

    fn set_dirty(&mut self, is_dirty: bool) {
        if !is_dirty {
            self.iter_mut().for_each(|b| b.mark_as_unchanged());
        }
    }

    fn size(&self) -> u64 {
        let mut len = 0;
        self.iter().for_each(|b| {
            len += b.buffer_len() as u64 * std::mem::size_of::<Data>() as u64;
        });
        len
    }

    fn fill_buffer(
        &self,
        render_core_context: &crate::RenderCoreContext,
        buffer: &mut crate::GpuBuffer,
    ) {
        self.iter().for_each(|b| {
            buffer.add_to_gpu_buffer(render_core_context, b.data());
        });
    }
}

impl<T> AsBinding for Buffer<T>
where
    T: Sized + Clone,
{
    fn is_dirty(&self) -> bool {
        self.is_changed()
    }

    fn set_dirty(&mut self, is_dirty: bool) {
        if !is_dirty {
            self.mark_as_unchanged();
        }
    }

    fn size(&self) -> u64 {
        self.total_len() as u64 * std::mem::size_of::<T>() as u64
    }

    fn fill_buffer(
        &self,
        render_core_context: &crate::RenderCoreContext,
        buffer: &mut crate::GpuBuffer,
    ) {
        buffer.add_to_gpu_buffer(render_core_context, self.data());
    }
}

impl<T, const ARRAY_SIZE: usize> AsBinding for [Buffer<T>; ARRAY_SIZE]
where
    T: Sized + Clone,
{
    fn is_dirty(&self) -> bool {
        self.iter().any(|b| b.is_changed())
    }

    fn set_dirty(&mut self, is_dirty: bool) {
        if !is_dirty {
            self.iter_mut().for_each(|b| b.mark_as_unchanged());
        }
    }

    fn size(&self) -> u64 {
        let mut len = 0;
        self.iter().for_each(|b| {
            len += b.total_len() as u64 * std::mem::size_of::<T>() as u64;
        });
        len
    }

    fn fill_buffer(
        &self,
        render_core_context: &crate::RenderCoreContext,
        buffer: &mut crate::GpuBuffer,
    ) {
        self.iter().for_each(|b| {
            buffer.add_to_gpu_buffer(render_core_context, b.data());
        });
    }
}
