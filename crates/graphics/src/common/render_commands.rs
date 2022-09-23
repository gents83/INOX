use std::collections::HashMap;

use inox_resources::Buffer;

use crate::{AsBinding, DrawCommandType, DrawIndexedCommand, DrawMesh, DrawMeshlet, MeshId};

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

    fn fill_buffer(
        &self,
        render_core_context: &crate::RenderCoreContext,
        buffer: &mut crate::GpuBuffer,
    ) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.count]);
    }
}

#[derive(Default)]
pub struct RenderCommands {
    pub counter: RenderCommandsCount,
    pub commands: Buffer<DrawIndexedCommand>,
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
    fn remove_commands(&mut self, mesh_id: &MeshId) -> &mut Self {
        self.commands.remove(mesh_id);
        self.rebind();
        self
    }
    fn add_commands(
        &mut self,
        mesh_id: &MeshId,
        mesh: &DrawMesh,
        meshlets: &[DrawMeshlet],
        draw_command_type: DrawCommandType,
    ) -> &mut Self {
        let mut commands = Vec::new();
        match draw_command_type {
            DrawCommandType::PerMeshlet => {
                for meshlet_index in
                    mesh.meshlets_offset..mesh.meshlets_offset + mesh.meshlets_count
                {
                    let meshlet = &meshlets[meshlet_index as usize];
                    let command = DrawIndexedCommand {
                        vertex_count: meshlet.indices_count as _,
                        instance_count: 1,
                        base_index: (mesh.indices_offset + meshlet.indices_offset) as _,
                        vertex_offset: mesh.vertex_offset as _,
                        base_instance: meshlet_index as _,
                    };
                    commands.push(command);
                }
            }
            DrawCommandType::PerTriangle => {
                for meshlet_index in
                    mesh.meshlets_offset..mesh.meshlets_offset + mesh.meshlets_count
                {
                    let meshlet = &meshlets[meshlet_index as usize];

                    let total_indices =
                        mesh.indices_offset + meshlet.indices_offset + meshlet.indices_count;
                    debug_assert!(
                        total_indices % 3 == 0,
                        "indices count {} is not divisible by 3",
                        total_indices
                    );
                    let mut i = mesh.indices_offset + meshlet.indices_offset;
                    let mut triangle_index = 0;
                    while i < total_indices {
                        let command = DrawIndexedCommand {
                            vertex_count: 3,
                            instance_count: 1,
                            base_index: i as _,
                            vertex_offset: mesh.vertex_offset as _,
                            base_instance: (triangle_index << 24 | meshlet_index) as _,
                        };
                        commands.push(command);
                        i += 3;
                        triangle_index += 1;
                    }
                }
            }
            _ => {}
        }
        self.commands.allocate(mesh_id, commands.as_slice());
        let count = self.commands.item_count() as _;
        if self.counter.count != count {
            self.counter.count = count;
            self.counter.set_dirty(true);
        }
        self
    }
}

impl RenderCommandsPerType {
    pub fn remove_commands(&mut self, mesh_id: &MeshId) -> &mut Self {
        self.map.iter_mut().for_each(|(_, entry)| {
            entry.remove_commands(mesh_id);
        });
        self
    }
    pub fn add_commands(
        &mut self,
        mesh_id: &MeshId,
        mesh: &DrawMesh,
        meshlets: &Buffer<DrawMeshlet>,
    ) -> &mut Self {
        let meshlets = meshlets.data();

        let meshlet_entry = self.map.entry(DrawCommandType::PerMeshlet).or_default();
        meshlet_entry.remove_commands(mesh_id);
        meshlet_entry.add_commands(mesh_id, mesh, meshlets, DrawCommandType::PerMeshlet);

        let triangle_entry = self.map.entry(DrawCommandType::PerTriangle).or_default();
        triangle_entry.remove_commands(mesh_id);
        triangle_entry.add_commands(mesh_id, mesh, meshlets, DrawCommandType::PerTriangle);

        self
    }
}
