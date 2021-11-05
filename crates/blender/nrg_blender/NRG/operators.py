import bpy
import threading
import subprocess

from glob import glob
from os.path import join, exists
from os import chmod, mkdir
from pathlib import Path


blender_classes = []

nrg_engine = None
#process = None
#running_thread = None
#thread_request = None
#can_continue = True
#
#
# class ThreadRequest:
#    nrg_context = None
#    filepath = ""
#    exe_path = ""
#
#    def __init__(self, nrg_context, filepath, exe_path):
#        self.nrg_context = nrg_context
#        self.filepath = filepath
#        self.exe_path = exe_path
#
#
# def nrg_update():
#    import bpy
#    from NRG import nrg_blender
#
#    global can_continue
#    while can_continue:
#
#        global thread_request
#        if thread_request is not None:
#            filename = thread_request.filepath
#            exe_path = thread_request.exe_path
#
#            thread_request = None
#
#            print("Exporting scene into: " + filename)
#
#            for obj in bpy.context.scene.objects:
#                print("Exporting object: " + obj.name)
#
#            # bpy.ops.export_scene.gltf(filepath=filename, check_existing=True, export_format='GLTF_SEPARATE',
#            #                          export_apply=True, export_materials='EXPORT', export_cameras=True,
#            #                          export_yup=True, export_lights=True)
#
#            filename = filename.replace('data_raw', 'data')
#            filename = filename.replace('.gltf', '.scene_data')
#
#            print("Running NRG Engine")
#            print(nrg_blender.execute(
#                str(exe_path), str(filename)))


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
            path = path.parent.absolute().parent.absolute().parent.absolute()

        #filename = Path(bpy.data.filepath).absolute().parts[-1]

        # data_raw_path = join(join(join(path, "data_raw"),
        #                     "blender_export"), filename.replace('.blend', ''))
#
        # if not Path(data_raw_path).exists():
        #    Path(data_raw_path).mkdir(parents=True, exist_ok=True)
#
        #filename = join(data_raw_path, filename.replace('.blend', '.gltf'))
#
        #filename = filename.replace('data_raw', 'data')
        #filename = filename.replace('.gltf', '.scene_data')

        global nrg_engine
        if nrg_engine is None:
            from NRG import nrg_blender
            nrg_engine = nrg_blender.NRGEngine()

#
        print(nrg_blender.load(nrg_engine, str(
            preferences.exe_path), str(bpy.data.filepath)))

        #global process
        # if process is None:
        #    exe = join(preferences.exe_path, "nrg_launcher")
#
        #    filename = filename.replace('data_raw', 'data')
        #    filename = filename.replace('.gltf', '.scene_data')
#
        #    cmd_argument = [exe]
        #    cmd_argument.append('-plugin nrg_viewer')
        #    cmd_argument.append('-load_file '+filename)
#
        #    process = subprocess.Popen(
        #        cmd_argument, cwd=path)

        #global running_thread
        # if running_thread is None:
        #    global can_continue
        #    can_continue = True
        #    running_thread = threading.Thread(target=nrg_update)
        #    running_thread.start()
        #    print("Creating thread")
#
        #global thread_request
        # if thread_request is None:
        #    thread_request = ThreadRequest(
        #        bpy.context, filename, preferences.exe_path)
        #    print("Creating thread request")

        # Do NOT wait for the thread to be ended
        return {'FINISHED'}


blender_classes.append(NRGRun)


def register():
    for blender_class in blender_classes:
        bpy.utils.register_class(blender_class)


def unregister():
    #global can_continue
    #can_continue = False
    #
    #global running_thread
    # if running_thread is not None:
    #    running_thread.join()

    blender_classes.reverse()
    for blender_class in blender_classes:
        bpy.utils.unregister_class(blender_class)
