use std::{io::{BufRead, BufReader, Write}, process::{Child, ChildStdin, Command, Stdio}, sync::mpsc::{channel, Receiver}, thread};


pub struct GUI<const BUFFER_SIZE: usize = 1024> {
    process: Child,
    stdin: ChildStdin,
    receiver: Receiver<String>,
    buffer: String,
}

#[cfg(target_os = "windows")]
const NEWLINE: usize = 2;

#[cfg(not(target_os = "windows"))]
const NEWLINE: usize = 1;

impl<const BUFFER_SIZE: usize> GUI<BUFFER_SIZE> {
    #[inline(always)]
    pub fn launch(path: &str) -> Self {
        let mut process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Cannot launch GUI");
        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();
        
        println!("TASK: Opening GUI");
        
        let (sender, receiver) = channel();
        
        thread::spawn(move || {
            let mut bufreader = BufReader::new(stdout);
            let mut buffer = Vec::new();
            
            loop {
                match bufreader.read_until(b'\n', &mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if n >= NEWLINE && sender.send(String::from_utf8(buffer[..n - NEWLINE].to_vec()).expect("Invalid string")).is_err() {
                            break;
                        }
                    }
                    Err(e) => { println!("ERROR: GUI output {}", e); break }
                }
                
                buffer.clear();
            }
        });
        
        Self { process, stdin, receiver, buffer: String::new() }
    }
    
    #[inline(always)]
    pub fn finished(&mut self) -> bool {
        self.process.try_wait().expect("Failed to wait").is_some()
    }
    
    #[inline(always)]
    pub fn write(&mut self, string: &str) {
        self.buffer += string;
    }
    
    #[inline(always)]
    pub fn endline(&mut self) {
        let mut s = self.buffer.replace("\n", "\\n");
        
        s += "\n";
        
        if !self.finished() {
            self.stdin.write_all(s.as_bytes()).expect("Failed to write stdin");
        }
        
        self.buffer.clear();
    }
    
    #[inline(always)]
    pub fn write_line(&mut self, string: &str) {
        self.write(string);
        self.endline();
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
