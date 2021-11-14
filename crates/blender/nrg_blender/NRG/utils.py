import bpy
import functools


def gettype(name):
    from collections import deque
    # q is short for "queue", here
    q = deque([object])
    while q:
        t = q.popleft()
        if t.__name__ == name:
            return t

        try:
            # Keep looking!
            q.extend(t.__subclasses__())
        except TypeError:
            # type.__subclasses__ needs an argument, for whatever reason.
            if t is type:
                continue
            else:
                raise
    else:
        raise ValueError('No such type: %r' % name)


# Map from JSON strings to blender property types
TYPE_PROPERTIES = {
    "string": bpy.props.StringProperty,
    "bool": bpy.props.BoolProperty,
    "f64": bpy.props.FloatProperty,
    "f32": bpy.props.FloatProperty,
    "usize": bpy.props.IntProperty,
    "u32": bpy.props.IntProperty,
    "i32": bpy.props.IntProperty,
    "Vector4": functools.partial(bpy.props.FloatVectorProperty, size=4),
    "Vector3": functools.partial(bpy.props.FloatVectorProperty, size=3),
    "Vector2": functools.partial(bpy.props.FloatVectorProperty, size=2),
    "enum": bpy.props.EnumProperty,
}
