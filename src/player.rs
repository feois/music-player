use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};



pub struct Player {
    #[allow(dead_code)]
    stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
}

impl Player {
    #[inline(always)]
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().expect("Failed to output audio");
        
        Self {
            stream,
            handle,
            sink: None,
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
}
