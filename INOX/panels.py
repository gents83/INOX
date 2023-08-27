import bpy
from . import node_tree

blender_classes = []


class INOXRunner(bpy.types.Panel):
    """INOX Runner"""
    bl_label = "INOX"
    bl_idname = "INOX_PT_runner"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "render"

    def draw(self, context):
        layout = self.layout

        row = layout.row()
        row.operator("inox.run", icon='PLAY')


class INOXPropertiesGroup(bpy.types.PropertyGroup):
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
        col_2.operator("inox.open_in_logic_editor", icon='FILE_FOLDER')


class INOXProperties(bpy.types.Panel):
    """INOX related Object Data"""
    bl_label = "INOX Properties"
    bl_idname = "INOX_PT_object_data"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "object"

    def draw(self, context):
        layout = self.layout

        obj = context.object
        row = layout.row()
        obj.inox_properties.draw(layout, context)


blender_classes.append(INOXRunner)
blender_classes.append(INOXProperties)
blender_classes.append(INOXPropertiesGroup)


def register():
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)

    bpy.types.Object.inox_properties = bpy.props.PointerProperty(
        type=INOXPropertiesGroup)


def unregister():
    blender_classes.reverse()
    for blender_class in blender_classes:
        bpy.utils.unregister_class(blender_class)
