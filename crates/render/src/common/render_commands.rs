use std::collections::HashMap;

use inox_resources::{Buffer, ResourceId};

use crate::{AsBinding, DrawCommandType, DrawIndexedCommand, GPUMesh, GPUMeshlet, RenderContext};

#[derive(Default)]
pub struct RenderCommandsPerType {
    pub map: HashMap<DrawCommandType, RenderCommands>,
}

#[derive(Default)]
pub struct RenderCommands {
    pub counter: u32,
    pub commands: Buffer<DrawIndexedCommand>,
}

impl RenderCommands {
    pub fn rebind(&mut self, render_context: &RenderContext) {
        self.commands.defrag();
        let count = self.commands.item_count() as _;
        if self.counter != count {
            self.counter = count;
            self.counter.mark_as_dirty(render_context);
        }
    }
    fn bind(&mut self, render_context: &RenderContext, label: Option<&str>) {
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
    fn remove_draw_commands(
        &mut self,
        render_context: &RenderContext,
        id: &ResourceId,
    ) -> &mut Self {
        self.commands.remove(id);
        self.rebind(render_context);
        self
    }

    fn add_draw_commands(
        &mut self,
        render_context: &RenderContext,
        id: &ResourceId,
        commands: &[DrawIndexedCommand],
    ) -> &mut Self {
        if commands.is_empty() {
            if self.counter != 0 {
                self.counter = 0;
                self.counter.mark_as_dirty(render_context);
            }
            return self;
        }
        let is_changed = self.commands.allocate(id, commands).0;
        if is_changed || self.counter != commands.len() as u32 {
            self.counter = commands.len() as _;
            self.counter.mark_as_dirty(render_context);
        }
        self
    }
    fn add_mesh_commands(
        &mut self,
        render_context: &RenderContext,
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
        self.add_draw_commands(render_context, id, commands.as_slice())
    }
}

impl RenderCommandsPerType {
    pub fn bind(&mut self, render_context: &RenderContext, label: Option<&str>) -> &Self {
        if let Some(entry) = self.map.get_mut(&DrawCommandType::PerMeshlet) {
            entry.bind(render_context, label);
        }
        self
    }
    pub fn remove_draw_commands(
        &mut self,
        render_context: &RenderContext,
        id: &ResourceId,
    ) -> &mut Self {
        self.map.iter_mut().for_each(|(_, entry)| {
            entry.remove_draw_commands(render_context, id);
        });
        self
    }
    pub fn add_draw_commands(
        &mut self,
        render_context: &RenderContext,
        id: &ResourceId,
        commands: &[DrawIndexedCommand],
    ) -> &mut Self {
        let entry = self.map.entry(DrawCommandType::PerMeshlet).or_default();
        entry.remove_draw_commands(render_context, id);
        entry.add_draw_commands(render_context, id, commands);
        self
    }
    pub fn add_mesh_commands(
        &mut self,
        render_context: &RenderContext,
        id: &ResourceId,
        mesh: &GPUMesh,
        meshlets: &Buffer<GPUMeshlet>,
    ) -> &mut Self {
        if let Some(meshlets) = meshlets.get(id) {
            let meshlet_entry = self.map.entry(DrawCommandType::PerMeshlet).or_default();
            meshlet_entry.remove_draw_commands(render_context, id);
            meshlet_entry.add_mesh_commands(
                render_context,
                id,
                mesh,
                meshlets,
                DrawCommandType::PerMeshlet,
            );

            let triangle_entry = self.map.entry(DrawCommandType::PerTriangle).or_default();
            triangle_entry.remove_draw_commands(render_context, id);
            triangle_entry.add_mesh_commands(
                render_context,
                id,
                mesh,
                meshlets,
                DrawCommandType::PerTriangle,
            );
        }

        self
    }
}
