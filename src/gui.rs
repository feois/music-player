use std::{io::{BufRead, BufReader, Write}, process::{Child, ChildStdin, Command, Stdio}, sync::mpsc::{channel, Receiver}, thread, time::Duration};


pub const DELIMETER: &str = "::::";
pub const ENDLINE: &str = ";;;;";


pub struct GUI<const BUFFER_SIZE: usize = 1024> {
    process: Child,
    stdin: ChildStdin,
    receiver: Receiver<String>,
    buffer: String,
}

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
        
        let (sender, receiver) = channel();
        
        thread::spawn(move || {
            let mut bufreader = BufReader::new(stdout);
            let mut buffer = Vec::new();
            
            loop {
                match bufreader.read_until(b'\n', &mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if sender.send(String::from_utf8(buffer[..n - 1].to_vec()).expect("Invalid string")).is_err() {
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
        self.process.try_wait().expect("Failed to wait").inspect(|status| println!("GODOT-STATUS: {}", status)).is_some()
    }
    
    #[inline(always)]
    pub fn write(&mut self, string: &str) {
        self.buffer += string;
    }
    
    #[inline(always)]
    pub fn endline(&mut self) {
        self.buffer += ENDLINE;
        self.buffer += "\n";
        
        if !self.finished() {
            self.stdin.write_all(self.buffer.as_bytes()).expect("Failed to write stdin");
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
    pub fn close(mut self) {
        println!("TASK: Closing GUI");
        
        thread::spawn(move || {
            for i in 0..5 {
                self.write_line("EXIT");
                
                spin_sleep::sleep(Duration::from_secs(1));
                
                if self.finished() {
                    break;
                }
            }
        });
    }
}
