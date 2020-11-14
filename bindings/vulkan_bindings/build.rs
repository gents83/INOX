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

  let bindings = bindgen::Builder::default()
    .header("data/Vulkan-Headers/include/vulkan/vulkan.h")
    .ignore_functions()
    .clang_arg("-DVK_NO_PROTOTYPES")
    .generate()
    .expect("Unable to generate bindings");
  
  bindings
      .write_to_file(dest_path.clone())
      .expect("Couldn't write bindings!");

  
  let mut f = OpenOptions::new().append(true).open(dest_path).unwrap();

  let file = File::open("data/Vulkan-Headers/registry/vk.xml").unwrap();
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
  let mut is_platform_specific = false;
  let mut is_deprecated = false;
  let mut is_special_use = false;
  let mut is_feature_requirement = false;
  let mut version_number = String::from("1.0");
  
  let parser = EventReader::new(file);
  for e in parser {
    match e {
        Ok(XmlEvent::StartElement { name, attributes, .. }) => {  
          match name.to_string().as_str() {
            "command" => { 
              is_command = true;
              if is_extension && (is_platform_specific || is_deprecated || is_special_use) {
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
              is_platform_specific = {
                match attributes.iter().find(|ref attr| {
                  attr.to_string().contains("platform")
                }) {
                  Some(_) => true,
                  None => false
                }
              };
              is_deprecated = {
                match attributes.iter().find(|ref attr| {
                  attr.to_string().contains("deprecated")
                }) {
                  Some(_) => true,
                  None => false
                }
              };
              is_special_use = {
                match attributes.iter().find(|ref attr| {
                  attr.to_string().contains("specialuse")
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
            if is_platform_specific || is_deprecated || is_special_use {
              commands_extensions.insert(text.clone());
            }
            if is_command && is_proto {
              allcommands.insert(text);
            }
          }
        }
        Ok(XmlEvent::EndElement { name }) => {
          match name.to_string().as_str() {
            "command" => is_command = false,
            "feature" => is_feature_requirement = false,
            "proto" => is_proto = false,
            "extension" => { is_extension = false; is_platform_specific = false; is_deprecated = false; is_special_use = false; },
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
  
  writeln!(f, "").unwrap();

  writeln!(f, "pub struct LibLoader {{").unwrap();
  for command in allcommands.clone() {
      writeln!(f, "    pub {}: PFN_{},", command, command).unwrap();
  }
  writeln!(f, "}}").unwrap();

  writeln!(f, "").unwrap();
  
  
  writeln!(f, "impl<'a> LibLoader {{").unwrap();
  writeln!(
      f,
      "   pub fn new(lib : &'a Library, version_number: &str) -> Result<LibLoader, VkResult> {{"
  ).unwrap();
  writeln!(f, "").unwrap();
  writeln!(f, "       match version_number {{").unwrap();

  writeln!(f, "         \"1.2\" => Ok(LibLoader {{").unwrap();
    for command in allcommands.clone() {
        writeln!(f, "               {}: unsafe {{ Some( lib.library.get::<PFN_{}>(\"{}\").unwrap() ) }},", command, command, command).unwrap();
    }
  writeln!(f, "         }}),").unwrap();
  writeln!(f, "         \"1.1\" => Ok(LibLoader {{").unwrap();
    for command in allcommands.clone() {
      if commands_v12.contains(command) {
        writeln!(f, "               {}: None,", command).unwrap();
      }
      else {
        writeln!(f, "               {}: unsafe {{ Some( lib.library.get::<PFN_{}>(\"{}\").unwrap() ) }},", command, command, command).unwrap();
      }
    }
  writeln!(f, "         }}),").unwrap();
  writeln!(f, "         _ => Ok(LibLoader {{").unwrap();
    for command in allcommands.clone() {
      if commands_v12.contains(command) || commands_v11.contains(command) {
        writeln!(f, "               {}: None,", command).unwrap();
      }
      else {
        writeln!(f, "               {}: unsafe {{ Some( lib.library.get::<PFN_{}>(\"{}\").unwrap() ) }},", command, command, command).unwrap();
      }
    }
  writeln!(f, "         }})").unwrap();

  writeln!(f, "       }}").unwrap();

  writeln!(f, "   }}").unwrap();
  writeln!(f, "}}").unwrap();
}