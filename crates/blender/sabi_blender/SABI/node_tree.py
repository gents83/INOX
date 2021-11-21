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

    is_just_opened = True

    def update_nodes(self):
        for n in self.nodes:
            n.init(self)

    def update(self):
        if self.is_just_opened:
            self.is_just_opened = False
            self.update_nodes()


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

    updated_node_inputs = []
    updated_node_outputs = []

    def add_to_node(name, fullname, value_type, is_input):
        if is_input:
            updated_node_inputs.append((value_type, fullname, name))
        else:
            updated_node_outputs.append((value_type, fullname, name))

    def add_fields(node, dictionary, group_name, is_parent_input):
        for key in dictionary:
            name = str(key)
            is_input = is_parent_input
            if name.startswith("in_"):
                is_input = True
            elif name.startswith("out_"):
                is_input = False
            name = name.removeprefix("in_").removeprefix("out_")
            group = group_name.removeprefix("in_").removeprefix("out_")
            if group_name == "":
                fullname = name
            else:
                fullname = "[" + group + "]" + name
            value = dictionary[key]
            value_type = type(value)

            if value_type is int:
                add_to_node(name, fullname, "NodeSocketInt", is_input)
            elif value_type is float:
                add_to_node(name, fullname, "NodeSocketFloat", is_input)
            elif value_type is bool:
                add_to_node(name, fullname, "NodeSocketBool", is_input)
            elif value_type is dict:
                add_fields(node, value, fullname, is_input)
            elif value_type is str:
                if name == "type_name":
                    if value == "ScriptExecution":
                        add_to_node(fullname, group,
                                    "LogicExecutionSocket", is_parent_input)
                else:
                    add_to_node(name, fullname, "NodeSocketString", is_input)
            else:
                print("Type not supported " + str(value_type) + " for " + name)

    def update_inputs(node):
        for input in node.inputs:
            exists = False
            for n in updated_node_inputs:
                if input.name == n[1]:
                    exists = True
            if not exists:
                node.inputs.remove(input)

        for n in updated_node_inputs:
            exists = False
            for input in node.inputs:
                if input.name == n[1]:
                    exists = True
            if not exists:
                node.inputs.new(n[0], n[1])

    def update_outputs(node):
        for output in node.outputs:
            exists = False
            for n in updated_node_outputs:
                if output.name == n[1]:
                    exists = True
            if not exists:
                node.outputs.remove(output)

        for n in updated_node_outputs:
            exists = False
            for output in node.outputs:
                if output.name == n[1]:
                    exists = True
            if not exists:
                node.outputs.new(n[0], n[1])

    def register_fields(node):
        dict_from_fields = json.loads(serialized_class)
        print("Serialized in Rust:\n" + str(dict_from_fields))
        add_fields(node, dict_from_fields, "", False)
        update_inputs(node)
        update_outputs(node)

    def init(self, context):
        register_fields(self)

    def serialize_fields(dict, fields, is_input):
        for f in fields:
            name = f.name
            if is_input:
                name = "in_" + name
            else:
                name = "out_" + name
            print("Field type " + f.bl_idname + " for " + name)
            if f.bl_idname == "LogicExecutionSocket":
                script_execution = {}
                script_execution["type_name"] = "ScriptExecution"
                dict[name] = script_execution
            elif f.bl_idname == "NodeSocketInt":
                dict[name] = int(f.default_value)
            elif f.bl_idname == "NodeSocketFloat":
                dict[name] = float(f.default_value)
            elif f.bl_idname == "NodeSocketBool":
                dict[name] = bool(f.default_value)
            elif f.bl_idname == "NodeSocketString":
                dict[name] = str(f.default_value)
            elif hasattr(input, "default_value"):
                dict[name] = f.default_value
            else:
                dict[name] = len(f.links)

    def serialize(self):
        serialized_class = {}
        serialize_fields(serialized_class, self.inputs, True)
        serialize_fields(serialized_class, self.outputs, False)
        print("Serialized from Python:\n" + str(serialized_class))

    node_class = type(
        node_name,
        (base_type, Node, ),
        {
            "bl_idname": node_name,
            "bl_label": node_name,
            "name": node_name,
            "description": description,
            "init": init,
            "serialize": serialize,
        }
    )

    RUST_NODES.append(node_class)


class LogicNodeCategory(nodeitems_utils.NodeCategory):
    @ classmethod
    def poll(cls, context):
        return context.space_data.tree_type == 'LogicNodeTree'


# make a list of node categories for registration
node_categories = []


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

blender_classes.append(OpenInLogicEditor)


def register():
    for cls in blender_classes:
        bpy.utils.register_class(cls)

    nodeitems_utils.register_node_categories("LOGIC_NODES", node_categories)


def unregister():
    nodeitems_utils.unregister_node_categories("LOGIC_NODES")

    for cls in blender_classes:
        bpy.utils.unregister_class(cls)
