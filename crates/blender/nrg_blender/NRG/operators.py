import bpy
import time
import threading
import socket
import platform

from glob import glob
from os.path import join, exists
from os import chmod, mkdir
from pathlib import Path


blender_classes = []

nrg_engine = None


class NRGRun(bpy.types.Operator):
    """Run NRG Engine"""
    bl_idname = "nrg.run"
    bl_label = "Run in NRG"

    def __init__(self):
        global nrg_engine
        if nrg_engine is None:
            from NRG import nrg_blender
            nrg_engine = nrg_blender.NRGEngine()

    def execute(self, context):
        # Ensure blend has been saved before running game
        if bpy.data.filepath == "":
            def draw_popup(popup, context):
                popup.layout.operator_context = 'INVOKE_AREA'
                popup.layout.label(text="Save Blend Before Running Game")
                popup.layout.operator("wm.save_mainfile")
            context.window_manager.popover(draw_popup)
            return {'FINISHED'}

        preferences = context.preferences.addons['NRG'].preferences

        file_path = join(preferences.exe_path, "*")
        for file_path in glob(file_path):
            chmod(file_path, 0o755)

        path = Path(preferences.exe_path).absolute()
        last_part = path.parts[-1]
        if last_part.endswith('debug') or last_part.endswith('release'):
            path = path.parent.absolute().parent.absolute().parent.absolute()
#
        from NRG import nrg_blender
        print(nrg_blender.start(nrg_engine, str(
            preferences.exe_path)))

        print(nrg_blender.export(nrg_engine, str(
            bpy.data.filepath)))

        # Do NOT wait for the thread to be ended
        return {'FINISHED'}


blender_classes.append(NRGRun)


def register():
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)


def unregister():
    blender_classes.reverse()
    for blender_class in blender_classes:
        bpy.utils.unregister_class(blender_class)
