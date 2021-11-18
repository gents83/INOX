import nodeitems_utils
import bpy
from bpy.types import NodeTree, Node, NodeSocket, Operator, PropertyGroup
import json

blender_classes = []

RUST_NODES = []


class LogicNodeTree(NodeTree):
    '''A Logic node tree type'''
    bl_idname = 'LogicNodeTree'
    bl_label = 'Logic Node Tree'
    bl_icon = 'BLENDER'


class LogicExecutionSocket(NodeSocket):
    bl_idname = 'LogicExecutionSocket'
    bl_label = 'Script Execution Socket'

    def draw(self, context, layout, node, text):
        layout.label(text=text)

    def draw_color(self, context, node):
        return (0.0, 0.0, 1.0, 1.0)


class LogicNodeBase(Node):
    bl_idname = 'LogicNodeBase'
    bl_label = 'Logic Node Base'

    @classmethod
    def poll(cls, ntree):
        return ntree.bl_idname == 'LogicNodeTree'

    def copy(self, node):
        print("copied node", node)

    def free(self):
        print("Node removed", self)


def register_nodes(sabi_engine):
    from SABI import sabi_blender
    sabi_blender.register_nodes(sabi_engine)

    global RUST_NODES
    node_items = []
    for n in RUST_NODES:
        bpy.utils.register_class(n)
        node_items.append(
            nodeitems_utils.NodeItem(n.name,
                                     label=n.name)
        )

    nodeitems_utils.register_node_categories(
        "RUST_NODES", [nodeitems_utils.NodeCategory(
            "RUST_CATEGORY", "Rust Nodes", items=node_items)])


def create_node_from_data(node_name, base_class, description, serialized_class):
    from SABI import utils
    base_type = utils.gettype(base_class)

    # Create a class that stores all the internals of the properties in
    # a blender-compatible way.
    properties = type(
        node_name + "Properties",
        (PropertyGroup, ),
        {
            "bl_idname": node_name + "Properties"
        }
    )

    def is_field_input(key):
        name = str(key)
        if name.startswith("in_"):
            return True
        elif name.startswith("out_"):
            return False
        elif name != "type_name":
            print(
                "Node members should start with 'in_' or 'out_'! Invalid name: " + name)
        return False

    def field_name(key, is_input):
        if is_input:
            return key[3:]
        else:
            return key[4:]

    def add_to_node(node, name, value_type, is_input):
        field = field_name(name, is_input)
        if is_input:
            node.inputs.new(value_type, field)
        else:
            node.outputs.new(value_type, field)

    def add_fields(node, dictionary, group_name):
        for key in dictionary:
            name = str(key)
            value = dictionary[key]
            value_type = type(value)
            is_input = is_field_input(name)

            if value_type is int:
                add_to_node(node, name, "NodeSocketInt", is_input)
            elif value_type is float:
                add_to_node(node, name, "NodeSocketFloat", is_input)
            elif value_type is bool:
                add_to_node(node, name, "NodeSocketBool", is_input)
            elif value_type is dict:
                add_fields(node, value, name)
            elif value_type is str:
                if name == "type_name":
                    is_input = is_field_input(group_name)
                    if value == "ScriptExecution":
                        add_to_node(node, group_name,
                                    "LogicExecutionSocket", is_input)
                else:
                    add_to_node(
                        node, name, "NodeSocketString", is_input)
            else:
                print("Type not supported " + str(value_type) + " for " + name)

    def register_fields(node):
        properties.node = node
        dict_from_fields = json.loads(serialized_class)
        add_fields(node, dict_from_fields, "node")

    def draw(self, layout, context):
        col = layout.column()
        # for field_name in properties.node.inputs:
        #    col.prop(self, field_name)

    properties.draw = draw

    def register():
        bpy.utils.register_class(properties)
        node_class.properties = bpy.props.PointerProperty(
            name="properties", type=properties)

    def unregister():
        bpy.utils.unregister_class(properties)

    def init(self, context):
        register_fields(self)

    def draw_buttons(self, context, layout):
        row = layout.row()
        self.properties.draw(row, context)

    # Create a class to store the data about this node inside the
    # blender object
    node_class = type(
        node_name,
        (base_type, Node, ),
        {
            "bl_idname": node_name,
            "bl_label": node_name,
            "name": node_name,
            "description": description,
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
    @ classmethod
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


class OpenInLogicEditor(Operator):
    bl_idname = "sabi.open_in_logic_editor"
    bl_label = "Open Logic Editor"

    def execute(self, context):
        for area in bpy.context.screen.areas:
            if area.type == 'VIEW_3D':
                area.type = 'NODE_EDITOR'
                area.spaces.active.node_tree = context.object.sabi_properties.logic
        return {'FINISHED'}


blender_classes.append(LogicNodeTree)
blender_classes.append(LogicExecutionSocket)
blender_classes.append(LogicNodeBase)
blender_classes.append(LogicSimpleInputNode)

blender_classes.append(OpenInLogicEditor)


def register():
    for cls in blender_classes:
        bpy.utils.register_class(cls)

    nodeitems_utils.register_node_categories("LOGIC_NODES", node_categories)


def unregister():
    nodeitems_utils.unregister_node_categories("LOGIC_NODES")

    for cls in blender_classes:
        bpy.utils.unregister_class(cls)
