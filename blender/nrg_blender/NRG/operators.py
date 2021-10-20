import bpy

from glob import glob
from os.path import join
from os import chmod, add_dll_directory
from . import nrg_blender


blender_classes = []


class NRGRun(bpy.types.Operator):
    """Run NRG Engine"""
    bl_idname = "nrg.run"
    bl_label = "Run in NRG"

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

        print("Running NRG Engine")

        print(nrg_blender.execute(str(preferences.exe_path)))

        return {'FINISHED'}


blender_classes.append(NRGRun)


def register():
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)


def unregister():
    blender_classes.reverse()
    for blender_class in blender_classes:
        bpy.utils.unregister_class(blender_class)
