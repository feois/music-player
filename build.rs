use std::{env, path::PathBuf, process::Command, str::FromStr};


fn main() {
    println!("cargo::rerun-if-changed=src/gui/src");
    println!("cargo::rerun-if-changed=src/gui/project.godot");
    
    let mut path = PathBuf::from_str(&env::var("OUT_DIR").unwrap()).unwrap();
    
    path.pop();
    path.pop();
    path.pop();
    path.push("godot");
    
    let target = env::var("TARGET").unwrap();
    let os = target.split('-').nth(2).unwrap();
    
    Command::new(env::var("GODOT_PATH").ok().as_ref().map_or("godot", |s| s.as_str()))
        .arg("--headless")
        .args(&["--path", "src/gui"])
        .args(&["--export-release", os, path.as_os_str().to_str().unwrap()])
        .spawn()
        .expect("Cannot export godot")
        .wait()
        .unwrap();
}
