use bitvec::vec::BitVec;
use serde_derive::{Serialize, Deserialize};
use serde_json::{to_value, Value};

use crate::history::History;


#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum RepeatMode {
    #[serde(rename = "none")]
    NoRepeat,
    #[serde(rename = "all")]
    Repeat,
    #[serde(rename = "one")]
    RepeatTrack,
    #[serde(rename = "stop")]
    Stop,
}


impl RepeatMode {
    #[inline(always)]
    pub fn get_string(&self) -> String {
        let Value::String(s) = to_value(self).unwrap() else { unreachable!() };
        
        s
    }
    
    pub fn describe(&self) -> &'static str {
        match self {
            RepeatMode::NoRepeat => "No repeat",
            RepeatMode::Repeat => "Repeat all",
            RepeatMode::RepeatTrack => "Repeat one only",
            RepeatMode::Stop => "Stop after every song",
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct Playlist {
    songs: Vec<String>,
    history: History<String>,
    played: BitVec,
    play_index: usize,
    play_index_deleted: bool,
    repeat_mode: RepeatMode,
    pub shuffle: bool,
}


impl Playlist {
    #[inline(always)]
    pub fn new(history_keep_at_most: usize) -> Self {
        Self {
            songs: Vec::new(),
            history: History::new(history_keep_at_most),
            played: BitVec::new(),
            play_index: 0,
            play_index_deleted: false,
            repeat_mode: RepeatMode::NoRepeat,
            shuffle: false,
        }
    }
    
    #[inline(always)]
    pub unsafe fn history_keep_at_most(&mut self, capacity: usize) {
        self.history.set_capacity(capacity)
    }
    
    #[inline(always)]
    pub fn count(&self) -> usize {
        self.songs.len()
    }
    
    pub fn get_history(&self) -> &History<String> {
        &self.history
    }
    
    #[inline(always)]
    pub fn append(&mut self, song: String) {
        if self.play_index_deleted && self.play_index >= self.count() {
            self.play_index = self.count();
        }
        
        self.songs.push(song);
        self.played.push(false);
    }
    
    #[inline(always)]
    pub fn delete(&mut self, index: usize) {
        self.songs.remove(index);
        self.played.remove(index);
        
        if index < self.play_index {
            self.play_index -= 1;
        }
        else if self.play_index == index {
            self.play_index_deleted = true;
        }
    }
    
    #[inline(always)]
    pub fn clear(&mut self) {
        self.songs.clear();
        self.played.clear();
        self.play_index_deleted = true;
    }
    
    #[inline(always)]
    pub fn update(&mut self, index: usize, song: String) {
        self.songs[index] = song
    }
    
    #[inline(always)]
    pub fn arrange(&mut self, from: usize, to: usize) {
        let song = self.songs.remove(from);
        self.songs.insert(to, song);
        
        let played = self.played.remove(from);
        self.played.insert(to, played);
        
        if from == self.play_index {
            self.play_index = to;
        }
        else {
            if from < self.play_index {
                self.play_index -= 1;
            }
            
            if to <= self.play_index {
                self.play_index += 1;
            }
        }
    }
    
    #[inline(always)]
    pub fn get_repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }
    
    #[inline(always)]
    pub fn toggle_repeat_mode(&mut self) {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::NoRepeat => RepeatMode::Repeat,
            RepeatMode::Repeat => RepeatMode::RepeatTrack,
            RepeatMode::RepeatTrack => RepeatMode::Stop,
            RepeatMode::Stop => RepeatMode::NoRepeat,
        }
    }
    
    #[inline(always)]
    pub fn select(&mut self, index: usize) -> &str {
        self.play_index_deleted = false;
        self.play_index = index;
        self.history.push(self.songs[index].clone());
        
        &self.songs[index]
    }
    
    #[inline(always)]
    pub fn look_back(&mut self) -> Option<String> {
        self.history.look_back().map(Clone::clone)
    }
    
    pub fn poll(&mut self) -> Option<&str> {
        let repeat = match self.repeat_mode {
            RepeatMode::NoRepeat => false,
            RepeatMode::Repeat => true,
            RepeatMode::RepeatTrack => return self.history.get_current().map(String::as_str),
            RepeatMode::Stop => return None,
        };
        
        if self.history.is_current_latest() {
            let next;
            
            if self.play_index_deleted {
                next = self.play_index;
            }
            else {
                next = self.play_index + 1;
                self.played.set(self.play_index, true);
            }
            
            if self.count() > 0 {
                if self.shuffle {
                    let n = self.played.count_ones();
                    
                    if n < self.count() {
                        Some(self.select(self.played.iter_zeros().nth(rand::random::<usize>() % (self.count() - n)).unwrap()))
                    }
                    else if repeat { // all songs played once
                        self.played.fill(false);
                        Some(self.select(rand::random::<usize>() % self.count()))
                    }
                    else {
                        self.played.fill(false);
                        None
                    }
                }
                else if next < self.count() { // next song in playlist
                    Some(self.select(next))
                }
                else if repeat { // playlist finished
                    self.played.fill(false);
                    Some(self.select(0))
                }
                else {
                    self.played.fill(false);
                    None
                }
            }
            else {
                None
            }
        }
        else {
            self.history.advance().map(String::as_str)
        }
    }
}

