#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use std::{env::current_exe, ffi::OsString, fs::{create_dir_all, read_to_string, write}, path::PathBuf, time::{Duration, Instant}};

mod gui;
mod player;
mod events;
mod history;
mod playlist;
mod lyrics;
mod metadata;

use gui::*;
use player::*;
use playlist::*;
use events::*;
use lyrics::*;
use metadata::*;

use serde_json::{from_str, to_string_pretty};
use notify_rust::Notification;
use fslock::LockFile;


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


macro_rules! arg {
    ($s:literal $e:expr) => {
        {
            let mut s = OsString::from(concat!("--", $s, "="));
            
            s.push($e);
            
            s
        }
    };
}


#[macro_export]
macro_rules! error {
    ($s:literal $(, $var:expr)*) => {
        eprintln!(concat!("RUST-ERROR: ", $s) $(, $var)*)
    };
    
    ($e:expr, $s:literal $(,$var:expr)*) => {
        eprintln!(concat!("RUST-ERROR: ", $s, " {}") $(,$var)*, $e)
    };
}

#[macro_export]
macro_rules! status {
    ($s:literal $(, $e:expr)*) => {
        println!(concat!("STATUS: ", $s) $(, $e)*)
    };
}

#[macro_export]
macro_rules! task {
    ($s:literal $(, $e:expr)*) => {
        println!(concat!("TASK: ", $s) $(, $e)*)
    };
}


#[inline(always)]
fn show_notification(content: impl AsRef<str>) {
    if let Err(e) = Notification::new().body(content.as_ref()).show() {
        error!(e, "Failed to show notification");
    }
}


struct App {
    gui: Option<GUI>,
    player: Player,
    playlist: Playlist,
    listener: EventListener,
    lyrics: Option<Lyrics>,
    
    fps: u16,
    delta: Duration,
    volume_step: f32,
    key_duration: Duration,
    seek_duration: Duration,
    lyrics_layout: LyricsLayout,
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
    jump_to_begin: Option<usize>,
    jump_to_end: Option<usize>,
    prev_song: Option<usize>,
    toggle_lyrics_visibility: Option<usize>,
    
    lyrics_top_left: Option<usize>,
    lyrics_top_center: Option<usize>,
    lyrics_top_right: Option<usize>,
    lyrics_center_left: Option<usize>,
    lyrics_center: Option<usize>,
    lyrics_center_right: Option<usize>,
    lyrics_bottom_left: Option<usize>,
    lyrics_bottom_center: Option<usize>,
    lyrics_bottom_right: Option<usize>,
    
    request_duration: bool,
    stop_next: bool,
}


impl App {
    fn run() {
        let mut cache_path = dirs::cache_dir().unwrap();
                
        cache_path.push("feois-music-player");
        
        if !cache_path.is_dir() {
            create_dir_all(&cache_path).expect("Failed to create cache directory");
        }
        
        let mut lock = LockFile::open(cache_path.as_path().join("instance.lock").as_os_str()).expect("Failed to create lock");
        
        if !lock.try_lock_with_pid().expect("Failed to lock file") {
            return;
        }
        
        let mut app = App {
            gui: None,
            playlist: Playlist::new(100),
            player: Player::new(),
            listener: EventListener::listen(),
            lyrics: None,
            
            fps: 120,
            delta: Duration::ZERO,
            volume_step: 0.05,
            key_duration: Duration::from_millis(100),
            seek_duration: Duration::from_secs(5),
            lyrics_layout: LyricsLayout { position: LyricsPosition::TopCenter, margin: 48, visible: true },
            cache_path,
            
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
            jump_to_begin: None,
            jump_to_end: None,
            prev_song: None,
            toggle_lyrics_visibility: None,
            
            lyrics_top_left: None,
            lyrics_top_center: None,
            lyrics_top_right: None,
            lyrics_center_left: None,
            lyrics_center: None,
            lyrics_center_right: None,
            lyrics_bottom_left: None,
            lyrics_bottom_center: None,
            lyrics_bottom_right: None,
            
            request_duration: false,
            stop_next: false,
        };
        
        let playlist_cache_path = app.cache_path.as_path().join("playlist.json");
        let player_cache_path = app.cache_path.as_path().join("player.json");
        let lyrics_cache_path = app.cache_path.as_path().join("lyrics.json");
        
        if let Some(playlist) = read_to_string(&playlist_cache_path).ok().and_then(|s| from_str(&s).ok()) {
            app.playlist = playlist;
            unsafe { app.playlist.history_keep_at_most(100) };
        }
        
        if let Some(player) = read_to_string(&player_cache_path).ok().and_then(|s| from_str(&s).ok()) {
            app.player = player;
        }
        
        if let Some(lp) = read_to_string(&lyrics_cache_path).ok().and_then(|s| from_str(&s).ok()) {
            app.lyrics_layout = lp;
        }
        
        match Lyrics::new(app.lyrics_layout) {
            Ok(lyrics) => { app.lyrics.replace(lyrics); }
            Err(e) => error!(e, "Failed to initialize floating lyrics")
        }
        
        app.delta = Duration::from_secs_f64(1. / app.fps as f64);
        
        let mut regonce = |keys: &[Key]| Some(app.listener.register_once_combination(keys));
        
        app.toggle_gui                  = regonce(&[Key::Alt, Key::KeyC]);
        app.quit_app                    = regonce(&[Key::Alt, Key::KeyE]);
        app.pause_resume_song           = regonce(&[Key::Alt, Key::Space]);
        app.stop_player                 = regonce(&[Key::Alt, Key::ShiftLeft, Key::Space]);
        app.toggle_repeat_mode          = regonce(&[Key::Alt, Key::KeyR]);
        app.toggle_shuffling            = regonce(&[Key::Alt, Key::ShiftLeft, Key::KeyR]);
        app.toggle_mute                 = regonce(&[Key::Alt, Key::KeyM]);
        app.toggle_stop_next            = regonce(&[Key::Alt, Key::ShiftLeft, Key::KeyM]);
        app.jump_to_begin               = regonce(&[Key::Alt, Key::ControlLeft, Key::LeftArrow]);
        app.jump_to_end                 = regonce(&[Key::Alt, Key::ControlLeft, Key::RightArrow]);
        app.prev_song                   = regonce(&[Key::Alt, Key::ControlLeft, Key::UpArrow]);
        app.toggle_lyrics_visibility    = regonce(&[Key::Alt, Key::KeyH]);
        
        app.lyrics_top_left         = regonce(&[Key::Alt, Key::KeyL, Key::Num1]);
        app.lyrics_top_center       = regonce(&[Key::Alt, Key::KeyL, Key::Num2]);
        app.lyrics_top_right        = regonce(&[Key::Alt, Key::KeyL, Key::Num3]);
        app.lyrics_center_left      = regonce(&[Key::Alt, Key::KeyL, Key::Num4]);
        app.lyrics_center           = regonce(&[Key::Alt, Key::KeyL, Key::Num5]);
        app.lyrics_center_right     = regonce(&[Key::Alt, Key::KeyL, Key::Num6]);
        app.lyrics_bottom_left      = regonce(&[Key::Alt, Key::KeyL, Key::Num7]);
        app.lyrics_bottom_center    = regonce(&[Key::Alt, Key::KeyL, Key::Num8]);
        app.lyrics_bottom_right     = regonce(&[Key::Alt, Key::KeyL, Key::Num9]);
        
        let mut reg = |keys: &[Key], d| Some(app.listener.register_combination(keys, d));
        
        app.volume_increase = reg(&[Key::Alt, Key::UpArrow], app.key_duration);
        app.volume_decrease = reg(&[Key::Alt, Key::DownArrow], app.key_duration);
        app.rewind          = reg(&[Key::Alt, Key::LeftArrow], app.key_duration * 2);
        app.fast_forward    = reg(&[Key::Alt, Key::RightArrow], app.key_duration * 2);
        
        app.launch_gui();
        
        loop {
            let t = Instant::now();
            
            app.player.update_state();
            
            match app.player.get_state() {
                PlayerState::Play => app.update_lyrics(app.player.get_position()),
                PlayerState::Finished => {
                    app.player.idle();
                    
                    let song = app.playlist.poll();
                    
                    if app.stop_next || song.map(str::to_string).map(|s| app.play(s)).is_none()  {
                        app.stop_next = false;
                        app.gui(GUICommand::STOP);
                        
                        status!("Idle");
                        
                        app.clear_lyrics();
                    }
                }
                _ => {}
            }
            
            if app.gui_events() || app.key_events() {
                app.close_gui();
                break;
            }
            
            if let Some(lyrics) = &mut app.lyrics {
                if let Err(e) = lyrics.refresh() {
                    error!(e, "Failed to refresh lyrics");
                }
                
                if let Err(e) = lyrics.set_layout(app.lyrics_layout) {
                    error!(e, "Failed to reposition lyrics");
                }
            }
            
            
            let t = t.elapsed();
            
            if t < app.delta {
                spin_sleep::sleep(app.delta - t);
            }
        }
        
        write(playlist_cache_path, to_string_pretty(&app.playlist).expect("Failed to serialize")).expect("Failed to save cache");
        write(player_cache_path, to_string_pretty(&app.player).expect("Failed to serialize")).expect("Failed to save cache");
        write(lyrics_cache_path, to_string_pretty(&app.lyrics_layout).expect("Failed to serialize")).expect("Failed to save cache");
        
        status!("Exiting");
        
        lock.unlock().expect("Failed to unlock");
    }
    
    fn gui_events(&mut self) -> bool {
        let mut close = false;
        
        if self.request_duration && self.player.get_length() != Duration::ZERO {
            self.gui(GUICommand::DURATION(self.player.get_length()));
            self.request_duration = false;
        }
        
        while let Some(command) = self.gui.as_ref().and_then(GUI::read) {
            let (command_name, args) = command.split_once(' ').unwrap_or((&command, ""));
            
            match command_name {
                "MARGIN" => self.lyrics_layout.margin = args.parse().unwrap(),
                "READTAG" => self.read_tags(args),
                "PLAY" => {
                    let song = &self.playlist.select(args.parse().unwrap()).to_string();
                    
                    self.player.play(song);
                    self.request_duration = true;
                    self.show_lyrics(song);
                }
                "STOP" => { self.player.stop(); self.clear_lyrics() }
                "REPLAY" => if let Some(song) = self.playlist.get_history().get_current().cloned() { self.play(song) }
                "PREV" => if let Some(song) = self.playlist.look_back() { self.play(song); }
                "SKIP" => self.player.skip(),
                "PAUSE" => self.player.pause(),
                "RESUME" => self.player.resume(),
                "MUTE" => { self.player.mute = true; self.player.update_volume() }
                "UNMUTE" => { self.player.mute = false; self.player.update_volume() }
                "VOLUME" => if let Err(e) = args.parse().map(|v| self.volume(v, false)) { error!(e, "Cannot parse volume") }
                "VOLINC" => self.volume(self.player.volume + self.volume_step, true),
                "VOLDEC" => self.volume(self.player.volume - self.volume_step, true),
                "APPEND" => self.playlist.append(args.to_string()),
                "UPDATE" => { let (i, path) = args.split_once(' ').unwrap(); self.playlist.update(i.parse().unwrap(), path.to_string()) }
                "MOVE" => { let (from, to) = args.split_once(' ').unwrap(); self.playlist.arrange(from.parse().unwrap(), to.parse().unwrap()) }
                "DELETE" => self.playlist.delete(args.parse().unwrap()),
                "DELETE_ALL" => self.playlist.clear(),
                "TOGGLE_REPEAT" => { self.playlist.toggle_repeat_mode(); self.gui(GUICommand::REPEAT(self.playlist.get_repeat_mode())) },
                "SHUFFLE" => self.playlist.shuffle = true,
                "NO_SHUFFLE" => self.playlist.shuffle = false,
                "INFO" => println!("GODOT-PRINT: {}", args),
                "REWIND" => self.rewind(),
                "FAST_FORWARD" => self.fast_forward(),
                "SEEK" => if let Err(e) = args.parse().map(|d| self.seek(Duration::from_secs_f64(d))) { error!(e, "Cannot seek {}", args) }
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
                    PlayerState::Idle => {
                        if let Some(song) = self.playlist.get_history().get_current().cloned() {
                            self.play(song);
                        }
                    }
                    PlayerState::Play => {
                        self.gui(GUICommand::PAUSE);
                        self.player.pause();
                    }
                    PlayerState::Pause => {
                        self.gui(GUICommand::RESUME);
                        self.player.resume();
                    }
                    _ => {}
                }
            }
            
            if comb == self.stop_player {
                self.gui(GUICommand::STOP);
                self.player.stop();
                self.clear_lyrics();
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
                self.gui(if self.player.mute { GUICommand::MUTE } else { GUICommand::UNMUTE });
            }
            
            if comb == self.toggle_repeat_mode {
                self.playlist.toggle_repeat_mode();
                self.gui(GUICommand::REPEAT(self.playlist.get_repeat_mode()));
                
                show_notification(format!("Repeat mode: {}", self.playlist.get_repeat_mode().describe()));
            }
            
            if comb == self.toggle_shuffling {
                self.playlist.shuffle = !self.playlist.shuffle;
                self.gui(if self.playlist.shuffle { GUICommand::SHUFFLE } else { GUICommand::NO_SHUFFLE });
                
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
            
            if comb == self.jump_to_begin {
                self.gui(GUICommand::REPLAY);
                self.seek(Duration::ZERO);
            }
            
            if comb == self.jump_to_end {
                self.gui(GUICommand::STOP);
                self.player.skip();
            }
            
            if comb == self.prev_song {
                self.gui(GUICommand::STOP);
                
                if let Some(song) = self.playlist.look_back() {
                    self.play(song);
                }
            }
            
            if comb == self.toggle_lyrics_visibility {
                self.lyrics_layout.visible = !self.lyrics_layout.visible;
            }
            
            if comb == self.lyrics_top_left {
                self.lyrics_layout.position = LyricsPosition::TopLeft;
            }
            
            if comb == self.lyrics_top_center {
                self.lyrics_layout.position = LyricsPosition::TopCenter;
            }
            
            if comb == self.lyrics_top_right {
                self.lyrics_layout.position = LyricsPosition::TopRight;
            }
            
            if comb == self.lyrics_center_left {
                self.lyrics_layout.position = LyricsPosition::CenterLeft;
            }
            
            if comb == self.lyrics_center {
                self.lyrics_layout.position = LyricsPosition::Center;
            }
            
            if comb == self.lyrics_center_right {
                self.lyrics_layout.position = LyricsPosition::CenterRight;
            }
            
            if comb == self.lyrics_bottom_left {
                self.lyrics_layout.position = LyricsPosition::BottomLeft;
            }
            
            if comb == self.lyrics_bottom_center {
                self.lyrics_layout.position = LyricsPosition::BottomCenter;
            }
            
            if comb == self.lyrics_bottom_right {
                self.lyrics_layout.position = LyricsPosition::BottomRight;
            }
        }
        
        false
    }
    
    #[inline(always)]
    fn gui(&mut self, command: GUICommand) {
        if let Some(gui) = &mut self.gui {
            gui.command(command)
        }
    }
    
    fn launch_gui(&mut self) {
        self.close_gui();
        
        let mut dir = current_exe().expect("Failed to get current directory");
        
        dir.pop();
        dir.push("godot");
        
        let mut args = vec![
            arg!("cache-path" &self.cache_path),
            arg!("lyrics-margin" self.lyrics_layout.margin.to_string()),
            arg!("fps" self.fps.to_string()),
        ];
        
        match self.player.get_state() {
            PlayerState::Play | PlayerState::Pause => {
                if let PlayerState::Pause = self.player.get_state() {
                    args.push(OsString::from("--paused"))
                }
                
                args.push(arg!("song-path" self.playlist.get_history().get_current().unwrap()));
                args.push(arg!("song-duration" self.player.get_length().as_secs_f64().to_string()));
                args.push(arg!("song-position" self.player.get_position().as_secs_f64().to_string()));
            }
            _ => {
                if let Some(song) = self.playlist.get_history().get_current().cloned() {
                    args.push(arg!("last-song" song))
                }
            }
        }
        
        task!("Launching GUI");
        
        let mut gui = GUI::launch(dir.as_os_str(), args);
        
        gui.command(GUICommand::VOLUME(self.player.volume));
        gui.command(if self.player.mute { GUICommand::MUTE } else { GUICommand::UNMUTE });
        gui.command(if self.playlist.shuffle { GUICommand::SHUFFLE } else { GUICommand::NO_SHUFFLE });
        gui.command(GUICommand::REPEAT(self.playlist.get_repeat_mode()));
        
        self.gui.replace(gui);
    }
    
    #[inline(always)]
    fn close_gui(&mut self) {
        self.gui.take().map(GUI::close);
    }
    
    #[inline(always)]
    fn read_tags(&mut self, path: &str) {
        match Song::new(path) {
            Ok(song) => self.gui(GUICommand::TAGOF(song)),
            Err(e) => error!(e, "Failed to read tags {}", path),
        }
    }
    
    #[inline(always)]
    fn volume(&mut self, target: f32, notify: bool) {
        self.player.volume = target.clamp(0., 1.);
        self.player.update_volume();
        
        if notify {
            self.gui(GUICommand::VOLUME(self.player.volume));
        }
    }
    
    #[inline(always)]
    fn rewind(&mut self) {
        self.player.rewind(self.seek_duration);
        self.update_lyrics(self.player.get_position());
        self.gui(GUICommand::REWIND(self.seek_duration));
    }
    
    #[inline(always)]
    fn fast_forward(&mut self) {
        self.player.fast_forward(self.seek_duration);
        self.update_lyrics(self.player.get_position());
        self.gui(GUICommand::FAST_FORWARD(self.seek_duration));
    }
    
    #[inline(always)]
    fn seek(&mut self, time: Duration) {
        self.player.seek(time);
        self.update_lyrics(time);
    }
    
    #[inline(always)]
    fn update_lyrics(&mut self, time: Duration) {       
        if let Some(lyrics) = &mut self.lyrics {
            if let Err(e) = lyrics.update(time) {
                error!(e, "Failed to update lyrics")
            }
        }
    }
    
    #[inline(always)]
    fn play(&mut self, song: String) {
        self.player.play(&song);
        self.request_duration = true;
        self.show_lyrics(&song);
        self.gui(GUICommand::PLAY(song));
    }
    
    #[inline(always)]
    fn show_lyrics(&mut self, path: &str) {
        if let Some(lyrics) = &mut self.lyrics {
            match Song::synced_lyrics(path) {
                Ok(Some(l)) => if let Err(e) = lyrics.set_lyrics(l) { error!(e, "Failed to display lyrics") }
                Ok(None) => {}
                Err(e) => error!(e, "Failed to read tag from {}", path)
            }
        }
    }
    
    #[inline(always)]
    fn clear_lyrics(&mut self) {
        if let Some(lyrics) = &mut self.lyrics {
            if let Err(e) = lyrics.clear() {
                error!(e, "Failed to clear lyrics")
            }
        }
    }
}


fn main() {
    App::run();
}
