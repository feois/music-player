use std::{env::current_exe, time::{Duration, Instant}};

mod gui;
mod player;
mod events;

use gui::*;
use player::*;
use events::*;
use id3::{Tag, TagLike};

const DELIMETER: &str = "::::";

#[inline(always)]
fn write_tags(gui: &mut GUI, tag: &str, content: &str) {
    gui.write(tag);
    gui.write(DELIMETER);
    gui.write(content);
    gui.write(DELIMETER);
}

#[inline(always)]
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

#[inline(always)]
fn set_volume(gui: &mut Option<GUI>, player: &Player, mut target: f32, volume: &mut f32) {
    target = target.clamp(0., 1.);
    
    *volume = target;
    
    player.volume(target);
    
    if let Some(gui) = gui {
        gui.write_line(&("VOLUME".to_string() + DELIMETER + &(target as i32).to_string()));
    }
}

#[inline(always)]
fn launch_gui(gui: &mut Option<GUI>) {
    let mut dir = current_exe().expect("Failed to get current directory");
    
    dir.pop();
    dir.push("godot");
    
    gui.replace(GUI::launch(dir.as_os_str().to_str().unwrap())).map(GUI::kill);
}

#[inline(always)]
fn kill_gui(gui: &mut Option<GUI>) {
    gui.take().map(GUI::kill);
}

fn main() {
    let fps = 60.;
    let delta = Duration::from_secs_f64(1. / fps);
    let volume_step = 0.05;
    let key_duration = Duration::from_millis(500);
    
    let mut player = Player::new();
    let mut gui: Option<GUI> = None;
    let mut listener = EventListener::listen();
    let mut volume = 1f32;
    
    let toggle_gui = listener.register_once_combination(&[Key::Alt, Key::KeyC]);
    let quit_app = listener.register_once_combination(&[Key::Alt, Key::KeyE]);
    let pause_resume_song = listener.register_once_combination(&[Key::Alt, Key::Space]);
    let stop_player = listener.register_once_combination(&[Key::Alt, Key::ShiftLeft, Key::KeyM]);
    
    let volume_increase = listener.register_combination(&[Key::Alt, Key::ShiftLeft, Key::UpArrow], key_duration);
    let volume_decrease = listener.register_combination(&[Key::Alt, Key::ShiftLeft, Key::DownArrow], key_duration);
    
    launch_gui(&mut gui);
    
    'event_loop: loop {
        let t = Instant::now();
        
        let mut exit = false;
        
        // read
        if let Some(gui) = &mut gui {
            while let Some(s) = gui.read() {
                for command in s.split('\n').filter(|s| !s.is_empty()) {
                    let split = command.bytes().position(|b| b == b' ').unwrap_or(command.len());
                    let args = if split == command.len() { "" } else { &command[split + 1..] };
                    
                    match &command[..split] {
                        "READTAG" => read_tags(gui, args),
                        "PLAY" => player.play(args),
                        "STOP" => player.stop(),
                        "PAUSE" => player.pause(),
                        "RESUME" => player.resume(),
                        "VOLUME" => {
                            let Ok(v) = args.parse() else { continue; };
                            
                            volume = v;
                            player.volume(volume);
                        }
                        "EXIT" => exit = true,
                        _ => println!("GODOT: {}", command),
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
            kill_gui(&mut gui);
        }
        
        // key events
        listener.poll_and_register_events();
        
        for comb in listener.consume_all() {
            if comb == toggle_gui {
                if gui.is_none() {
                    launch_gui(&mut gui);
                }
                else {
                    kill_gui(&mut gui);
                }
            }
            
            if comb == quit_app {
                kill_gui(&mut gui);
                
                break 'event_loop;
            }
            
            if comb == pause_resume_song {
                if player.is_paused() {
                    player.resume();
                }
                else {
                    player.pause();
                }
            }
            
            if comb == stop_player {
                player.stop();
            }
            
            if comb == volume_increase {
                set_volume(&mut gui, &player, volume + volume_step, &mut volume);
            }
            
            if comb == volume_decrease {
                set_volume(&mut gui, &player, volume - volume_step, &mut volume);
            }
        }
        
        // sleep till next frame
        let t = Instant::now().duration_since(t);
        
        if t < delta {
            spin_sleep::sleep(delta - t);
        }
    }
}
