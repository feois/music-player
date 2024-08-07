use std::time::Duration;

use id3::{frame::SynchronisedLyricsType, v1v2::read_from_path};
use playback_rs::Song;
use serde::{Deserialize, Serialize};

use crate::{error, task};


pub enum PlayerState {
    Idle,
    Play,
    Pause,
    Finished,
}


pub struct Player {
    player: playback_rs::Player,
    state: PlayerState,
    lyrics: Vec<(u32, String)>,
    pub mute: bool,
    pub volume: f32,
}


#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
struct SerializedPlayer {
    mute: bool,
    volume: f32,
}


impl Player {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            player: playback_rs::Player::new(None).expect("Failed to initialize player"),
            state: PlayerState::Idle,
            lyrics: Vec::new(),
            mute: false,
            volume: 1.,
        }
    }
    
    #[inline(always)]
    pub fn update_state(&mut self) {
        match self.state {
            PlayerState::Play | PlayerState::Pause => {
                if !self.player.has_current_song() {
                    self.state = PlayerState::Finished
                }
            }
            _ => {}
        }
    }
    
    #[inline(always)]
    pub fn get_state(&self) -> &PlayerState {
        &self.state
    }
    
    #[inline(always)]
    pub fn idle(&mut self) {
        if let PlayerState::Finished = self.state {
            self.state = PlayerState::Idle
        }
    }
    
    #[inline(always)]
    pub fn play(&mut self, path: &str) {
        match Song::from_file(path, None) {
            Ok(song) => {
                if let Some(e) = self.player.play_song_now(&song, None).err() {
                    error!(e, "Failed to play song")
                }
                else {
                    task!("Playing song {}", path);
                    
                    self.player.set_playing(true);
                    self.state = PlayerState::Play;
                    
                    match read_from_path(path) {
                        Ok(tag) => {
                            if let Some(lyrics) = tag.synchronised_lyrics().find(|sl| sl.lang == "eng" && sl.content_type == SynchronisedLyricsType::Lyrics) {
                                self.lyrics = lyrics.content.clone();
                                self.lyrics.sort_by_key(|&(time, _)| time);
                            }
                        }
                        Err(e) => error!(e, "Cannot read tag from {}", path)
                    }
                }
            }
            Err(e) => error!(e, "Failed to load song {}", path),
        }
    }
    
    #[inline(always)]
    pub fn stop(&mut self) {
        if let PlayerState::Finished = self.state {}
        else {
            task!("Stopping current song");
            
            self.player.stop();
            
            self.state = PlayerState::Idle;
        }
    }
    
    #[inline(always)]
    pub fn skip(&mut self) {
        if let PlayerState::Idle = self.state {}
        else {
            task!("Skipping current song");
            
            self.player.stop();
            
            self.state = PlayerState::Finished;
        }
    }
    
    #[inline(always)]
    pub fn pause(&mut self) {
        if let PlayerState::Play = self.state {
            task!("Pausing");
            
            self.player.set_playing(false);
            
            self.state = PlayerState::Pause;
        }
    }
    
    #[inline(always)]
    pub fn resume(&mut self) {
        if let PlayerState::Pause = self.state {
            task!("Resuming");
            
            self.player.set_playing(true);
            
            self.state = PlayerState::Play;
        }
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
        self.player.set_volume(if self.mute { 0. } else { self.volume })
    }
    
    #[inline(always)]
    pub fn rewind(&self, duration: Duration) {
        self.seek(self.get_position().checked_sub(duration).unwrap_or(Duration::ZERO));
    }
    
    #[inline(always)]
    pub fn fast_forward(&self, duration: Duration) {
        self.seek(self.get_position() + duration);
    }
    
    #[inline(always)]
    pub fn seek(&self, duration: Duration) {
        self.player.seek(duration);
    }
}


impl Serialize for Player {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        SerializedPlayer {
            mute: self.mute,
            volume: self.volume
        }.serialize(serializer)
    }
}


impl<'de> Deserialize<'de> for Player {
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let s = SerializedPlayer::deserialize(deserializer)?;
        
        let mut p = Player::new();
        
        p.mute = s.mute;
        p.volume = s.volume;
        
        p.update_volume();
        
        Ok(p)
    }
}
