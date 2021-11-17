import bpy
from . import node_tree

blender_classes = []


class SABIRunner(bpy.types.Panel):
    """SABI Runner"""
    bl_label = "SABI"
    bl_idname = "SABI_PT_runner"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "render"

    def draw(self, context):
        layout = self.layout

        row = layout.row()
        row.operator("sabi.run", icon='PLAY')


class SABIPropertiesGroup(bpy.types.PropertyGroup):
    def filter_on_custom_property(self, node_group):
        return node_group.bl_idname == 'LogicNodeTree'

    logic: bpy.props.PointerProperty(name="Logic",
                                     description="Logic node tree",
                                     type=node_tree.LogicNodeTree,
                                     poll=filter_on_custom_property)

    def draw(self, layout, context):
        split = layout.split(factor=0.9)
        col_1 = split.column()
        col_2 = split.column()
        col_1.prop_search(self, "logic", bpy.data, "node_groups")
        col_2.operator("sabi.open_in_logic_editor", icon='FILE_FOLDER')


class SABIProperties(bpy.types.Panel):
    """SABI related Object Data"""
    bl_label = "SABI Properties"
    bl_idname = "SABI_PT_object_data"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "object"

    def draw(self, context):
        layout = self.layout

        obj = context.object
        row = layout.row()
        obj.sabi_properties.draw(layout, context)


blender_classes.append(SABIRunner)
blender_classes.append(SABIProperties)
blender_classes.append(SABIPropertiesGroup)


def register():
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)

    bpy.types.Object.sabi_properties = bpy.props.PointerProperty(
        type=SABIPropertiesGroup)


def unregister():
    blender_classes.reverse()
    for blender_class in blender_classes:
        bpy.utils.unregister_class(blender_class)
