@ECHO OFF

ECHO "---------------------------------------------------------"
ECHO "Processing VERTEX SHADERS"
ECHO "---------------------------------------------------------"

for %%I in (.\source\*.vert) do (
	ECHO Processing '%%~nI'
	%VULKAN_SDK%\Bin\glslc.exe -S %%I -o .\temp\%%~nI_vert.spv_assembly
	%VULKAN_SDK%\Bin\glslangValidator.exe -o .\compiled\%%~nI_vert.spv -V %%I
	%VULKAN_SDK%\Bin\spirv-val.exe .\compiled\%%~nI_vert.spv
)

ECHO "---------------------------------------------------------"
ECHO "Processing FRAGMENT SHADERS"
ECHO "---------------------------------------------------------"

for %%I in (.\source\*.frag) do (
	ECHO Processing '%%~nI'
	%VULKAN_SDK%\Bin\glslc.exe -S %%I -o .\temp\%%~nI_frag.spv_assembly
	%VULKAN_SDK%\Bin\glslangValidator.exe -o .\compiled\%%~nI_frag.spv -V %%I
	%VULKAN_SDK%\Bin\spirv-val.exe .\compiled\%%~nI_frag.spv
)

ECHO "---------------------------------------------------------"
ECHO "Process ended"
ECHO "---------------------------------------------------------"

PAUSE