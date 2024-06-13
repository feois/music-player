use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};


pub enum RepeatMode {
    NoRepeat,
    Repeat,
    RepeatTrack,
}

impl RepeatMode {
    #[inline(always)]
    pub fn to_string(&self) -> &str {
        match self {
            Self::NoRepeat => "none",
            Self::Repeat => "all",
            Self::RepeatTrack => "one",
        }
    }
    
    #[inline(always)]
    pub fn from_string(str: &str) -> Option<Self> {
        match str {
            "none" => Some(Self::NoRepeat),
            "all" => Some(Self::Repeat),
            "one" => Some(Self::RepeatTrack),
            _ => None,
        }
    }
}

pub struct Player {
    #[allow(dead_code)]
    stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
    pub repeat_mode: RepeatMode, 
}

impl Player {
    #[inline(always)]
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().expect("Failed to output audio");
        
        Self {
            stream,
            handle,
            sink: None,
            repeat_mode: RepeatMode::NoRepeat,
        }
    }
    
    #[inline(always)]
    pub fn play(&mut self, filepath: &str) {
        let file = BufReader::new(File::open(filepath).expect("Failed to open file"));
        let source = Decoder::new(file).unwrap();
        let sink = Sink::try_new(&self.handle).expect("Failed to create sink");
        
        sink.append(source);
        
        self.sink.replace(sink);
    }
    
    #[inline(always)]
    pub fn stop(&mut self) {
        self.sink.take();
    }
    
    #[inline(always)]
    pub fn pause(&self) {
        if let Some(sink) = &self.sink {
            sink.pause();
        }
    }
    
    #[inline(always)]
    pub fn resume(&self) {
        if let Some(sink) = &self.sink {
            sink.play();
        }
    }
    
    #[inline(always)]
    pub fn is_paused(&self) -> bool {
        self.sink.as_ref().is_some_and(|s| s.is_paused())
    }
    
    #[inline(always)]
    pub fn volume(&self, amp: f32) {
        if let Some(sink) = &self.sink {
            sink.set_volume(amp);
        }
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
