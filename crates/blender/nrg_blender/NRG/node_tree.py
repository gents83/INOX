import nodeitems_utils
import bpy


class LogicNodeTree(bpy.types.NodeTree):
    '''A Logic node tree type'''
    bl_idname = 'LogicNodeTree'
    bl_label = 'Logic Node Tree'
    bl_icon = 'BLENDER'


class LogicNodeBase(bpy.types.Node):
    @classmethod
    def poll(cls, ntree):
        return ntree.bl_idname == 'LogicNodeTree'


class LogicSimpleInputNode(LogicNodeBase):
    '''A simple input node'''
    bl_idname = 'LogicSimpleInputNode'
    bl_label = 'Simple Input Node'
    bl_icon = 'PLUS'

    integer_value: bpy.props.IntProperty(name="InputPin")

    def init(self, context):
        self.integer_value = 0
        self.outputs.new('OutputPin', "output")

    def copy(self, node):
        print("copied node", node)

    def free(self):
        print("Node removed", self)

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
                                 label="Simple Input Node", settings={"integer_value": repr(1.0)}),
        nodeitems_utils.NodeItem("LogicSimpleInputNode"),
    ]),
]


classes = (
    LogicNodeTree,
    LogicSimpleInputNode,
)


def register():
    for cls in classes:
        bpy.utils.register_class(cls)
    nodeitems_utils.register_node_categories("LOGIC_NODES", node_categories)


def unregister():
    nodeitems_utils.unregister_node_categories("LOGIC_NODES")
    for cls in classes:
        bpy.utils.unregister_class(cls)
