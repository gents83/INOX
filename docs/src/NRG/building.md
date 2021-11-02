# Building **NRG** code

## How to download _**NRG**_ repository

You can clone it using [https://github.com/gents83/NRG.git](https://github.com/gents83/NRG.git)

Or you can download the zip from [here](https://github.com/gents83/NRG/archive/refs/heads/master.zip)

## How to build and run _**NRG**_ code

Once that you've cloned and downloaded the repository, you can build it by running the following commands:

> git submodule update --init --recursive

The above command is needed to download all extern submodules and could require some time.

> cargo build --manifest-path ./crates/Cargo.toml

The above command build the project in debug but you can switch to release adding --release at the end.

> cargo run --manifest-path ./crates/Cargo.toml -- -plugin nrg_viewer

The above command run the nrg_launcher executable with the nrg_viewer plugin. \
Please provided needed command line parameters, like the -file_to_load 'path' \
See the section [Command line parameters accepted by nrg_launcher](#command-line-parameters-accepted-by-nrg_launcher)

## How to use _**Microsoft VSCode**_ launch and tasks created for _**NRG**_ on Windows

If you are using _**Microsoft VSCode**_ on Windows you can benefits of several shortcuts to build and execute _**NRG**_ code.

Using the _NRG.code-workspace_ file will allow you to get even raccomandation on useful extensions to maximise the _VSCode_ experience.

Let's see some useful shortcuts.

- When building or pressing CTRL+SHIFT+B: \
   You'll have many debug possibilities as:
   - BUILD DEBUG - Build workspace in debug
   - BUILD RELEASE - Build workspace in release
   - BUILD BOOK - Build and launch a rendered version of documentation in `/docs/` folder
   - RUN CLIPPY - Execute clippy fix on crates code and check if there are any errors
   - CHECK CRATES DEPENDENCIES - Check crates dependencies detecting and warning on unused ones.

- When debugging or pressing F5: \
   You'll have many debug possibilities as:
   - RUN LAUNCHER DEBUG - Run the launcher in debug mode
   - RUN LAUNCHER RELEASE - Run the launcher in release mode
   - RUN VIEWER RELEASE - Run the viewer with a test scene (customizable in `/.vscode/launch.json`)

## Command line parameters accepted by **nrg_launcher**

- **-plugin [name]**: \
    The plugin to use. \
    You can specify names of crates inside apps folder like nrg_viewer, nrg_editor, etc \
    When not specified an empty window will be opened with only the binarizer executing in background.
    
- **-load_file [path]**: \
    A path of a scene to load with path relative to `/data/` \
    As example could be `./data/blender_export/TestScene/TestScene.scene_data`