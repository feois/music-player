// #![windows_subsystem = "windows"]

use std::{env::current_exe, time::{Duration, Instant}};

mod gui;
mod player;
mod events;

use gui::*;
use player::*;
use events::*;
use id3::{Tag, TagLike};

#[cfg(target_os = "linux")]
use xosd_rs::Xosd;


const DELIMETER: &str = "::::";


// #[cfg(target_os = "windows")]
// mod testwin {
//     use native_windows_gui::*;//{dispatch_thread_events_with_callback, NativeUi, Window, Label, stop_thread_dispatch};
//     use native_windows_derive::NwgUi;
    
//     #[derive(Default, NwgUi)]
//     pub struct App {
//         #[nwg_control(size: (500, 80), position: (100, 100), title: "test", flags: "VISIBLE|WINDOW")]//, ex_flags: 524456)]
//         #[nwg_events( OnWindowClose: [App::testf] )]
//         window: Window,
        
//         #[nwg_control(position: (10, 10), size: (200, 20), text: "Test")]
//         label: Label,
//     }
    
//     impl App {
//         fn testf(&self) {
//             stop_thread_dispatch();
//         }
//     }
    
//     pub fn f() {
//         let flags = 524456; //0x00080000 | 0x00000080 | 0x00000008 | 0x00000020;
        
//         native_windows_gui::init().expect("Failed to init Windows GUI");
        
//         let _ = App::build_ui(Default::default()).expect("Failed to build UI");
        
//         loop {
//         dispatch_thread_events();//_with_callback(|| ());
//         }
//     }
// }


pub trait BooleanConditional {
    fn ifdo(self, f: impl FnOnce()) -> Self;
    fn elsedo(self, f: impl FnOnce()) -> Self;
}

impl BooleanConditional for bool {
    fn ifdo(self, f: impl FnOnce()) -> Self {
        if self {
            f();
        }
        
        self
    }
    
    fn elsedo(self, f: impl FnOnce()) -> Self {
        if !self {
            f();
        }
        
        self
    }
}

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
            
            println!("TASK: Reading tag of {}", path);
            
            write_tags(gui, "TAGOF", path);
            write_tags(gui, "Title", tag.title().unwrap_or("No Title"));
            write_tags(gui, "Album", tag.album().unwrap_or("No Album"));
            write_tags(gui, "Artist", &tag.artists().unwrap_or(vec![]).join(", "));
            write_tags(gui, "Lyrics", lyrics);
            gui.endline();
        }
        Err(e) => println!("ERROR: Cannot read tag from {} ({})", path, e)
    }
}

#[inline(always)]
fn set_volume(gui: Option<&mut GUI>, player: &Player, mut target: f32, volume: &mut f32) {
    target = target.clamp(0., 1.);
    
    *volume = target;
    
    player.volume(target);
    
    if let Some(gui) = gui {
        gui.write_line(&("VOLUME".to_string() + DELIMETER + &((target * 100.).round() as i32).to_string()));
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
    let key_duration = Duration::from_millis(100);
    
    let mut player = Player::new();
    let mut gui: Option<GUI> = None;
    let mut listener = EventListener::listen();
    let mut volume = 1f32;
    
    let toggle_gui = listener.register_once_combination(&[Key::Alt, Key::KeyC]);
    let quit_app = listener.register_once_combination(&[Key::Alt, Key::KeyE]);
    let pause_resume_song = listener.register_once_combination(&[Key::Alt, Key::Space]);
    let stop_player = listener.register_once_combination(&[Key::Alt, Key::ShiftLeft, Key::KeyM]);
    
    let volume_increase = listener.register_combination(&[Key::Alt, Key::UpArrow], key_duration);
    let volume_decrease = listener.register_combination(&[Key::Alt, Key::DownArrow], key_duration);
    
    // launch_gui(&mut gui);
    
    // let mut xosd = Xosd::new(1).unwrap();
    
    // xosd.set_color("white").unwrap();
    // xosd.set_timeout(5).unwrap();
    // xosd.set_horizontal_align(xosd_rs::HorizontalAlign::Center).unwrap();
    // xosd.set_vertical_align(xosd_rs::VerticalAlign::Top).unwrap();
    // xosd.display(0, Command::String(flags.to_string())).unwrap();
    
    // #[cfg(target_os = "windows")]
    // testwin::f();
    
    // println!("loop");
    
    'event_loop: loop {
        // break;
        
        let t = Instant::now();
        
        let mut close_gui = false;
        
        // read
        if let Some(gui) = &mut gui {
            while let Some(command) = gui.read() {
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
                    "VOLINC" => set_volume(Some(gui), &player, volume + volume_step, &mut volume),
                    "VOLDEC" => set_volume(Some(gui), &player, volume - volume_step, &mut volume),
                    "INFO" => println!("GODOT-PRINT: {}", args),
                    "EXIT" => close_gui = true,
                    _ => println!("GODOT: {}", command),
                }
            }
            
            if close_gui {
                gui.endline();
            }
        }
        
        
        // kill when finished
        if close_gui || gui.as_mut().is_some_and(GUI::finished) {
            println!("TASK: Closing GUI");
            kill_gui(&mut gui);
        }
        
        // key events
        listener.poll_events();
        
        for comb in listener.iter_pressed() {
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
                set_volume(gui.as_mut(), &player, volume + volume_step, &mut volume);
            }
            
            if comb == volume_decrease {
                set_volume(gui.as_mut(), &player, volume - volume_step, &mut volume);
            }
        }
        
        // sleep till next frame
        let t = Instant::now().duration_since(t);
        
        if t < delta {
            spin_sleep::sleep(delta - t);
        }
    }
}
