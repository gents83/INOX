use vulkan_bindings::*;

use super::BackendDevice;

#[derive(Clone)]
pub struct BackendCommandBuffer {
    command_buffer: VkCommandBuffer,
}

impl BackendCommandBuffer {
    pub fn create(device: &mut BackendDevice) -> Self {
        Self {
            command_buffer: { device.acquire_command_buffer() },
        }
    }

    pub fn execute(&self, device: &BackendDevice) {
        unsafe {
            // Execute render commands from the secondary command buffer
            vkCmdExecuteCommands.unwrap()(
                device.get_primary_command_buffer(),
                1,
                &self.command_buffer,
            );
        }
    }

    pub fn get(&self) -> VkCommandBuffer {
        self.command_buffer
    }
}
