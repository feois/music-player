use std::time::{Duration, Instant};

mod gui;
mod player;
mod events;

use gui::*;
use player::*;
use events::*;
use id3::{Tag, TagLike};

const DELIMETER: &str = "::::";

fn write_tags(gui: &mut GUI, tag: &str, content: &str) {
    gui.write(tag);
    gui.write(DELIMETER);
    gui.write(content);
    gui.write(DELIMETER);
}

fn read_tags(gui: &mut GUI, path: &str) {
    match Tag::read_from_path(path) {
        Ok(tag) => {
            let lyrics = tag.lyrics().find(|lyrics| lyrics.lang == "eng").map_or("None", |lyrics| &lyrics.text);
            let synced = tag.synchronised_lyrics().find(|lyrics| lyrics.lang == "eng");
            
            println!("TASK: Reading tag of {}", path);
            
            write_tags(gui, "TAGOF", path);
            write_tags(gui, "Title", tag.title().unwrap_or("No Title"));
            write_tags(gui, "Album", tag.album().unwrap_or("No Album"));
            write_tags(gui, "Artist", &tag.artists().unwrap_or(vec![]).join(", "));
            write_tags(gui, "Lyrics", lyrics);
            // write_tags(gui, "Synced", );
            gui.endline();
            gui.flush();
        }
        Err(e) => {
            println!("ERROR: Cannot read tag from {} {}", path, e);
        }
    }
}

fn main() {
    assert!(single_instance::SingleInstance::new("Music player").is_ok_and(|i| i.is_single()));
    
    let fps = 60.;
    let delta = Duration::from_secs_f64(1. / fps);
    let path = "./godot.x86_64";
    
    let mut player = Player::new();
    let mut gui: Option<GUI> = None;
    let mut listener = EventListener::listen();
    
    let toggle_gui = listener.register_combination(&[Key::Alt, Key::KeyC]);
    let quit_app = listener.register_combination(&[Key::Alt, Key::KeyE]);
    let pause_song = listener.register_combination(&[Key::Alt, Key::Space]);
    
    gui.replace(GUI::launch(path)).map(GUI::kill);
    
    'event_loop: loop {
        let t = Instant::now();
        
        let mut exit = false;
        
        // read
        if let Some(gui) = &mut gui {
            while let Some(s) = gui.read() {
                for command in s.split('\n') {
                    if command == "EXIT" {
                        exit = true;
                    }
                    else {
                        let Some(split) = command.bytes().position(|b| b == b' ') else { continue; };
                        let args = &command[split + 1..];
                        
                        match &command[..split] {
                            "READTAG" => read_tags(gui, args),
                            "PLAY" => player.play(args),
                            "STOP" => player.stop(),
                            "PAUSE" => player.pause(),
                            "RESUME" => player.resume(),
                            name => println!("ERROR: Unknown command {}", name),
                        }
                    }
                }
            }
            
            if exit {
                gui.endline();
                gui.flush();
            }
        }
        
        // kill when finished
        if exit || gui.as_mut().is_some_and(GUI::finished) {
            println!("TASK: Closing GUI");
            gui.take().map(GUI::kill);
        }
        
        listener.poll_and_register_events();
        
        for comb in listener.consume_all() {
            if comb == toggle_gui {
                if gui.is_none() {
                    gui.replace(GUI::launch(path));
                }
                else {
                    gui.take().map(GUI::kill);
                }
            }
            
            if comb == quit_app {
                gui.take().map(GUI::kill);
                
                break 'event_loop;
            }
            
            if comb == pause_song {
                if player.is_paused() {
                    player.resume();
                }
                else {
                    player.pause();
                }
            }
        }
        
        // sleep till next frame
        let t = Instant::now().duration_since(t);
        
        if t < delta {
            spin_sleep::sleep(delta - t);
        }
    }
}
