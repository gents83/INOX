#![crate_type = "dylib"]

extern crate cpython;

use std::{path::PathBuf, process::Command};

use cpython::{py_fn, py_module_initializer, PyResult, Python};

// add bindings to the generated python module
// N.B: names: "nrg_blender" must be the name of the `.so` or `.pyd` file
py_module_initializer!(nrg_blender, initnrg_blender, PyInit_nrg_blender, |py, m| {
    m.add(py, "__doc__", "This module is implemented in Rust")?;
    m.add(
        py,
        "execute",
        py_fn!(py, execute(executable_path: &str, file_to_load: &str)),
    )?;
    Ok(())
});

fn execute(_py: Python, executable_path: &str, file_to_load: &str) -> PyResult<String> {
    let mut output_string = String::new();

    let mut path = PathBuf::from(executable_path);
    let mut current_dir = path.clone();
    if current_dir.ends_with("release") || current_dir.ends_with("debug") {
        current_dir.pop();
        current_dir.pop();
    }
    output_string += format!("Current Dir = {:?}\n", current_dir.as_path()).as_str();

    path = path.join("nrg_launcher.exe");

    output_string += format!("Path to command = {:?}\n", path.as_path()).as_str();

    let mut command = Command::new(path.as_path());
    command
        .arg("-plugin nrg_viewer")
        .arg("-load_file")
        .arg(file_to_load)
        .current_dir(current_dir.as_path());

    let result = if let Ok(process) = command.spawn() {
        let output = process
            .wait_with_output()
            .expect("failed to execute process");
        String::from_utf8(output.stdout).unwrap()
    } else {
        String::from("Failed to execute process")
    };
    output_string += result.as_str();

    Ok("Rust output: ".to_owned() + output_string.as_str())
}
