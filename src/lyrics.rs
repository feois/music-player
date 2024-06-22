
use std::{fmt::{Debug, Display}, time::Duration};
use serde_derive::*;


#[cfg(not(feature = "xosd"))]
pub type Lyrics = Dummy;

#[cfg(feature = "xosd")]
pub type Lyrics = xosd::XosdLyrics;


#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LyricsPosition {
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
pub struct LyricsLayout {
    pub position: LyricsPosition,
    pub margin: i32,
    pub visible: bool,
}


pub trait LyricsTrait {
    type Error;
    
    fn new(layout: LyricsLayout) -> std::result::Result<Self, Self::Error> where Self: Sized;
    fn is_showing(&self) -> Result<bool, Self::Error> { Ok(false) }
    fn reset(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn start(&mut self, _lyrics: Vec<(u32, String)>) -> Result<(), Self::Error> { Ok(()) }
    fn update(&mut self, _time: Duration) -> Result<(), Self::Error> { Ok(()) }
    fn set_layout(&mut self, _layout: LyricsLayout) -> Result<(), Self::Error> { Ok(()) }
}


pub struct Dummy;
#[derive(Debug)]
pub struct DummyError;

impl LyricsTrait for Dummy {
    type Error = DummyError;
    
    #[inline(always)]
    fn new(_layout: LyricsLayout) -> Result<Self, Self::Error> {
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
    
    fn new_three(small_color: &str, big_color: &str, small: i32, big: i32, osize: i32, layout: LyricsLayout) -> Result<(Xosd, Xosd, Xosd)> {
        let (v, h, ho, vo) = get_xosd_align_margin(layout);
        
        let (o1, o2, o3) = match v {
            VerticalAlign::Top => (0, small, small + big),
            VerticalAlign::Center => (big, 0, -big),
            VerticalAlign::Bottom => (small + big, small, 0),
        };
        
        Ok((
            new_xosd(small_color, small as u8, osize, h, v, ho, vo + o1)?,
            new_xosd(big_color, big as u8, osize, h, v, ho, vo + o2)?,
            new_xosd(small_color, small as u8, osize, h, v, ho, vo + o3)?,
        ))
    }
    
    #[inline(always)]
    fn show(xosd: &mut Xosd, string: Option<String>) -> Result<()> {
        xosd.display(0, Command::String(string.unwrap_or_default()))?;
        
        Ok(())
    }
    
    #[inline(always)]
    fn get_xosd_align_margin(layout: LyricsLayout) -> (VerticalAlign, HorizontalAlign, i32, i32) {
        use LyricsPosition::*;
        use HorizontalAlign::{Left, Right, Center as HCenter};
        use VerticalAlign::{Top, Bottom, Center as VCenter};
        
        match layout.position {
            TopLeft => (Top, Left, layout.margin, layout.margin),
            TopCenter => (Top, HCenter, 0, layout.margin),
            TopRight => (Top, Right, layout.margin, layout.margin),
            CenterLeft => (VCenter, Left, layout.margin, 0),
            Center => (VCenter, HCenter, 0, 0),
            CenterRight => (VCenter, Right, layout.margin, 0),
            BottomLeft => (Bottom, Left, layout.margin, layout.margin),
            BottomCenter => (Bottom, HCenter, 0, layout.margin),
            BottomRight => (Bottom, Right, layout.margin, layout.margin),
        }
    }
    
    pub struct XosdLyrics {
        lines: Vec<(u32, String)>,
        index: usize,
        showing: bool,
        hidden: bool,
        layout: LyricsLayout,
        prev: Xosd,
        curr: Xosd,
        next: Xosd,
    }
    
    impl LyricsTrait for XosdLyrics {
        type Error = Error;
        
        #[inline(always)]
        fn new(layout: LyricsLayout) -> Result<Self> {
            let (prev, curr, next) = new_three("dark gray", "white", 24, 32, 2, layout)?;
            
            Ok(Self {
                lines: Vec::new(),
                index: usize::MAX,
                showing: false,
                hidden: false,
                layout,
                prev,
                curr,
                next,
            })
        }
        
        #[inline(always)]
        fn set_layout(&mut self, layout: LyricsLayout) -> std::result::Result<(), Self::Error> {
            if self.layout.visible != layout.visible {
                self.layout.visible = layout.visible;
            }
            
            if self.layout != layout {
                let (prev, curr, next) = new_three("dark gray", "white", 24, 32, 2, layout)?;
                
                self.prev = prev;
                self.curr = curr;
                self.next = next;
                
                self.layout = layout;
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
            show(&mut self.prev, None)?;
            show(&mut self.curr, None)?;
            show(&mut self.next, None)?;
            
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
            if self.layout.visible {
                let i = self.lines.partition_point(|&(t, _)| time.as_millis() > t as u128);
                
                if self.hidden || i != self.index {
                    self.index = i;
                    self.hidden = false;
                    
                    show(&mut self.prev, (i > 1).then(|| self.lines[i - 2].1[1..].to_string()))?;
                    show(&mut self.curr, (i > 0).then(|| self.lines[i - 1].1[1..].to_string()))?;
                    show(&mut self.next, (i < self.lines.len()).then(|| self.lines[i].1[1..].to_string()))?;
                }
            }
            else if !self.hidden {
                show(&mut self.prev, None)?;
                show(&mut self.curr, None)?;
                show(&mut self.next, None)?;
                
                self.hidden = true;
            }
            
            Ok(())
        }
    }
}
