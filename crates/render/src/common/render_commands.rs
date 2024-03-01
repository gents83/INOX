use std::collections::HashMap;

use inox_resources::{Buffer, ResourceId};

use crate::{
    declare_as_binding_vector, AsBinding, DrawCommandType, DrawIndexedCommand, GPUMesh, GPUMeshlet,
    GpuBuffer, RenderContext,
};

declare_as_binding_vector!(VecDrawIndexedCommand, DrawIndexedCommand);

#[derive(Default)]
pub struct RenderCommandsPerType {
    pub map: HashMap<DrawCommandType, RenderCommands>,
}

#[derive(Default)]
pub struct RenderCommandsCount {
    pub count: u32,
    is_dirty: bool,
}

impl AsBinding for RenderCommandsCount {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }

    fn size(&self) -> u64 {
        std::mem::size_of_val(&self.count) as u64
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_context, &[self.count]);
    }
}

#[derive(Default)]
pub struct RenderCommands {
    pub counter: RenderCommandsCount,
    pub commands: Buffer<DrawIndexedCommand, 0>,
}

impl RenderCommands {
    pub fn rebind(&mut self) {
        self.commands.defrag();
        let count = self.commands.item_count() as _;
        if self.counter.count != count {
            self.counter.count = count;
            self.counter.set_dirty(true);
        }
    }
    fn bind(&mut self, render_context: &RenderContext, label: Option<&str>) {
        if self.commands.is_dirty() {
            let usage = wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::INDIRECT;
            render_context.binding_data_buffer().bind_buffer(
                Some(label.unwrap_or("Commands")),
                &mut self.commands,
                usage,
                render_context,
            );
        }
        if self.counter.is_dirty() {
            let usage = wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::INDIRECT;
            render_context.binding_data_buffer().bind_buffer(
                Some("Counter"),
                &mut self.counter,
                usage,
                render_context,
            );
        }
    }
    fn remove_draw_commands(&mut self, id: &ResourceId) -> &mut Self {
        self.commands.remove(id);
        self.rebind();
        self
    }

    fn add_draw_commands(&mut self, id: &ResourceId, commands: &[DrawIndexedCommand]) -> &mut Self {
        if commands.is_empty() {
            return self;
        }
        let is_changed = self.commands.allocate(id, commands).0;
        if is_changed || self.counter.count != commands.len() as u32 {
            self.counter.count = commands.len() as _;
            self.counter.set_dirty(true);
        }
        self
    }
    fn add_mesh_commands(
        &mut self,
        id: &ResourceId,
        mesh: &GPUMesh,
        meshlets: &[GPUMeshlet],
        draw_command_type: DrawCommandType,
    ) -> &mut Self {
        let mut commands = Vec::new();
        match draw_command_type {
            DrawCommandType::PerMeshlet => {
                let mut meshlet_index = mesh.meshlets_offset;
                for meshlet in meshlets {
                    let command = DrawIndexedCommand {
                        vertex_count: meshlet.indices_count as _,
                        instance_count: 1,
                        base_index: meshlet.indices_offset as _,
                        vertex_offset: mesh.vertices_position_offset as _,
                        base_instance: meshlet_index as _,
                    };
                    meshlet_index += 1;
                    commands.push(command);
                }
            }
            DrawCommandType::PerTriangle => {
                let mut meshlet_index = mesh.meshlets_offset;
                for meshlet in meshlets {
                    let total_indices = meshlet.indices_offset + meshlet.indices_count;
                    debug_assert!(
                        meshlet.indices_count % 3 == 0,
                        "indices count {} is not divisible by 3",
                        meshlet.indices_count
                    );
                    let mut i = meshlet.indices_offset;
                    let mut triangle_index = 0;
                    while i < total_indices {
                        let command = DrawIndexedCommand {
                            vertex_count: 3,
                            instance_count: 1,
                            base_index: i as _,
                            vertex_offset: mesh.vertices_position_offset as _,
                            base_instance: (triangle_index << 24 | meshlet_index) as _,
                        };
                        commands.push(command);
                        i += 3;
                        triangle_index += 1;
                    }
                    meshlet_index += 1;
                }
            }
            _ => {}
        }
        self.add_draw_commands(id, commands.as_slice())
    }
}

impl RenderCommandsPerType {
    pub fn bind(&mut self, render_context: &RenderContext, label: Option<&str>) -> &Self {
        if let Some(entry) = self.map.get_mut(&DrawCommandType::PerMeshlet) {
            entry.bind(render_context, label);
        }
        self
    }
    pub fn remove_draw_commands(&mut self, id: &ResourceId) -> &mut Self {
        self.map.iter_mut().for_each(|(_, entry)| {
            entry.remove_draw_commands(id);
        });
        self
    }
    pub fn add_draw_commands(
        &mut self,
        id: &ResourceId,
        commands: &[DrawIndexedCommand],
    ) -> &mut Self {
        let entry = self.map.entry(DrawCommandType::PerMeshlet).or_default();
        entry.remove_draw_commands(id);
        entry.add_draw_commands(id, commands);
        self
    }
    pub fn add_mesh_commands<const MAX_COUNT: usize>(
        &mut self,
        id: &ResourceId,
        mesh: &GPUMesh,
        meshlets: &Buffer<GPUMeshlet, MAX_COUNT>,
    ) -> &mut Self {
        if let Some(meshlets) = meshlets.get(id) {
            let meshlet_entry = self.map.entry(DrawCommandType::PerMeshlet).or_default();
            meshlet_entry.remove_draw_commands(id);
            meshlet_entry.add_mesh_commands(id, mesh, meshlets, DrawCommandType::PerMeshlet);

            let triangle_entry = self.map.entry(DrawCommandType::PerTriangle).or_default();
            triangle_entry.remove_draw_commands(id);
            triangle_entry.add_mesh_commands(id, mesh, meshlets, DrawCommandType::PerTriangle);
        }

        self
    }
}
