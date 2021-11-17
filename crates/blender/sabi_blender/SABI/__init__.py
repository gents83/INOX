
import bpy
from glob import glob
from os.path import dirname, join, isfile
from os import chmod, add_dll_directory

from . import *

if "bpy" in locals():
    import importlib
    if "keymaps" in locals():
        importlib.reload(keymaps)
    if "node_tree" in locals():
        importlib.reload(node_tree)
    if "operators" in locals():
        importlib.reload(operators)
    if "panels" in locals():
        importlib.reload(panels)

from . import keymaps
from . import node_tree
from . import operators
from . import panels

bl_info = {
    "name": "SABI Engine",
    "author": "GENTS <gents83@gmail.com>",
    "version": (0, 1, 0),
    "blender": (2, 93, 0),
    "location": "Everywhere",
    "description": "SABI Game Engine",
    "category": "Game Engines",
}


blender_classes = []


class SABIAddonPreferences(bpy.types.AddonPreferences):
    # this must match the add-on name, use '__package__'
    # when defining this in a submodule of a python package.
    bl_idname = __name__

    exe_path: bpy.props.StringProperty(
        name="SABI folder",
        description="Set folder where sabi_launcher.exe can be found",
        subtype="DIR_PATH",
        default="./bin/")

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "exe_path")


blender_classes.append(SABIAddonPreferences)


def register():
    # Ensure "Execute" permissions on files in the "bin" dir
    addon_dir = dirname(__spec__.origin)
    add_dll_directory(addon_dir)

    # Register Blender Classes
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)

    preferences = bpy.context.preferences.addons['SABI'].preferences

    file_path = join(preferences.exe_path, "*")
    for file_path in glob(file_path):
        chmod(file_path, 0o755)

    keymaps.register()
    node_tree.register()
    operators.register()
    panels.register()


def unregister():
    panels.unregister()
    operators.unregister()
    node_tree.unregister()
    keymaps.unregister()

    # Unregister Blender Classes
    blender_classes.reverse()
    for blender_class in blender_classes:
        bpy.utils.unregister_class(blender_class)
