extern crate xml;
extern crate bindgen;

use std::{
    env,
    collections::HashSet, 
    path::Path, 
    fs::{File, OpenOptions}, 
    io::{Write, BufReader}
};

use xml::reader::{ EventReader, XmlEvent };

fn main() {
    // Write the bindings to the $OUT_DIR/bindings.rs file.  
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("bindings.rs");

    let vulkan_sdk_path = env::var("VULKAN_SDK").unwrap();
    let mut vulkan_header = vulkan_sdk_path.clone();
    vulkan_header.push_str("\\include\\vulkan\\vulkan.h");
    
    let mut builder = bindgen::Builder::default()
                            .header(vulkan_header.to_owned())
                            .rustfmt_bindings(true)
                            .ignore_functions()
                            .ignore_methods();

    #[cfg(windows)]
    {
        builder = builder
                    .clang_arg("-DVK_USE_PLATFORM_WIN32_KHR")
                    .opaque_type("_IMAGE_TLS_DIRECTORY64");
    }
    #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
    {
        builder = builder
                    .clang_arg("-VK_USE_PLATFORM_XLIB_KHR");
    }
    #[cfg(target_os = "macos")]
    {
        builder = builder
                    .clang_arg("-VK_USE_PLATFORM_MACOS_MVK");
    }
    #[cfg(target_os = "macos")]
    {
        builder = builder
                    .clang_arg("-VK_USE_PLATFORM_ANDROID_KHR");
    }

    let bindings = builder
                .generate()
                .expect("Unable to generate bindings");

    bindings
        .write_to_file(dest_path.clone())
        .expect("Couldn't write bindings!");

    let mut f = OpenOptions::new().append(true).open(dest_path).unwrap();

    let mut vulkan_xml = vulkan_sdk_path.clone();
    vulkan_xml.push_str("\\share\\vulkan\\registry\\vk.xml");
    let file = File::open(vulkan_xml.to_owned()).unwrap();
    let file = BufReader::new(file);

    let mut allcommands: HashSet<_> = HashSet::new();
    let mut commands_v10: HashSet<_> = HashSet::new();
    let mut commands_v11: HashSet<_> = HashSet::new();
    let mut commands_v12: HashSet<_> = HashSet::new();
    let mut commands_extensions: HashSet<_> = HashSet::new();

    let mut is_command = false;
    let mut is_proto = false;
    let mut is_fn_name = false;
    let mut is_extension = false;
    let mut should_be_excluded = false;
    let mut is_feature_requirement = false;
    let mut version_number = String::from("1.0");

    let parser = EventReader::new(file);
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {  
                match name.to_string().as_str() {
                "command" => { 
                    is_command = true;
                    if is_extension && should_be_excluded {
                        match attributes.iter().find(|ref attr| {
                            attr.to_string().contains("name")
                        }) {
                            Some(a) => {             
                                commands_extensions.insert(a.value.clone());
                            },
                            None => ()
                        };
                    }
                    else if is_feature_requirement {
                        match attributes.iter().find(|ref attr| {
                            attr.to_string().contains("name")
                        }) {
                            Some(a) => {             
                                match version_number.as_str() {
                                    "1.2" => commands_v12.insert(a.value.clone()),
                                    "1.1" => commands_v11.insert(a.value.clone()),
                                    _ => commands_v10.insert(a.value.clone()),
                                };
                            },
                            None => ()
                        };
                    }
                },
                "feature" => {
                    is_feature_requirement = true;
                    version_number = {
                    match attributes.iter().find(|ref attr| {
                        attr.to_string().contains("number")
                    }) {
                        Some(a) => a.value.clone(),
                        None => String::from("1.0")
                    }
                    };
                },
                "proto" => is_proto = true,
                "extension" => {
                    is_extension = true;
                    should_be_excluded = {
                    match attributes.iter().find(|ref attr| {
                        attr.to_string().contains("platform") ||
                        attr.to_string().contains("deprecated") ||
                        attr.to_string().contains("specialuse") ||
                        attr.value.to_string() == "NV" ||
                        attr.value.to_string() == "EXT"
                    }) {
                        Some(_) => true,
                        None => false
                    }
                    };
                },
                "name" => is_fn_name = true,
                _ => (),
                }
            }
            Ok(XmlEvent::Characters(text)) => {
                if is_fn_name {
                if should_be_excluded {
                    commands_extensions.insert(text.clone());
                }
                else if is_command && is_proto {
                    allcommands.insert(text);
                }
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                "command" => is_command = false,
                "feature" => is_feature_requirement = false,
                "proto" => is_proto = false,
                "extension" => { is_extension = false; should_be_excluded = false; },
                "name" => is_fn_name = false,
                _ => (),
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    writeln!(f, "").unwrap();
    writeln!(f, "").unwrap();
    writeln!(f, "// autogenerated vulkan_bindings code - DO NOT EDIT manually").unwrap();
    writeln!(f, "").unwrap();

    let allcommands = allcommands.difference(&commands_extensions)
                        .collect::<HashSet<_>>();

    let mut allcommands:Vec<_> = allcommands.into_iter().collect();
    allcommands.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    for command in allcommands.clone() {
        writeln!(f, "pub static mut {}: PFN_{} = None;", command, command).unwrap();
    }

    writeln!(f, "").unwrap();

    writeln!(f, "pub struct VK;").unwrap();

    writeln!(f, "").unwrap();

    writeln!(f, "").unwrap();

    writeln!(f, "impl<'a> VK {{").unwrap();
    writeln!( f, "  pub fn initialize(lib : &'a Lib) {{").unwrap();
    writeln!(f, "       unsafe {{").unwrap();    
    for command in allcommands.clone() {
        writeln!(f, "       {} = lib.library.get::<PFN_{}>(\"{}\");", command, command, command).unwrap();
    }
    writeln!(f, "       }}").unwrap();
    writeln!(f, "   }}").unwrap();
    writeln!(f, "}}").unwrap();
}