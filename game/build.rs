
use std::{env, path::{Path, PathBuf}};

fn move_pdb(path:PathBuf) {
    let locked_path = path.join("nrg_game.pdb.locked");
    let pdb_path = path.join("nrg_game.pdb");
    
    if locked_path.exists() {
        let res = ::std::fs::remove_file(locked_path.clone());
        if !res.is_ok() {
            println!("Remove {} failed", locked_path.to_str().unwrap());
        }
    }
    if pdb_path.exists() && !locked_path.exists() {
        let res = ::std::fs::rename(pdb_path.clone(), locked_path.clone());
        if !res.is_ok() {
            println!("Renamo {} to {} failed", pdb_path.to_str().unwrap(), locked_path.to_str().unwrap());
        }
    }
}

fn main() {
    let out_dir = env::current_dir().unwrap();
    let build_path = Path::new(&out_dir).join("..\\target\\debug").canonicalize().unwrap(); 
    let deps_path = Path::new(&out_dir).join("..\\target\\debug\\deps").canonicalize().unwrap(); 
    
    move_pdb(deps_path);
    move_pdb(build_path);
}