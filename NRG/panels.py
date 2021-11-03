import bpy
from . import node_tree

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


def filter_on_custom_property(self, node_group):
    return node_group.bl_idname == 'LogicNodeTree'


class NRGProperties(bpy.types.PropertyGroup):
    logic: bpy.props.PointerProperty(name="Logic",
                                     description="Logic node tree",
                                     type=node_tree.LogicNodeTree,
                                     poll=filter_on_custom_property)

    def draw(self, layout, context):
        layout.prop_search(self, "logic", bpy.data, "node_groups")


class ObjectData(bpy.types.Panel):
    """NRG related Object Data"""
    bl_label = "NRG Properties"
    bl_idname = "NRG_PT_object_data"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "object"

    def draw(self, context):
        layout = self.layout

        obj = context.object
        row = layout.row()
        obj.nrg_properties.draw(layout, context)


blender_classes.append(NRGRunner)
blender_classes.append(NRGProperties)
blender_classes.append(ObjectData)


def register():
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)

    bpy.types.Object.nrg_properties = bpy.props.PointerProperty(
        type=NRGProperties)


def unregister():
    blender_classes.reverse()
    for blender_class in blender_classes:
        bpy.utils.unregister_class(blender_class)
