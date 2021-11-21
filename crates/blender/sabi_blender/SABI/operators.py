import bpy

from glob import glob
from os.path import join, exists
from os import chmod, mkdir
from pathlib import Path


blender_classes = []

sabi_engine = None


class SABIRun(bpy.types.Operator):
    """Run SABI Engine"""
    bl_idname = "sabi.run"
    bl_label = "Run in SABI"

    def execute(self, context):
        # Ensure blend has been saved before running game
        if bpy.data.filepath == "":
            def draw_popup(popup, context):
                popup.layout.operator_context = 'INVOKE_AREA'
                popup.layout.label(text="Save Blend Before Running Game")
                popup.layout.operator("wm.save_mainfile")
            context.window_manager.popover(draw_popup)
            return {'FINISHED'}

        preferences = context.preferences.addons['SABI'].preferences

        file_path = join(preferences.exe_path, "*")
        for file_path in glob(file_path):
            chmod(file_path, 0o755)

        path = Path(preferences.exe_path).absolute()
        last_part = path.parts[-1]
        if last_part.endswith('debug') or last_part.endswith('release'):
            path = path.parent.absolute().parent.absolute().parent.absolute()
#
        from SABI import sabi_blender
        # sabi_blender.start(sabi_engine)
        sabi_blender.export(sabi_engine, str(bpy.data.filepath), True)

        # Do NOT wait for the thread to be ended
        return {'FINISHED'}


blender_classes.append(SABIRun)


def register():
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)

    prefs = bpy.context.preferences.addons['SABI'].preferences
    libs_to_load = []
    for i, v in enumerate(prefs.checkboxes):
        if v is True and i < len(prefs.libs_to_load):
            libs_to_load.append(prefs.libs_to_load[i])

    global sabi_engine
    if sabi_engine is None:
        from SABI import sabi_blender
        from SABI import node_tree

        sabi_engine = sabi_blender.SABIEngine(
            str(prefs.exe_path), libs_to_load)
        node_tree.register_nodes(sabi_engine)


def unregister():
    blender_classes.reverse()
    for blender_class in blender_classes:
        bpy.utils.unregister_class(blender_class)
