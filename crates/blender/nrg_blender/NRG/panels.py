import bpy

blender_classes = []


class NRGRunner(bpy.types.Panel):
    """NRG Runner"""
    bl_label = "NRG"
    bl_idname = "NRG_PT_runner"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "render"

    def draw(self, context):
        layout = self.layout

        row = layout.row()
        row.operator("nrg.run", icon='PLAY')


blender_classes.append(NRGRunner)


def register():
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)


def unregister():
    blender_classes.reverse()
    for blender_class in blender_classes:
        bpy.utils.unregister_class(blender_class)
