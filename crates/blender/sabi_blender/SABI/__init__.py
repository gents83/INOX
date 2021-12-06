
import os
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


def load_dlls():
    preferences = bpy.context.preferences.addons['SABI'].preferences

    if os.path.isdir(preferences.exe_path):
        if len(preferences.libs_to_load) == 0:
            for file in os.listdir(preferences.exe_path):
                if file.endswith(".dll") or file.endswith(".so"):
                    preferences.libs_to_load.append(file)


class SABIAddonPreferences(bpy.types.AddonPreferences):
    # this must match the add-on name, use '__package__'
    # when defining this in a submodule of a python package.
    bl_idname = __name__

    exe_path: bpy.props.StringProperty(
        name="SABI folder",
        description="Set folder where sabi_launcher.exe can be found",
        subtype="DIR_PATH",
        default="./bin/")

    checkboxes: bpy.props.BoolVectorProperty(
        name="DLLs to load", size=32)

    libs_to_load = []

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "exe_path")

        box = layout.box()
        split = box.split(factor=.25)
        column = split.column()
        column.label(text="DLLs to load:")
        for i, lib in enumerate(self.libs_to_load):
            name = lib.removesuffix(".dll").removesuffix(".so")
            column.row().prop(self, 'checkboxes', index=i, text=name)


blender_classes.append(SABIAddonPreferences)


def register():
    # Ensure "Execute" permissions on files in the "bin" dir
    addon_dir = dirname(__spec__.origin)
    add_dll_directory(addon_dir)

    # Register Blender Classes
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)

    preferences = bpy.context.preferences.addons['SABI'].preferences
    load_dlls()

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
