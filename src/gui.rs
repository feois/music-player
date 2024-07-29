use core::str;
use std::{ffi::OsStr, io::{BufRead, BufReader, Write}, process::{Child, ChildStdin, Command, Stdio}, sync::mpsc::{channel, Receiver}, thread::{self, panicking}, time::Duration};
use crate::{error, task, RepeatMode, Song};
use serde_json::to_string;


macro_rules! enum_str {
    ($e:ident $($v:ident $(($t:ty $(, $ts:ty)* $(,)?))?)+) => {
        #[allow(non_camel_case_types)]
        pub enum $e {
            $($v $(($t $(, $ts)*))?,)+
        }
        
        impl $e {
            #[inline(always)]
            pub const fn stringify(&self) -> &'static str {
                match self {
                    $($e::$v $((enum_str!(@FIELD $t) $(, enum_str!(@FIELD $ts))*))? => stringify!($v),)+
                }
            }
        }
    };
    
    (@FIELD $t:ty) => {
        _
    };
}


macro_rules! args {
    ($e:expr $(, $es:expr)* $(,)?) => {
        &format!(args!(@GET_LITERAL $e $(, $es)*), $e.stringify() $(, $es)*)
    };
    
    (@GET_LITERAL $e:expr $(,)?) => {
        "{}"
    };
    
    (@GET_LITERAL $e:expr $(, $es:expr)+ $(,)?) => {
        concat!("{} ", args!(@GET_LITERAL $($es,)*))
    };
}


enum_str!(GUICommand
    TAGOF(Song)
    DURATION(Duration)
    STOP
    PAUSE
    RESUME
    MUTE
    UNMUTE
    SHUFFLE
    NO_SHUFFLE
    REPEAT(RepeatMode)
    REPLAY
    VOLUME(f32)
    FAST_FORWARD(Duration)
    REWIND(Duration)
    PLAY(String)
    EXIT
);


impl GUICommand {
    #[inline(always)]
    pub fn get_str(&self, f: impl FnOnce(&str)) {
        use GUICommand::*;
        
        let s = match self {
            TAGOF(song) => args!(self, to_string(song).unwrap()),
            DURATION(duration) => args!(self, duration.as_secs_f64().to_string()),
            VOLUME(volume) => args!(self, volume.to_string()),
            REPEAT(mode) => args!(self, mode.get_string()),
            FAST_FORWARD(duration) => args!(self, duration.as_secs_f64().to_string()),
            REWIND(duration) => args!(self, duration.as_secs_f64().to_string()),
            PLAY(song) => args!(self, song),
            _ => self.stringify(),
        };
        
        f(s)
    }
}


pub struct GUI {
    process: Child,
    stdin: ChildStdin,
    receiver: Receiver<String>,
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
                        match String::from_utf8(buffer[..n - 1].to_vec()) {
                            Ok(s) => if sender.send(s).is_err() { break }
                            Err(e) => error!(e, "Invalid string {:?}", &buffer)
                        }
                    }
                    Err(e) => { error!(e, "Failed to read GUI output"); break }
                }
                
                buffer.clear();
            }
        });
        
        Self { process, stdin, receiver }
    }
    
    #[inline(always)]
    pub fn finished(&mut self) -> bool {
        self.process.try_wait().expect("Failed to wait").inspect(|status| println!("GODOT-STATUS: {}", status)).is_some()
    }
    
    #[inline(always)]
    pub fn command(&mut self, command: GUICommand) {
        command.get_str(|s| self.write(s))
    }
    
    #[inline(always)]
    pub fn write(&mut self, s: &str) {
        #[inline(always)]
        fn m(b: u8) -> usize {
            const L2: u8 = 0b11000000;
            const L3: u8 = 0b11100000;
            const L4: u8 = 0b11110000;
            
            if b >> 7 == 0 {
                1
            }
            else if b & L4 == L4 {
                4
            }
            else if b & L3 == L3 {
                3
            }
            else if b & L2 == L2 {
                2
            }
            else if b >> 7 == 1 {
                0
            }
            else {
                unreachable!()
            }
        }
        
        #[inline(always)]
        fn f(w: &mut impl Write, mut b: &[u8]) -> Result<(), impl std::error::Error> {
            const CHUNK_SIZE: usize = 1024;
            const END: &'static [u8] = "\n\n".as_bytes();
            const NEWLINE: &'static [u8] = "\n".as_bytes();
            const LINE_SIZE: usize = CHUNK_SIZE - NEWLINE.len();
            
            while b.len() + END.len() > CHUNK_SIZE {
                let n = match m(b[LINE_SIZE - 1]) {
                    1 => 0,
                    2 | 3 | 4 => 1,
                    0 => {
                        match m(b[LINE_SIZE - 2]) {
                            0 => match m(b[LINE_SIZE - 3]) {
                                0 => match m(b[LINE_SIZE - 4]) {
                                    0 | 1 | 2 | 3 => unreachable!("Invalid byte"),
                                    4 => 0,
                                    _ => unreachable!(),
                                }
                                1 | 2 => unreachable!("Invalid byte"),
                                3 => 0,
                                4 => 3,
                                _ => unreachable!(),
                            }
                            1 => unreachable!("Invalid byte"),
                            2 => 0,
                            3 | 4 => 2,
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!()
                };
                
                w.write_all(&b[..LINE_SIZE - n])?;
                w.write_all(NEWLINE)?;
                
                b = &b[LINE_SIZE - n..];
            }
            
            w.write_all(b)?;
            w.write_all(END)
        }
        
        if self.finished() {
            error!("Tried to write to non-existent GUI")
        }
        else if let Err(e) = f(&mut self.stdin, s.as_bytes()) {//self.stdin.write_all(s.as_bytes()) {
            error!(e, "Failed to communicate with GUI")
        }
    }
    
    #[inline(always)]
    pub fn read(&self) -> Option<String> {
        self.receiver.try_recv().ok()
    }
    
    #[inline(always)]
    pub fn close(mut self) {
        if !self.finished() {
            task!("Closing GUI");
            
            self.command(GUICommand::EXIT);
        }
    }
}

impl Drop for GUI {
    #[inline(always)]
    fn drop(&mut self) {
        if panicking() {
            self.command(GUICommand::EXIT);
        }
    }
}
