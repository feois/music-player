
use std::{fmt::{Debug, Display}, time::Duration};
use serde_derive::*;


#[cfg(not(feature = "xosd"))]
pub type Lyrics = Dummy;

#[cfg(feature = "xosd")]
pub type Lyrics = xosd::XosdLyrics;


#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LyricsLayout {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LyricsPosition {
    pub layout: LyricsLayout,
    pub margin: i32,
}


pub trait LyricsTrait {
    type Error;
    
    fn new(pos: LyricsPosition) -> std::result::Result<Self, Self::Error> where Self: Sized;
    fn is_showing(&self) -> Result<bool, Self::Error> { Ok(false) }
    fn reset(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn start(&mut self, _lyrics: Vec<(u32, String)>) -> Result<(), Self::Error> { Ok(()) }
    fn update(&mut self, _time: Duration) -> Result<(), Self::Error> { Ok(()) }
    fn set_pos(&mut self, _pos: LyricsPosition) -> Result<(), Self::Error> { Ok(()) }
}


pub struct Dummy;
#[derive(Debug)]
pub struct DummyError;

impl LyricsTrait for Dummy {
    type Error = DummyError;
    
    #[inline(always)]
    fn new(_pos: LyricsPosition) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

#[cfg(feature = "xosd")]
mod xosd {
    use super::*;
    use xosd_rs::*;
    
    #[inline(always)]
    fn new_xosd(color: &str, size: u8, osize: i32, h: HorizontalAlign, v: VerticalAlign, ho: i32, vo: i32) -> Result<Xosd> {
        let mut xosd = Xosd::new(1)?;
        
        xosd.set_color(color)?;
        xosd.set_outline_offset(osize)?;
        xosd.set_font(format!("-*-*-*-r-*-*-{}-*-*-*-*-*-*-*", size))?;
        xosd.set_horizontal_align(h)?;
        xosd.set_vertical_align(v)?;
        xosd.set_horizontal_offset(ho)?;
        xosd.set_vertical_offset(vo)?;
        
        Ok(xosd)
    }
    
    fn new_three(small_color: &str, big_color: &str, small: i32, big: i32, osize: i32, pos: LyricsPosition) -> Result<(Xosd, Xosd, Xosd)> {
        let (v, h, ho, vo) = get_xosd_align_margin(pos);
        
        let (o1, o2, o3) = match v {
            VerticalAlign::Top => (0, small, small + big),
            VerticalAlign::Center => (-big, 0, big),
            VerticalAlign::Bottom => (small + big, small, 0),
        };
        
        Ok((
            new_xosd(small_color, small as u8, osize, h, v, ho, vo + o1)?,
            new_xosd(big_color, big as u8, osize, h, v, ho, vo + o2)?,
            new_xosd(small_color, small as u8, osize, h, v, ho, vo + o3)?,
        ))
    }
    
    #[inline(always)]
    fn show(xosd: &mut Xosd, string: String) -> Result<()> {
        xosd.display(0, Command::String(string))?;
        
        Ok(())
    }
    
    #[inline(always)]
    fn get_xosd_align_margin(pos: LyricsPosition) -> (VerticalAlign, HorizontalAlign, i32, i32) {
        use LyricsLayout::*;
        use HorizontalAlign::{Left, Right, Center as HCenter};
        use VerticalAlign::{Top, Bottom, Center as VCenter};
        
        match pos.layout {
            TopLeft => (Top, Left, pos.margin, pos.margin),
            TopCenter => (Top, HCenter, 0, pos.margin),
            TopRight => (Top, Right, pos.margin, pos.margin),
            CenterLeft => (VCenter, Left, pos.margin, 0),
            Center => (VCenter, HCenter, 0, 0),
            CenterRight => (VCenter, Right, pos.margin, 0),
            BottomLeft => (Bottom, Left, pos.margin, pos.margin),
            BottomCenter => (Bottom, HCenter, 0, pos.margin),
            BottomRight => (Bottom, Right, pos.margin, pos.margin),
        }
    }
    
    pub struct XosdLyrics {
        lines: Vec<(u32, String)>,
        index: usize,
        showing: bool,
        pos: LyricsPosition,
        prev: Xosd,
        curr: Xosd,
        next: Xosd,
    }

    impl LyricsTrait for XosdLyrics {
        type Error = Error;
        
        #[inline(always)]
        fn new(pos: LyricsPosition) -> Result<Self> {
            let (prev, curr, next) = new_three("dark gray", "white", 24, 32, 2, pos)?;
            
            Ok(Self {
                lines: Vec::new(),
                index: usize::MAX,
                showing: false,
                pos,
                prev,
                curr,
                next,
            })
        }
        
        #[inline(always)]
        fn set_pos(&mut self, pos: LyricsPosition) -> std::result::Result<(), Self::Error> {
            if self.pos != pos {
                let (prev, curr, next) = new_three("dark gray", "white", 24, 32, 2, pos)?;
                
                self.prev = prev;
                self.curr = curr;
                self.next = next;
                
                self.pos = pos;
                self.index = usize::MAX;
            }
            
            Ok(())
        }
        
        #[inline(always)]
        fn is_showing(&self) -> std::result::Result<bool, Self::Error> {
            Ok(self.showing)
        }
        
        #[inline(always)]
        fn reset(&mut self) -> std::result::Result<(), Self::Error> {
            show(&mut self.prev, String::new())?;
            show(&mut self.curr, String::new())?;
            show(&mut self.next, String::new())?;
            
            self.showing = false;
            
            Ok(())
        }
        
        #[inline(always)]
        fn start(&mut self, lyrics: Vec<(u32, String)>) -> std::result::Result<(), Self::Error> {
            self.lines = lyrics;
            self.lines.sort_by_key(|&(t, _)| t);
            self.index = usize::MAX;
            
            Ok(())
        }
        
        #[inline(always)]
        fn update(&mut self, time: Duration) -> Result<()> {
            let i = self.lines.partition_point(|&(t, _)| time.as_millis() > t as u128);
            
            if i != self.index {
                self.index = i;
                
                show(&mut self.prev, if i > 1 { self.lines[i - 2].1[1..].to_string() } else { String::new() })?;
                show(&mut self.curr, if i > 0 { self.lines[i - 1].1[1..].to_string() } else { String::new() })?;
                show(&mut self.next, if i < self.lines.len() { self.lines[i].1[1..].to_string() } else { String::new() })?;
            }
            
            Ok(())
        }
    }
}
