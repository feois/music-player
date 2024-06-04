use std::{io::{Read, Write}, process::{Child, ChildStdin, Command, Stdio}, sync::mpsc::{channel, Receiver}, thread};


pub struct GUI<const BUFFER_SIZE: usize = 1024> {
    process: Child,
    stdin: ChildStdin,
    receiver: Receiver<String>,
}


impl<const BUFFER_SIZE: usize> GUI<BUFFER_SIZE> {
    #[inline(always)]
    pub fn launch(path: String) -> Self {
        let mut process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Cannot launch GUI");
        let stdin = process.stdin.take().unwrap();
        let mut stdout = process.stdout.take().unwrap();
        
        let (sender, receiver) = channel();
        
        thread::spawn(move || {
            let mut buffer = [0; BUFFER_SIZE];
            
            loop {
                let n = stdout.read(&mut buffer).expect("Failed to read stdout");
                
                if n == 0 || sender.send(String::from_utf8(buffer[..n].to_vec()).expect("Invalid string")).is_err() {
                    break;
                }
            }
        });
        
        Self { process, stdin, receiver }
    }
    
    #[inline(always)]
    pub fn finished(&mut self) -> bool {
        self.process.try_wait().expect("Failed to wait").is_some()
    }
    
    #[inline(always)]
    pub fn write(&mut self, string: &str) {
        if !self.finished() {
            self.stdin.write_all(string.as_bytes()).expect("Failed to write stdin");
        }
    }
    
    #[inline(always)]
    pub fn endline(&mut self) {
        self.write("\n");
    }
    
    #[inline(always)]
    pub fn write_line(&mut self, string: &str) {
        self.write(string);
        self.endline();
    }
    
    #[inline(always)]
    pub fn flush(&mut self) {
        if !self.finished() {
            self.stdin.flush().expect("Failed to flush");
        }
    }
    
    #[inline(always)]
    pub fn read(&self) -> Option<String> {
        self.receiver.try_recv().ok()
    }
    
    #[inline(always)]
    pub fn kill(self) {
        let Self { mut process, receiver, .. } = self;
        
        std::mem::drop(receiver);
        
        process.kill().expect("Failed to kill GUI");
    }
}
