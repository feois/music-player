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


struct App {
    gui: Option<GUI>,
    player: Player,
    listener: EventListener,
    
    fps: f64,
    delta: Duration,
    volume_step: f32,
    key_duration: Duration,
    
    toggle_gui: Option<usize>,
    quit_app: Option<usize>,
    pause_resume_song: Option<usize>,
    stop_player: Option<usize>,
    volume_increase: Option<usize>,
    volume_decrease: Option<usize>,
    toggle_repeat_mode: Option<usize>,
    toggle_shuffling: Option<usize>,
}


impl App {
    fn run() {
        let mut app = App {
            gui: None,
            player: Player::new(),
            listener: EventListener::listen(),
            
            fps: 60.,
            delta: Duration::ZERO,
            volume_step: 0.05,
            key_duration: Duration::from_millis(100),
            
            toggle_gui: None,
            quit_app: None,
            pause_resume_song: None,
            stop_player: None,
            volume_increase: None,
            volume_decrease: None,
            toggle_repeat_mode: None,
            toggle_shuffling: None,
        };
        
        app.delta = Duration::from_secs_f64(1. / app.fps);
        
        app.toggle_gui = Some(app.listener.register_once_combination([Key::Alt, Key::KeyC]));
        app.quit_app = Some(app.listener.register_once_combination([Key::Alt, Key::KeyE]));
        app.pause_resume_song = Some(app.listener.register_once_combination([Key::Alt, Key::Space]));
        app.stop_player = Some(app.listener.register_once_combination([Key::Alt, Key::ShiftLeft, Key::KeyM]));
        app.toggle_repeat_mode = Some(app.listener.register_once_combination([Key::Alt, Key::KeyR]));
        app.toggle_shuffling = Some(app.listener.register_once_combination([Key::Alt, Key::ShiftLeft, Key::KeyR]));
        
        app.volume_increase = Some(app.listener.register_combination([Key::Alt, Key::UpArrow], app.key_duration));
        app.volume_decrease = Some(app.listener.register_combination([Key::Alt, Key::DownArrow], app.key_duration));
        
        app.launch_gui();
        
        loop {
            let t = Instant::now();
            
            app.gui_events();
            
            if app.key_events() {
                break;
            }
            
            let t = Instant::now().duration_since(t);
            
            if t < app.delta {
                spin_sleep::sleep(app.delta - t);
            }
        }
    }
    
    fn gui_events(&mut self) {
        let mut close = false;
        
        while let Some(command) = self.gui.as_ref().and_then(GUI::read) {
            let split = command.bytes().position(|b| b == b' ').unwrap_or(command.len());
            let args = if split == command.len() { "" } else { &command[split + 1..] };
            
            match &command[..split] {
                "READTAG" => self.read_tags(args),
                "PLAY" => self.player.play(args),
                "STOP" => self.player.stop(),
                "PAUSE" => self.player.pause(),
                "RESUME" => self.player.resume(),
                "VOLUME" => args.parse().map(|v| self.volume(v, false)).err().map(|e| println!("ERROR: Cannot parse volume {}", e)).unwrap_or(()),
                "VOLINC" => self.volume(self.player.volume + self.volume_step, true),
                "VOLDEC" => self.volume(self.player.volume - self.volume_step, true),
                "TOGGLE_REPEAT" => { self.player.toggle_repeat_mode(); self.write_repeat_mode() },
                "SHUFFLE" => self.player.shuffle = true,
                "NO_SHUFFLE" => self.player.shuffle = false,
                "INFO" => println!("GODOT-PRINT: {}", args),
                "EXIT" => close = true,
                _ => println!("GODOT: {}", command),
            }
        }
        
        if close || self.gui.as_mut().is_some_and(GUI::finished) {
            self.close_gui();
        }
    }
    
    fn key_events(&mut self) -> bool {
        self.listener.poll_events();
        
        for comb in self.listener.iter_pressed().collect::<Vec<_>>().into_iter().map(|i| Some(i)) {
            if comb == self.toggle_gui {
                if self.gui.is_none() {
                    self.launch_gui();
                }
                else {
                    self.close_gui();
                }
            }
            
            if comb == self.quit_app {
                self.close_gui();
                return true;
            }
            
            if comb == self.pause_resume_song {
                if self.player.is_paused() {
                    ["RESUME"].gui_write_if(self);
                    self.player.resume();
                }
                else {
                    ["PAUSE"].gui_write_if(self);
                    self.player.pause();
                }
            }
            
            if comb == self.stop_player {
                ["STOP"].gui_write_if(self);
                self.player.stop();
            }
            
            if comb == self.volume_increase {
                self.volume(self.player.volume + self.volume_step, true);
            }
            
            if comb == self.volume_decrease {
                self.volume(self.player.volume - self.volume_step, true);
            }
            
            if comb == self.toggle_repeat_mode {
                self.player.toggle_repeat_mode();
                self.write_repeat_mode();
            }
            
            if comb == self.toggle_shuffling {
                self.player.shuffle = !self.player.shuffle;
                [if self.player.shuffle { "SHUFFLE" } else { "NO_SHUFFLE" }].gui_write_if(self);
            }
        }
        
        false
    }
    
    #[inline(always)]
    fn gui(&self) -> &GUI {
        self.gui.as_ref().unwrap()
    }
    
    #[inline(always)]
    fn mgui(&mut self) -> &mut GUI {
        self.gui.as_mut().unwrap()
    }
    
    fn launch_gui(&mut self) {
        self.close_gui();
        
        let mut dir = current_exe().expect("Failed to get current directory");
        
        dir.pop();
        dir.push("godot");
        
        println!("TASK: Launching GUI");
        
        self.gui.replace(GUI::launch(dir.as_os_str()));
        ["VOLUME", &self.rounded_volume().to_string()].gui_write(self);
        self.write_repeat_mode();
        [if self.player.shuffle { "SHUFFLE" } else { "NO_SHUFFLE" }].gui_write_if(self);
    }
    
    #[inline(always)]
    fn close_gui(&mut self) {
        self.gui.take().map(GUI::close);
    }
    
    fn read_tags(&mut self, path: &str) {
        match Tag::read_from_path(path) {
            Ok(tag) => {
                println!("TASK: Reading tag of {}", path);
                
                self.mgui().write_iter(&[
                    "TAGOF", path,
                    "Title", tag.title().unwrap_or("No Title"),
                    "Album", tag.album().unwrap_or("No Album"),
                    "Artist", &tag.artists().map_or("No Artist".to_string(), |artists| artists.join(", ")),
                    "Lyrics", tag.lyrics().find(|lyrics| lyrics.lang == "eng").map_or("No Lyrics", |lyrics| &lyrics.text),
                    "Duration", &mp3_duration::from_path(path).expect("Failed to read duration").as_secs().to_string(),
                ]);
            }
            Err(e) => println!("ERROR: Cannot read tag from {} ({})", path, e)
        }
    }
    
    fn volume(&mut self, target: f32, notify: bool) {
        self.player.volume = target.clamp(0., 1.);
        self.player.update_volume();
        
        if notify && self.gui.is_some() {
            ["VOLUME", &self.rounded_volume().to_string()].gui_write_if(self);
        }
    }
    
    fn rounded_volume(&self) -> i32 {
        (self.player.volume * 100.).round() as i32
    }
    
    fn write_repeat_mode(&mut self) {
        ["REPEAT", self.player.repeat_mode.to_str()].gui_write_if(self);
    }
}

impl AsMut<GUI> for App {
    fn as_mut(&mut self) -> &mut GUI {
        self.mgui()
    }
}

impl AsMut<Option<GUI>> for App {
    fn as_mut(&mut self) -> &mut Option<GUI> {
        &mut self.gui
    }
}

fn main() {
    // let mut xosd = Xosd::new(1).unwrap();
    
    // xosd.set_color("white").unwrap();
    // xosd.set_timeout(5).unwrap();
    // xosd.set_horizontal_align(xosd_rs::HorizontalAlign::Center).unwrap();
    // xosd.set_vertical_align(xosd_rs::VerticalAlign::Top).unwrap();
    // xosd.display(0, Command::String(flags.to_string())).unwrap();
    
    App::run();
}
