// #![windows_subsystem = "windows"]

use std::{env::current_exe, time::{Duration, Instant}};

mod gui;
mod player;
mod events;

use gui::*;
use player::*;
use events::*;
use id3::{Tag, TagLike};

// #[cfg(target_os = "linux")]
// use xosd_rs::Xosd;


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
    seek_duration: Duration,
    
    toggle_gui: Option<usize>,
    quit_app: Option<usize>,
    pause_resume_song: Option<usize>,
    stop_player: Option<usize>,
    volume_increase: Option<usize>,
    volume_decrease: Option<usize>,
    toggle_repeat_mode: Option<usize>,
    toggle_shuffling: Option<usize>,
    rewind: Option<usize>,
    fast_forward: Option<usize>,
    
    request_duration: bool,
}


impl App {
    fn run() {
        let mut app = App {
            gui: None,
            player: Player::new(),
            listener: EventListener::listen(),
            
            fps: 120.,
            delta: Duration::ZERO,
            volume_step: 0.05,
            key_duration: Duration::from_millis(100),
            seek_duration: Duration::from_secs(5),
            
            toggle_gui: None,
            quit_app: None,
            pause_resume_song: None,
            stop_player: None,
            volume_increase: None,
            volume_decrease: None,
            toggle_repeat_mode: None,
            toggle_shuffling: None,
            rewind: None,
            fast_forward: None,
            
            request_duration: false,
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
        app.rewind = Some(app.listener.register_combination([Key::Alt, Key::LeftArrow], app.key_duration * 2));
        app.fast_forward = Some(app.listener.register_combination([Key::Alt, Key::RightArrow], app.key_duration * 2));
        
        app.launch_gui();
        
        loop {
            let t = Instant::now();
            
            if app.gui_events() || app.key_events() {
                app.close_gui();
                break;
            }
            
            if app.player.is_finished() {
                match app.player.repeat_mode {
                    RepeatMode::NoRepeat => {}
                    RepeatMode::RepeatTrack => app.player.replay(),
                    RepeatMode::Repeat => todo!()
                }
            }
            
            let t = Instant::now().duration_since(t);
            
            if t < app.delta {
                spin_sleep::sleep(app.delta - t);
            }
        }
    }
    
    fn gui_events(&mut self) -> bool {
        let mut close = false;
        
        if self.request_duration && self.player.get_length() != Duration::ZERO {
            ["DURATION", &self.player.get_length().as_secs().to_string()].gui_write_if(self);
            self.request_duration = false;
        }
        
        while let Some(command) = self.gui.as_ref().and_then(GUI::read) {
            let split = command.bytes().position(|b| b == b' ').unwrap_or(command.len());
            let args = if split == command.len() { "" } else { &command[split + 1..] };
            
            match &command[..split] {
                "READTAG" => self.read_tags(args),
                "PLAY" => { self.player.play(args); self.request_duration = true },
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
                "REWIND" => self.rewind(),
                "FAST_FORWARD" => self.fast_forward(),
                "SEEK" => args.parse().map(|d| self.player.seek(Duration::from_secs_f64(d))).err().map(|e| println!("ERROR: Cannot seek {} {}", args, e)).unwrap_or(()),
                "EXIT" => close = true,
                "EXIT_ALL" => return true,
                _ => println!("GODOT: {}", command),
            }
        }
        
        if close || self.gui.as_mut().is_some_and(GUI::finished) {
            self.close_gui();
        }
        
        false
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
                return true;
            }
            
            if comb == self.pause_resume_song {
                if self.player.is_playing() {
                    ["PAUSE"].gui_write_if(self);
                    self.player.pause();
                }
                else {
                    ["RESUME"].gui_write_if(self);
                    self.player.resume();
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
            
            if comb == self.rewind {
                self.rewind();
            }
            
            if comb == self.fast_forward {
                self.fast_forward();
            }
        }
        
        false
    }
    
    #[inline(always)]
    fn gui(&mut self) -> &mut GUI {
        self.gui.as_mut().unwrap()
    }
    
    fn launch_gui(&mut self) {
        self.close_gui();
        
        let mut dir = current_exe().expect("Failed to get current directory");
        
        dir.pop();
        dir.push("godot");
        
        println!("TASK: Launching GUI");
        
        self.gui.replace(GUI::launch(dir.as_os_str()));
        ["VOLUME", &self.player.volume.to_string()].gui_write(self);
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
                
                self.gui().write_iter(&[
                    "TAGOF", path,
                    "Title", tag.title().filter(|s| !s.is_empty()).unwrap_or("No Title"),
                    "Album", tag.album().filter(|s| !s.is_empty()).unwrap_or("No Album"),
                    "Artist", &tag.artists().map(|artists| artists.join(", ")).filter(|s| !s.is_empty()).unwrap_or("No Artist".to_string()),
                    "Lyrics", tag.lyrics().find(|lyrics| lyrics.lang == "eng").map(|lyrics| lyrics.text.as_str()).filter(|s| !s.is_empty()).unwrap_or("No Lyrics"),
                ]);
            }
            Err(e) => println!("ERROR: Cannot read tag from {} ({})", path, e)
        }
    }
    
    fn volume(&mut self, target: f32, notify: bool) {
        self.player.volume = target.clamp(0., 1.);
        self.player.update_volume();
        
        if notify && self.gui.is_some() {
            ["VOLUME", &self.player.volume.to_string()].gui_write_if(self);
        }
    }
    
    fn write_repeat_mode(&mut self) {
        ["REPEAT", self.player.repeat_mode.to_str()].gui_write_if(self);
    }
    
    fn rewind(&mut self) {
        self.player.rewind(self.seek_duration);
        ["REWIND", &self.seek_duration.as_secs_f64().to_string()].gui_write_if(self);
    }
    
    fn fast_forward(&mut self) {
        self.player.fast_forward(self.seek_duration);
        ["FAST_FORWARD", &self.seek_duration.as_secs_f64().to_string()].gui_write_if(self);
    }
}

impl AsMut<GUI> for App {
    fn as_mut(&mut self) -> &mut GUI {
        self.gui()
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
