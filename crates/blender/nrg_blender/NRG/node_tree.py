import nodeitems_utils
import collections
import functools
import types
import bpy

blender_classes = []

RUST_NODES = []


class LogicNodeTree(bpy.types.NodeTree):
    '''A Logic node tree type'''
    bl_idname = 'LogicNodeTree'
    bl_label = 'Logic Node Tree'
    bl_icon = 'BLENDER'


class LogicNodeBase(bpy.types.Node):
    bl_idname = 'LogicNodeBase'

    @classmethod
    def poll(cls, ntree):
        return ntree.bl_idname == 'LogicNodeTree'

    def copy(self, node):
        print("copied node", node)

    def free(self):
        print("Node removed", self)


def register_nodes(nrg_engine):
    from NRG import nrg_blender
    nrg_blender.register_nodes(nrg_engine)

    global RUST_NODES
    node_items = []
    for n in RUST_NODES:
        print("name = " + n.name)
        print("type = " + str(n))
        bpy.utils.register_class(n)
        node_items.append(
            nodeitems_utils.NodeItem("NRG_" + n.name,
                                     label=n.name)
        )

    nodeitems_utils.register_node_categories(
        "RUST_NODES", [nodeitems_utils.NodeCategory(
            "RUST_CATEGORY", "Rust Nodes", items=node_items)])


def create_node_from_data(node):

    # Create a class that stores all the internals of the properties in
    # a blender-compatible way.
    properties = type(
        "NRG_" + node.node_type + "Properties",
        (bpy.types.PropertyGroup, ),
        {
            "bl_idname": "NRG_" + node.node_type + "Properties"
        }
    )

    def register_fields(self):
        print("register_fields")
        properties.integer_value = bpy.props.IntProperty(name="InputPin")
        properties.test = bpy.props.BoolProperty(name="test", default=False)

    def draw(self, layout, context):
        col = layout.column()
        col.prop(self, "integer_value")
        col.prop(self, "test")

    properties.register_fields = register_fields
    properties.draw = draw

    def register():
        print("register")
        bpy.utils.register_class(properties)
        node_class.properties = bpy.props.PointerProperty(
            name="properties", type=properties)

    def unregister():
        bpy.utils.unregister_class(properties)
        print("unregister")

    def init(self, context):
        print("init" + str(self))
        self.properties.register_fields()
        self.outputs.new("NodeSocketShader", "output")

    def draw_buttons(self, context, layout):
        row = layout.row()
        self.properties.draw(row, context)

    # Create a class to store the data about this node inside the
    # blender object
    node_class = type(
        "NRG_" + node.node_type,
        (LogicNodeBase, bpy.types.Node, ),
        {
            "bl_idname": "NRG_" + node.node_type,
            "bl_label": node.node_type,
            "name": node.node_type,
            "description": node.description,
            "init": init,
            "register": register,
            "unregister": unregister,
            "draw_buttons": draw_buttons,
        }
    )

    RUST_NODES.append(node_class)


class LogicSimpleInputNode(LogicNodeBase):
    '''A simple input node'''
    bl_idname = 'LogicSimpleInputNode'
    bl_label = 'Simple Input Node'
    bl_icon = 'PLUS'

    integer_value: bpy.props.IntProperty(name="InputPin")

    def init(self, context):
        self.integer_value = 0
        self.outputs.new("NodeSocketShader", "output")

    # NOTE: input sockets are drawn by their respective methods
    #   but output ones DO NOT for some reason, do it manually
    #   and connect the drawn value to the output socket
    def draw_buttons(self, context, layout):
        layout.prop(self, "integer_value")

    # this method lets you design how the node properties
    #   are drawn on the side panel (to the right)
    #   if it is not defined, draw_buttons will be used instead
    # def draw_buttons_ext(self, context, layout):


class LogicNodeCategory(nodeitems_utils.NodeCategory):
    @classmethod
    def poll(cls, context):
        return context.space_data.tree_type == 'LogicNodeTree'


# make a list of node categories for registration
node_categories = [
    LogicNodeCategory("LOGICINPUTNODES", "Logic Input Nodes", items=[
        #   NOTE: use 'repr()' to convert the value to string IMPORTANT
        nodeitems_utils.NodeItem("LogicSimpleInputNode",
                                 label="Simple Input Node"),
    ]),
]


class OpenInLogicEditor(bpy.types.Operator):
    bl_idname = "nrg.open_in_logic_editor"
    bl_label = "Open Logic Editor"

    def execute(self, context):
        for area in bpy.context.screen.areas:
            if area.type == 'VIEW_3D':
                area.type = 'NODE_EDITOR'
                area.spaces.active.node_tree = context.object.nrg_properties.logic
        return {'FINISHED'}


blender_classes.append(LogicNodeTree)
blender_classes.append(LogicSimpleInputNode)

blender_classes.append(OpenInLogicEditor)


def register():
    for cls in blender_classes:
        print("type = " + str(cls))
        bpy.utils.register_class(cls)

    nodeitems_utils.register_node_categories("LOGIC_NODES", node_categories)


def unregister():
    nodeitems_utils.unregister_node_categories("LOGIC_NODES")

    for cls in blender_classes:
        bpy.utils.unregister_class(cls)
