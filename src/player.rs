use std::time::Duration;

use playback_rs::Song;


pub enum RepeatMode {
    NoRepeat,
    Repeat,
    RepeatTrack,
}

impl RepeatMode {
    #[inline(always)]
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::NoRepeat => "none",
            Self::Repeat => "all",
            Self::RepeatTrack => "one",
        }
    }
}

pub struct Player {
    player: playback_rs::Player,
    pub volume: f32,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
}

impl Player {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            player: playback_rs::Player::new(None).expect("Failed to initialize player"),
            volume: 1.,
            repeat_mode: RepeatMode::NoRepeat,
            shuffle: false,
        }
    }
    
    #[inline(always)]
    pub fn play(&mut self, filepath: &str) {
        match Song::from_file(filepath, None) {
            Ok(song) => self.player.play_song_now(&song, None).inspect_err(|e| println!("Failed to play song {} {:?}", filepath, e)).unwrap_or(()),
            Err(e) => println!("Failed to load song {} {:?}", filepath, e),
        }
    }
    
    #[inline(always)]
    pub fn stop(&mut self) {
        self.player.stop();
    }
    
    #[inline(always)]
    pub fn pause(&self) {
        self.player.set_playing(false);
    }
    
    #[inline(always)]
    pub fn resume(&self) {
        self.player.set_playing(true);
    }
    
    #[inline(always)]
    pub fn is_playing(&self) -> bool {
        self.player.is_playing()
    }
    
    #[inline(always)]
    pub fn get_position(&self) -> Duration {
        self.player.get_playback_position().unwrap_or((Duration::ZERO, Duration::ZERO)).0
    }
    
    #[inline(always)]
    pub fn get_length(&self) -> Duration {
        self.player.get_playback_position().unwrap_or((Duration::ZERO, Duration::ZERO)).1
    }
    
    #[inline(always)]
    pub fn update_volume(&self) {
        self.player.set_volume(self.volume)
    }
    
    #[inline(always)]
    pub fn toggle_repeat_mode(&mut self) -> &RepeatMode {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::NoRepeat => RepeatMode::Repeat,
            RepeatMode::Repeat => RepeatMode::RepeatTrack,
            RepeatMode::RepeatTrack => RepeatMode::NoRepeat,
        };
        
        &self.repeat_mode
    }
}
