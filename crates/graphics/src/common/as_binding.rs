use crate::{GpuBuffer, RenderCoreContext};
use inox_resources::{Buffer, HashBuffer};
use inox_uid::{generate_uid_from_string, Uid};

pub type BufferId = Uid;

pub fn generate_buffer_id<T>(v: &T) -> BufferId {
    let address = v as *const T;
    let string = format!("{:p}", address);
    generate_uid_from_string(string.as_str())
}

pub trait AsBinding {
    fn id(&self) -> BufferId
    where
        Self: Sized,
    {
        generate_buffer_id(self)
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
        self.mark_as_changed(is_dirty);
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
        self.iter_mut().for_each(|b| b.mark_as_changed(is_dirty));
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

impl<T> AsBinding for Vec<T>
where
    T: Sized + Clone,
{
    fn is_dirty(&self) -> bool {
        true
    }

    fn set_dirty(&mut self, _is_dirty: bool) {}

    fn size(&self) -> u64 {
        self.len() as u64 * std::mem::size_of::<T>() as u64
    }

    fn fill_buffer(
        &self,
        render_core_context: &crate::RenderCoreContext,
        buffer: &mut crate::GpuBuffer,
    ) {
        buffer.add_to_gpu_buffer(render_core_context, self.as_slice());
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
        self.mark_as_changed(is_dirty);
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
        self.iter_mut().for_each(|b| b.mark_as_changed(is_dirty));
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

impl AsBinding for u32 {
    fn is_dirty(&self) -> bool {
        true
    }

    fn set_dirty(&mut self, _is_dirty: bool) {
        // do nothing
    }

    fn size(&self) -> u64 {
        std::mem::size_of_val(self) as u64
    }

    fn fill_buffer(
        &self,
        render_core_context: &crate::RenderCoreContext,
        buffer: &mut crate::GpuBuffer,
    ) {
        buffer.add_to_gpu_buffer(render_core_context, &[*self]);
    }
}
