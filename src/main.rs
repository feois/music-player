// #![windows_subsystem = "windows"]

use std::{env::current_exe, ffi::{OsStr, OsString}, fs::{create_dir_all, read_to_string, write}, path::PathBuf, time::{Duration, Instant}};

mod gui;
mod player;
mod events;
mod history;
mod playlist;

use gui::*;
use notify_rust::Notification;
use player::*;
use playlist::*;
use events::*;
use id3::{Tag, TagLike};
use serde_json::{from_str, to_string_pretty};

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


#[inline(always)]
fn arg(arg_name: &str, argument: impl AsRef<OsStr>) -> OsString {
    let mut s = OsString::from(arg_name);
    
    s.push(argument);
    
    s
}


#[inline(always)]
fn show_notification(content: impl AsRef<str>) {
    if let Err(e) = Notification::new().body(content.as_ref()).show() {
        println!("RUST-ERROR: Failed to show notification {}", e);
    }
}


struct App {
    gui: Option<GUI>,
    player: Player,
    playlist: Playlist,
    listener: EventListener,
    
    fps: f64,
    delta: Duration,
    volume_step: f32,
    key_duration: Duration,
    seek_duration: Duration,
    cache_path: PathBuf,
    
    toggle_gui: Option<usize>,
    quit_app: Option<usize>,
    pause_resume_song: Option<usize>,
    stop_player: Option<usize>,
    volume_increase: Option<usize>,
    volume_decrease: Option<usize>,
    toggle_mute: Option<usize>,
    toggle_repeat_mode: Option<usize>,
    toggle_shuffling: Option<usize>,
    toggle_stop_next: Option<usize>,
    rewind: Option<usize>,
    fast_forward: Option<usize>,
    
    request_duration: bool,
    stop_next: bool,
}


impl App {
    fn run() {
        let mut app = App {
            gui: None,
            playlist: Playlist::new(100),
            player: Player::new(),
            listener: EventListener::listen(),
            
            fps: 120.,
            delta: Duration::ZERO,
            volume_step: 0.05,
            key_duration: Duration::from_millis(100),
            seek_duration: Duration::from_secs(5),
            cache_path: {
                let mut path = dirs::cache_dir().unwrap();
                
                path.push("feois-music-player");
                
                if !path.is_dir() {
                    create_dir_all(&path).expect("Failed to create cache directory");
                }
                
                path
            },
            
            toggle_gui: None,
            quit_app: None,
            pause_resume_song: None,
            stop_player: None,
            volume_increase: None,
            volume_decrease: None,
            toggle_mute: None,
            toggle_repeat_mode: None,
            toggle_shuffling: None,
            toggle_stop_next: None,
            rewind: None,
            fast_forward: None,
            
            request_duration: false,
            stop_next: false,
        };
        
        let playlist_cache_path = app.cache_path.as_path().join("playlist.json");
        let player_cache_path = app.cache_path.as_path().join("player.json");
        
        if let Some(playlist) = read_to_string(&playlist_cache_path).ok().and_then(|s| from_str(&s).ok()) {
            app.playlist = playlist;
            unsafe { app.playlist.history_keep_at_most(100) };
        }
        
        if let Some(player) = read_to_string(&player_cache_path).ok().and_then(|s| from_str(&s).ok()) {
            app.player = player;
        }
        
        app.delta = Duration::from_secs_f64(1. / app.fps);
        
        app.toggle_gui = Some(app.listener.register_once_combination([Key::Alt, Key::KeyC]));
        app.quit_app = Some(app.listener.register_once_combination([Key::Alt, Key::KeyE]));
        app.pause_resume_song = Some(app.listener.register_once_combination([Key::Alt, Key::Space]));
        app.stop_player = Some(app.listener.register_once_combination([Key::Alt, Key::ShiftLeft, Key::Space]));
        app.toggle_repeat_mode = Some(app.listener.register_once_combination([Key::Alt, Key::KeyR]));
        app.toggle_shuffling = Some(app.listener.register_once_combination([Key::Alt, Key::ShiftLeft, Key::KeyR]));
        app.toggle_mute = Some(app.listener.register_once_combination([Key::Alt, Key::KeyM]));
        app.toggle_stop_next = Some(app.listener.register_once_combination([Key::Alt, Key::ShiftLeft, Key::KeyM]));
        
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
            
            let t = Instant::now().duration_since(t);
            
            if t < app.delta {
                spin_sleep::sleep(app.delta - t);
            }
        }
        
        write(playlist_cache_path, to_string_pretty(&app.playlist).expect("Failed to serialize")).expect("Failed to save cache");
        write(player_cache_path, to_string_pretty(&app.player).expect("Failed to serialize")).expect("Failed to save cache");
    }
    
    fn gui_events(&mut self) -> bool {
        let mut close = false;
        
        self.player.update_state();
        
        if let PlayerState::Finished = self.player.get_state() {
            self.player.idle();
            
            if self.stop_next {
                self.stop_next = false;
                
                println!("STATUS: Idle");
            }
            else {
                self.poll();
            }
        }
        
        if self.request_duration && self.player.get_length() != Duration::ZERO {
            ["DURATION", &self.player.get_length().as_secs().to_string()].gui_write_if(self);
            self.request_duration = false;
        }
        
        while let Some(command) = self.gui.as_ref().and_then(GUI::read) {
            let (command_name, args) = command.split_once(' ').unwrap_or((&command, ""));
            
            match command_name {
                "READTAG" => self.read_tags(args),
                "PLAY" => { self.player.play(self.playlist.select(args.parse().unwrap())); self.request_duration = true }
                "STOP" => self.player.stop(),
                "REPLAY" => if let Some(song) = self.playlist.get_history().get_current().cloned() { self.play(&song) }
                "PREV" => if let Some(song) = self.playlist.look_back() { self.play(&song); }
                "SKIP" => self.player.skip(),
                "PAUSE" => self.player.pause(),
                "RESUME" => self.player.resume(),
                "MUTE" => { self.player.mute = true; self.player.update_volume() }
                "UNMUTE" => { self.player.mute = false; self.player.update_volume() }
                "VOLUME" => if let Err(e) = args.parse().map(|v| self.volume(v, false)) { println!("RUST-ERROR: Cannot parse volume {}", e) }
                "VOLINC" => self.volume(self.player.volume + self.volume_step, true),
                "VOLDEC" => self.volume(self.player.volume - self.volume_step, true),
                "APPEND" => self.playlist.append(args.to_string()),
                "UPDATE" => { let (i, path) = args.split_once(' ').unwrap(); self.playlist.update(i.parse().unwrap(), path.to_string()) }
                "MOVE" => { let (from, to) = args.split_once(' ').unwrap(); self.playlist.arrange(from.parse().unwrap(), to.parse().unwrap()) }
                "DELETE" => self.playlist.delete(args.parse().unwrap()),
                "DELETE_ALL" => self.playlist.clear(),
                "TOGGLE_REPEAT" => { self.playlist.toggle_repeat_mode(); self.write_repeat_mode() },
                "SHUFFLE" => self.playlist.shuffle = true,
                "NO_SHUFFLE" => self.playlist.shuffle = false,
                "INFO" => println!("GODOT-PRINT: {}", args),
                "REWIND" => self.rewind(),
                "FAST_FORWARD" => self.fast_forward(),
                "SEEK" => if let Err(e) = args.parse().map(|d| self.player.seek(Duration::from_secs_f64(d))) { println!("RUST-ERROR: Cannot seek {} {}", args, e) }
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
                match self.player.get_state() {
                    PlayerState::Play => {
                        ["PAUSE"].gui_write_if(self);
                        self.player.pause();
                    }
                    PlayerState::Pause => {
                        ["RESUME"].gui_write_if(self);
                        self.player.resume();
                    }
                    _ => {}
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
            
            if comb == self.toggle_mute {
                self.player.mute = !self.player.mute;
                self.player.update_volume();
                [if self.player.mute { "MUTE" } else { "UNMUTE" }].gui_write_if(self);
            }
            
            if comb == self.toggle_repeat_mode {
                self.playlist.toggle_repeat_mode();
                self.write_repeat_mode();
                
                show_notification(format!("Repeat mode: {}", self.playlist.get_repeat_mode().describe()));
            }
            
            if comb == self.toggle_shuffling {
                self.playlist.shuffle = !self.playlist.shuffle;
                [if self.playlist.shuffle { "SHUFFLE" } else { "NO_SHUFFLE" }].gui_write_if(self);
                
                show_notification(if self.playlist.shuffle { "Shuffle: true" } else { "Shuffle: false" });
            }
            
            if comb == self.toggle_stop_next {
                self.stop_next = !self.stop_next;
                
                show_notification(if self.stop_next { "Stop after song finished: true" } else { "Stop after song finished: false" });
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
        
        let mut args = vec![arg("--cache-path=", &self.cache_path)];
        
        match self.player.get_state() {
            PlayerState::Play | PlayerState::Pause => {
                if let PlayerState::Pause = self.player.get_state() {
                    args.push(OsString::from("--paused"))
                }
                
                args.push(arg("--song-path=", self.playlist.get_history().get_current().unwrap()));
                args.push(arg("--song-duration=", self.player.get_length().as_secs_f64().to_string()));
                args.push(arg("--song-position=", self.player.get_position().as_secs_f64().to_string()));
            }
            _ => {
                if let Some(song) = self.playlist.get_history().get_current().cloned() {
                    args.push(arg("--last-song=", song))
                }
            }
        }
        
        println!("TASK: Launching GUI");
        
        self.gui.replace(GUI::launch(dir.as_os_str(), args));
        ["VOLUME", &self.player.volume.to_string()].gui_write(self);
        self.write_repeat_mode();
        [if self.playlist.shuffle { "SHUFFLE" } else { "NO_SHUFFLE" }].gui_write_if(self);
        [if self.player.mute { "MUTE" } else { "UNMUTE" }].gui_write_if(self);
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
            Err(e) => println!("RUST-ERROR: Cannot read tag from {} ({})", path, e)
        }
    }
    
    #[inline(always)]
    fn volume(&mut self, target: f32, notify: bool) {
        self.player.volume = target.clamp(0., 1.);
        self.player.update_volume();
        
        if notify && self.gui.is_some() {
            ["VOLUME", &self.player.volume.to_string()].gui_write_if(self);
        }
    }
    
    #[inline(always)]
    fn write_repeat_mode(&mut self) {
        ["REPEAT", &self.playlist.get_repeat_mode().get_string()].gui_write_if(self);
    }
    
    #[inline(always)]
    fn rewind(&mut self) {
        self.player.rewind(self.seek_duration);
        ["REWIND", &self.seek_duration.as_secs_f64().to_string()].gui_write_if(self);
    }
    
    #[inline(always)]
    fn fast_forward(&mut self) {
        self.player.fast_forward(self.seek_duration);
        ["FAST_FORWARD", &self.seek_duration.as_secs_f64().to_string()].gui_write_if(self);
    }
    
    #[inline(always)]
    fn play(&mut self, song: &str) {
        self.player.play(song);
        ["PLAY", song].gui_write_if(self);
        self.request_duration = true;
    }
    
    #[inline(always)]
    fn poll(&mut self) {
        if let Some(song) = self.playlist.poll().map(str::to_string) {
            self.play(&song);
        }
        else {
            println!("STATUS: Idle");
        }
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
    #[cfg(target_os = "linux")]
    match Xosd::new(1) {
        Ok(mut xosd) => {
            xosd.set_color("white").unwrap();
            xosd.set_timeout(5).unwrap();
            xosd.set_horizontal_align(xosd_rs::HorizontalAlign::Center).unwrap();
            xosd.set_vertical_align(xosd_rs::VerticalAlign::Top).unwrap();
            xosd.display(0, xosd_rs::Command::String("test".to_string())).unwrap();
        }
        Err(e) => println!("RUST-ERROR: Failed to initialize {}", e)
    }
    
    App::run();
}
