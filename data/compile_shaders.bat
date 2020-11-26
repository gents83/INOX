%VULKAN_SDK%\Bin\glslc.exe -S shader.vert -o vert.spv_assembly
%VULKAN_SDK%\Bin\glslc.exe -O shader.vert -o vert.spv
%VULKAN_SDK%\Bin\spirv-val.exe vert.spv
%VULKAN_SDK%\Bin\glslc.exe -S shader.frag -o frag.spv_assembly
%VULKAN_SDK%\Bin\glslc.exe -O shader.frag -o frag.spv
%VULKAN_SDK%\Bin\spirv-val.exe frag.spv
pause