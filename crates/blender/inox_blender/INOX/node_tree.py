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
        nodes = []
        links = []

        for l in self.links:
            link = {}
            link["from_node"] = l.from_socket.node.name
            link["to_node"] = l.to_socket.node.name
            link["from_pin"] = l.from_socket.name
            link["to_pin"] = l.to_socket.name
            links.append(link)
            if hasattr(l.from_socket, 'default_value') and hasattr(l.to_socket, 'default_value'):
                l.to_socket.default_value = l.from_socket.default_value
        for n in self.nodes:
            serialized_node = {}
            serialized_node["node_type"] = n.bl_idname
            node = n.serialize()
            serialized_node["node"] = node["node"]
            nodes.append(serialized_node)

        node_tree['nodes'] = nodes
        node_tree['links'] = links

        return json.dumps(node_tree)


class NodeSocketLogicExecution(NodeSocket):
    bl_idname = 'NodeSocketLogicExecution'
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
    def __init__(self, name, value, is_input):
        self.name = str(name)
        self.is_input = is_input

        self.rust_type = str(value["pin_type"])
        if "value" in value:
            self.value = value["value"]
            self.value_type = type(self.value)

        if self.rust_type == "LogicExecution":
            self.socket_type = "NodeSocketLogicExecution"
        elif self.rust_type == "u32" or self.rust_type == "i32" or self.rust_type == "i8" or self.rust_type == "u8" or self.rust_type == "i16" or self.rust_type == "u16":
            self.socket_type = "NodeSocketInt"
        elif self.rust_type == "f32" or self.rust_type == "f64":
            self.socket_type = "NodeSocketFloat"
        elif self.rust_type == "String":
            self.socket_type = "NodeSocketString"
        elif self.value_type is int:
            self.socket_type = "NodeSocketInt"
        elif self.value_type is float:
            self.socket_type = "NodeSocketFloat"
        elif self.value_type is bool:
            self.socket_type = "NodeSocketBool"
        elif self.value_type is str:
            self.socket_type = "NodeSocketString"


def register_nodes(inox_engine):
    from INOX import inox_blender
    inox_blender.register_nodes(inox_engine)

    global RUST_NODES
    node_items = {}
    for n in RUST_NODES:
        bpy.utils.register_class(n)
        if n.category not in node_items:
            node_items[n.category] = []
        node_items[n.category].append(
            nodeitems_utils.NodeItem(n.name, label=n.name))

    for key in node_items:
        nodeitems_utils.register_node_categories(
            key, [nodeitems_utils.NodeCategory(
                key, key, items=node_items[key])])


def create_node_from_data(node_name, base_class, category, description, serialized_class):
    from INOX import utils
    base_type = utils.gettype(base_class)

    def extract(dictionary, is_input):
        types = "inputs"
        if is_input == False:
            types = "outputs"
        fields_data = []
        for k in dictionary["node"][types]:
            f = FieldData(k, dictionary["node"][types][k], is_input)
            fields_data.append(f)
        return fields_data

    fields_dictionary = json.loads(serialized_class)
    fields_input = extract(fields_dictionary, True)
    fields_output = extract(fields_dictionary, False)

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
        for f in fields_input:
            inputs.append((f.name, f.socket_type))
        for f in fields_output:
            outputs.append((f.name, f.socket_type))
        update_sockets(inputs, self.inputs)
        update_sockets(outputs, self.outputs)

    def serialize_fields(self, dictionary):
        for f in fields_input:
            if f.rust_type != "LogicExecution":
                i = [x for x in self.inputs if x.name == f.name]
                if len(i) > 0 and hasattr(i[0], "default_value"):
                    dictionary["node"]["inputs"][f.name]["value"] = i[0].default_value
        for f in fields_output:
            if f.rust_type != "LogicExecution":
                i = [x for x in self.outputs if x.name == f.name]
                if len(i) > 0 and hasattr(i[0], "default_value"):
                    dictionary["node"]["outputs"][f.name]["value"] = i[0].default_value
        return dictionary

    def serialize(self):
        output = self.serialize_fields(
            fields_dictionary)
        return output

    def init(self, context):
        self.deserialize()

    node_class = type(
        node_name,
        (base_type, Node, ),
        {
            "bl_idname": node_name,
            "bl_label": node_name,
            "name": node_name,
            "category": category,
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
    bl_idname = "inox.open_in_logic_editor"
    bl_label = "Open Logic Editor"

    def execute(self, context):
        for area in bpy.context.screen.areas:
            if area.type == 'VIEW_3D':
                area.type = 'NODE_EDITOR'
                area.spaces.active.node_tree = context.object.inox_properties.logic
        return {'FINISHED'}


blender_classes.append(NodeSocketLogicExecution)
blender_classes.append(LogicNodeTree)
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
