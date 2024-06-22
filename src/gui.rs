use std::{ffi::OsStr, io::{BufRead, BufReader, Write}, process::{Child, ChildStdin, Command, Stdio}, sync::mpsc::{channel, Receiver}, thread::{self, panicking}, time::Duration};


pub const DELIMETER: &str = "::::";
pub const ENDLINE: &str = ";;;;";


pub struct GUI {
    process: Child,
    stdin: ChildStdin,
    receiver: Receiver<String>,
    buffer: String,
}

impl GUI {
    #[inline(always)]
    pub fn launch<T: AsRef<OsStr>>(path: &OsStr, args: impl IntoIterator<Item = T>) -> Self {
        let mut process = Command::new(path)
            .args(args)
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
                    Err(e) => { println!("RUST-ERROR: GUI output {}", e); break }
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
    pub fn write_delimeter(&mut self, string: &str) {
        self.write(string);
        self.write(DELIMETER);
    }
    
    #[inline(always)]
    pub fn write_iter<T: AsRef<str>>(&mut self, iter: impl IntoIterator<Item = T>) {
        let mut iter = iter.into_iter();
        
        if let Some(mut s) = iter.next() {
            for t in iter {
                self.write_delimeter(s.as_ref());
                s = t;
            }
            
            self.write_line(s.as_ref());
        }
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
        if !self.finished() {
            println!("TASK: Closing GUI");
            
            self.write_line("EXIT");
            
            spin_sleep::sleep(Duration::from_millis(500));
            
            if !self.finished() {
                println!("INFO: GUI has not closed yet");
                
                thread::spawn(move || {
                    for i in 0..3 {
                        spin_sleep::sleep(Duration::from_secs(1));
                        
                        if self.finished() {
                            return;
                        }
                        
                        println!("INFO: GUI has not closed yet, waiting {}", i + 1);
                    }
                    
                    println!("RUST-ERROR: Failed to close GUI");
                    println!("TASK: Killing GUI");
                    
                    self.process.kill().expect("Failed to kill");
                });
            }
        }
    }
}

impl Drop for GUI {
    #[inline(always)]
    fn drop(&mut self) {
        if panicking() {
            if !self.finished() {
                println!("TASK: Closing GUI");
                
                self.write_line("EXIT");
                
                spin_sleep::sleep(Duration::from_millis(500));
                
                if !self.finished() {
                    println!("INFO: GUI has not closed yet");
                    
                    for i in 0..3 {
                        spin_sleep::sleep(Duration::from_secs(1));
                        
                        if self.finished() {
                            return;
                        }
                        
                        println!("INFO: GUI has not closed yet, waiting {}", i + 1);
                    }
                    
                    println!("RUST-ERROR: Failed to close GUI");
                    println!("TASK: Killing GUI");
                    
                    self.process.kill().expect("Failed to kill");
                }
            }
        }
    }
}

pub trait GUIWrite {
    fn gui_write(self, gui: &mut impl AsMut<GUI>);
    fn gui_write_if(self, gui: &mut impl AsMut<Option<GUI>>);
}

impl<T: AsRef<str>, U: IntoIterator<Item = T>> GUIWrite for U {
    #[inline(always)]
    fn gui_write(self, gui: &mut impl AsMut<GUI>) {
        gui.as_mut().write_iter(self);
    }
    
    #[inline(always)]
    fn gui_write_if(self, gui: &mut impl AsMut<Option<GUI>>) {
        gui.as_mut().as_mut().map(|gui| gui.write_iter(self));
    }
}
