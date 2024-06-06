use std::{env, path::Path, process::Command};


// godot --headless --path path_to_your_project --export-release my_export_preset_name game.exe

fn main() {
    println!("cargo::rerun-if-changed=src/gui");
    
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let path = Path::new(&dir).join("target").join(profile).join("godot");
    
    let target = env::var("TARGET").unwrap();
    let os = target.split('-').nth(2).unwrap();
    let preset = match os {
        "linux" => "Linux/X11",
        "windows" => todo!(),
        _ => unimplemented!(),
    };
    
    Command::new("godot")
        .arg("--headless")
        .args(&["--path", "src/gui"])
        .args(&["--export-release", preset, path.as_os_str().to_str().unwrap()])
        .spawn()
        .expect("Cannot export godot")
        .wait()
        .unwrap();
}
