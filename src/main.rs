use std::time::{Duration, Instant};

mod gui;
mod player;

use gui::GUI;
use player::Player;

fn main() {
    assert!(single_instance::SingleInstance::new("Music player").is_ok_and(|i| i.is_single()));
    
    let fps = 60.;
    let delta = Duration::from_secs_f64(1. / fps);
    
    let player = Player::new();
    let mut gui = None;
    
    gui.replace(GUI::launch(String::from("./godot.x86_64"))).map(GUI::kill);
    
    loop {
        let t = Instant::now();
        
        let mut exit = false;
        let mut commands = vec![];
        
        // read
        if let Some(gui) = &mut gui {
            while let Some(b) = gui.read() {
                for s in b.split('\n') {
                    if s == "EXIT" {
                        exit = true;
                    }
                    else {
                        commands.push(s.to_string());
                    }
                }
            }
            
            if exit {
                gui.endline();
                gui.flush();
            }
        }
        
        // process commands
        if !commands.is_empty() {
            println!("{:?}", commands);
        }
        
        // kill when finished
        if exit || gui.as_mut().is_some_and(GUI::finished) {
            gui.take().map(GUI::kill);
        }
        
        // sleep till next frame
        let t = Instant::now().duration_since(t);
        
        if t < delta {
            spin_sleep::sleep(delta - t);
        }
    }
}
