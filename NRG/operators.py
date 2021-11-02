import bpy

from glob import glob
from os.path import join, exists
from os import chmod, mkdir
from pathlib import Path
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

        path = Path(preferences.exe_path).absolute()
        last_part = path.parts[-1]
        if last_part.endswith('debug') or last_part.endswith('release'):
            path = path.parent.absolute().parent.absolute()

        filename = Path(bpy.data.filepath).absolute().parts[-1]

        data_raw_path = join(join(join(path, "data_raw"),
                             "blender_export"), filename.replace('.blend', ''))

        if not Path(data_raw_path).exists():
            Path(data_raw_path).mkdir(parents=True, exist_ok=True)

        filename = join(data_raw_path, filename.replace('.blend', '.gltf'))

        print("Exporting scene into: " + data_raw_path)
        bpy.ops.export_scene.gltf(filepath=filename, check_existing=True, export_format='GLTF_SEPARATE',
                                  export_apply=True, export_materials='EXPORT', export_cameras=True,
                                  export_yup=True, export_lights=True)

        filename = filename.replace('data_raw', 'data')
        filename = filename.replace('.gltf', '.scene_data')

        print("Running NRG Engine")
        print(nrg_blender.execute(str(preferences.exe_path), str(filename)))

        return {'FINISHED'}


blender_classes.append(NRGRun)


def register():
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)


def unregister():
    blender_classes.reverse()
    for blender_class in blender_classes:
        bpy.utils.unregister_class(blender_class)
