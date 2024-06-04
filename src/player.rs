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
    
    pub fn play(&self, filepath: &str) {
        let file = BufReader::new(File::open(filepath).expect("Failed to opeen file"));
        let source = Decoder::new(file).unwrap();
        
        self.sink.append(source);
    }
}
