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

    def serialize(self):
        node_tree = {}
        nodes = {}
        links = []

        for l in self.links:
            links.append((l.from_socket.node.name, l.from_socket.name,
                          l.to_socket.node.name, l.to_socket.name))
            if hasattr(l.from_socket, 'default_value') and hasattr(l.to_socket, 'default_value'):
                l.to_socket.default_value = l.from_socket.default_value
        for n in self.nodes:
            nodes[n.name] = n.serialize()

        node_tree['nodes'] = nodes
        node_tree['links'] = links

        return json.dumps(node_tree)


class LogicExecutionSocket(NodeSocket):
    bl_idname = 'LogicExecutionSocket'
    bl_label = 'Script Execution Socket'

    def draw(self, context, layout, node, text):
        layout.label(text=text)

    def draw_color(self, context, node):
        return (1.0, 0.0, 1.0, 1.0)


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


class FieldData:
    def __init__(self, key, value, is_parent_input, group_name):
        self.key = str(key)
        self.is_input = is_parent_input
        if self.key.startswith("in_"):
            self.is_input = True
        elif self.key.startswith("out_"):
            self.is_input = False
        self.name = self.key.removeprefix("in_").removeprefix("out_")
        self.group = group_name.removeprefix("in_").removeprefix("out_")
        if group_name == "":
            self.fullname = self.name
        else:
            self.fullname = self.group + "." + self.name
        self.value = value
        self.value_type = type(self.value)


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

    def create_fields_data(dictionary, group_name, is_parent_input):
        fields_data = []
        for key in dictionary:
            f = FieldData(
                key, dictionary[key], is_parent_input, group_name)
            if f.value_type is dict:
                inner_fields = create_fields_data(f.value,
                                                  f.fullname, f.is_input)
                for i in inner_fields:
                    fields_data.append(i)
            else:
                fields_data.append(f)
        return fields_data

    fields_dictionary = json.loads(serialized_class)
    fields_data = create_fields_data(fields_dictionary, "", False)

    def socket_from_field(f):
        socket_type = "LogicExecutionSocket"
        if f.value_type is int:
            socket_type = "NodeSocketInt"
        elif f.value_type is float:
            socket_type = "NodeSocketFloat"
        elif f.value_type is bool:
            socket_type = "NodeSocketBool"
        elif f.value_type is str:
            if f.name == "type_name" and f.value == "ScriptExecution":
                socket_type = "LogicExecutionSocket"
            else:
                socket_type = "NodeSocketString"
        return socket_type

    def update_sockets(new_values, node_values):
        for v in node_values:
            exists = False
            for n in new_values:
                if v.name == n[0]:
                    exists = True
            if not exists:
                node_values.remove(v)

        for n in new_values:
            exists = False
            for v in node_values:
                if v.name == n[0]:
                    exists = True
            if not exists:
                node_values.new(n[1], n[0])

    def deserialize(self):
        inputs = []
        outputs = []
        for f in fields_data:
            socket_type = socket_from_field(f)
            if socket_type == "LogicExecutionSocket":
                name = f.group
            else:
                name = f.fullname
            if f.is_input:
                inputs.append((name, socket_type))
            else:
                outputs.append((name, socket_type))
        update_sockets(inputs, self.inputs)
        update_sockets(outputs, self.outputs)

    def serialize_fields(self, dictionary, group_name, is_parent_input, input_index, output_index):
        for f in fields_data:
            if f.value_type is int or f.value_type is float or f.value_type is bool or f.value_type is str:
                if f.is_input:
                    i = [x for x in self.inputs if x.name == f.fullname]
                    if len(i) > 0 and hasattr(i[0], "default_value"):
                        dictionary[f.key] = i[0].default_value
                else:
                    o = [x for x in self.outputs if x.name == f.fullname]
                    if len(o) > 0 and hasattr(o[0], "default_value"):
                        dictionary[f.key] = o[0].default_value
            else:
                if f.is_input:
                    i = [x for x in self.inputs if x.name == f.fullname]
                else:
                    o = [x for x in self.outputs if x.name == f.fullname]
        return dictionary

    def serialize(self):
        output = self.serialize_fields(
            fields_dictionary, "", False, 0, 0)
        return json.dumps(output)

    def init(self, context):
        self.deserialize()

    node_class = type(
        node_name,
        (base_type, Node, ),
        {
            "bl_idname": node_name,
            "bl_label": node_name,
            "name": node_name,
            "description": description,
            "init": init,
            "deserialize": deserialize,
            "serialize": serialize,
            "serialize_fields": serialize_fields,
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
