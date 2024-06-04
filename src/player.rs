use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, Sink};



pub struct Player {
    #[allow(dead_code)]
    stream: OutputStream,
    sink: Sink,
}

impl Player {
    #[inline(always)]
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().expect("Failed to output audio");
        let sink = Sink::try_new(&handle).expect("Failed to instantiate sink");
        
        Self {
            stream,
            sink,
        }
    }
    
    #[inline(always)]
    pub fn play(&self, filepath: &str) {
        let file = BufReader::new(File::open(filepath).expect("Failed to open file"));
        let source = Decoder::new(file).unwrap();
        
        self.sink.append(source);
    }
    
    #[inline(always)]
    pub fn stop(&self) {
        self.sink.stop();
    }
    
    #[inline(always)]
    pub fn pause(&self) {
        self.sink.pause();
    }
    
    #[inline(always)]
    pub fn resume(&self) {
        self.sink.play();
    }
}
