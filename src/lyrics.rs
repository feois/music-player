
use std::{fmt::{Debug, Display}, time::Duration};
use serde_derive::*;

use crate::BooleanConditional;


#[cfg(not(any(feature = "x11-lyrics", feature = "windows-lyrics")))]
pub type Lyrics = Dummy;

#[cfg(feature = "x11-lyrics")]
pub type Lyrics = xosd::XosdLyrics;

#[cfg(feature = "windows-lyrics")]
pub type Lyrics = windows::WindowsLyrics;


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
    type Error: Display;
    
    fn new(layout: LyricsLayout) -> std::result::Result<Self, Self::Error> where Self: Sized;
    #[inline(always)]
    fn set_lyrics(&mut self, _lyrics: Vec<(Duration, String)>) -> Result<(), Self::Error> { Ok(()) }
    #[inline(always)]
    fn clear(&mut self) -> Result<(), Self::Error> { Ok(()) }
    #[inline(always)]
    fn update(&mut self, _time: Duration) -> Result<(), Self::Error> { Ok(()) }
    #[inline(always)]
    fn set_layout(&mut self, _layout: LyricsLayout) -> Result<(), Self::Error> { Ok(()) }
    #[inline(always)]
    fn refresh(&mut self) -> Result<(), Self::Error> { Ok(()) }
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

#[derive(Default)]
pub struct Lines {
    lines: Vec<(Duration, String)>,
    index: Option<usize>,
}

impl Lines {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }
    
    #[inline(always)]
    pub fn set(&mut self, lyrics: Vec<(Duration, String)>) {
        self.index.replace(0);
        self.lines = lyrics;
    }
    
    #[inline(always)]
    pub fn update(&mut self, time: Duration) -> bool {
        let i = self.lines.partition_point(|&(t, _)| time > t);
        
        self.index.is_some_and(|index| i != index).ifdo(|| { self.index.replace(i); })
    }
    
    #[inline(always)]
    pub fn clear(&mut self) {
        self.index.take();
    }
    
    #[inline(always)]
    pub fn prev(&self) -> Option<&str> {
        self.index.filter(|&i| i > 1).map(|i| &self.lines[i - 2].1[1..])
    }
    
    #[inline(always)]
    pub fn curr(&self) -> Option<&str> {
        self.index.filter(|&i| i > 0).map(|i| &self.lines[i - 1].1[1..])
    }
    
    #[inline(always)]
    pub fn next(&self) -> Option<&str> {
        self.index.filter(|&i| i < self.lines.len()).map(|i| &self.lines[i].1[1..])
    }
}

#[allow(unused_macros)]
macro_rules! error_enum {
    ($name:ident $($var:ident $e:ty)*) => {
        pub enum $name {
            $($var($e),)*
        }
        
        impl std::fmt::Display for $name {
            #[inline(always)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #[allow(unused_imports)]
                use $name::*;
                
                match self {
                    $($var(e) => std::fmt::Display::fmt(&e, f),)*
                }
            }
        }
        
        $(
            impl From<$e> for $name {
                #[inline(always)]
                fn from(value: $e) -> Self {
                    $name::$var(value)
                }
            }
        )*
    };
}

#[cfg(feature = "x11-lyrics")]
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
            VerticalAlign::Center => ((small + big) / 2, 0, -(small + big) / 2),
            VerticalAlign::Bottom => (small + big, small, 0),
        };
        
        Ok((
            new_xosd(small_color, small as u8, osize, h, v, ho, vo + o1)?,
            new_xosd(big_color, big as u8, osize, h, v, ho, vo + o2)?,
            new_xosd(small_color, small as u8, osize, h, v, ho, vo + o3)?,
        ))
    }
    
    #[inline(always)]
    fn show(xosd: &mut Xosd, string: Option<&str>) -> Result<()> {
        if let Some(string) = string {
            xosd.display(0, Command::String(string.to_string()))?;
        }
        else if xosd.onscreen()? {
            xosd.hide()?;
        }
        
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
        lines: Lines,
        layout: LyricsLayout,
        prev: Xosd,
        curr: Xosd,
        next: Xosd,
    }
    
    impl XosdLyrics {
        #[inline(always)]
        fn update_text(&mut self) -> Result<()> {
            if self.layout.visible {
                show(&mut self.prev, self.lines.prev())?;
                show(&mut self.curr, self.lines.curr())?;
                show(&mut self.next, self.lines.next())?;
                
                return Ok(());
            }
            
            show(&mut self.prev, None)?;
            show(&mut self.curr, None)?;
            show(&mut self.next, None)?;
            
            Ok(())
        }
    }
    
    impl LyricsTrait for XosdLyrics {
        type Error = Error;
        
        #[inline(always)]
        fn new(layout: LyricsLayout) -> Result<Self> {
            let (prev, curr, next) = new_three("dark gray", "white", 24, 32, 2, layout)?;
            
            Ok(Self {
                lines: Lines::new(),
                layout,
                prev,
                curr,
                next,
            })
        }
        
        #[inline(always)]
        fn set_layout(&mut self, layout: LyricsLayout) -> Result<()> {
            if self.layout.visible != layout.visible {
                self.layout.visible = layout.visible;
                
                self.update_text()?;
            }
            
            if self.layout != layout {
                let (prev, curr, next) = new_three("dark gray", "white", 24, 32, 2, layout)?;
                
                self.prev = prev;
                self.curr = curr;
                self.next = next;
                
                self.layout = layout;
                self.update_text()?;
            }
            
            Ok(())
        }
        
        #[inline(always)]
        fn set_lyrics(&mut self, lyrics: Vec<(Duration, String)>) -> Result<()> {
            self.lines.set(lyrics);
            self.update_text()?;
            
            Ok(())
        }
        
        #[inline(always)]
        fn clear(&mut self) -> Result<()> {
            self.lines.clear();
            self.update_text()?;
            
            Ok(())
        }
        
        #[inline(always)]
        fn update(&mut self, time: Duration) -> Result<()> {
            if self.lines.update(time) {
                self.update_text()?;
            }
            
            Ok(())
        }
    }
}

#[cfg(feature = "windows-lyrics")]
mod windows {
    use crate::error;

    use super::*;
    use osd::*;
    use ::windows::{core::{w, PCWSTR}, Win32::{Foundation::*, Graphics::Gdi::*}};
    
    #[allow(dead_code)]
    mod osd {
        use std::{ffi::c_void, marker::PhantomData, sync::{Arc, RwLock}};
        use windows::{core::*, Win32::{Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*}};

        pub type WResult<T> = Result<T>;
        pub type WError = Error;

        pub trait GetSize: Sized {
            const SIZE: u32 = std::mem::size_of::<Self>() as u32;
        }

        impl<T: Sized> GetSize for T {}

        pub struct Window<T> {
            hwnd: HWND,
            arc: Arc<RwLock<T>>,
        }

        pub trait WindowName {
            const NAME: PCWSTR;
            
            #[inline(always)]
            fn get_name() -> PCWSTR { Self::NAME }
        }

        pub trait WindowCallback<T>: WindowName {
            unsafe fn callback(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM, rw: &RwLock<T>) -> LRESULT;
        }

        unsafe extern "system" fn window_proc<T, C: WindowCallback<T>>(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
            if msg == WM_CREATE {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*(l.0 as *const CREATESTRUCTW)).lpCreateParams as isize);
                DefWindowProcW(hwnd, msg, w, l)
            }
            else {
                C::callback(hwnd, msg, w, l, &*(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RwLock<T>))
            }
        }

        pub trait Paint {
            unsafe fn paint(hwnd: HWND, hdc: HDC, r: RECT, rw: &RwLock<Self>);
        }
        
        #[inline(always)]
        pub fn draw_text(hdc: HDC, mut r: RECT, s: &str, dt: DRAW_TEXT_FORMAT) -> i32 {
            if s.is_empty() {
                0
            }
            else {
                unsafe { DrawTextW(hdc, &mut s.encode_utf16().collect::<Vec<_>>(), &mut r, dt) }
            }
        }
        
        #[inline(always)]
        pub fn set_font_size(hdc: HDC, size: i32, f: impl FnOnce()) {
            unsafe {
                let hfont = GetStockObject(DEFAULT_GUI_FONT);
                let logfont = &mut LOGFONTW::default();
                
                GetObjectW(hfont, HFONT::SIZE as i32, Some((logfont as *mut LOGFONTW) as *mut c_void));
                
                logfont.lfHeight = size;
                
                let font = CreateFontIndirectW(logfont);
                let old = SelectObject(hdc, font);
                
                f();
                
                SelectObject(hdc, old);
                assert!(DeleteObject(font).as_bool());
            }
        }
        
        pub struct PaintCallback<T>(PhantomData<T>);

        impl<T: Paint> WindowCallback<T> for PaintCallback<T> where PaintCallback<T>: WindowName {
            unsafe fn callback(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM, rw: &RwLock<T>) -> LRESULT {
                match msg {
                    WM_PAINT => {
                        let mut r = RECT::default();
                        let ps = &mut PAINTSTRUCT::default();
                        let hdc = BeginPaint(hwnd, ps);
                        
                        GetClientRect(hwnd, &mut r).expect("Failed to get rect");
                        SetBkMode(hdc, TRANSPARENT);
                        T::paint(hwnd, hdc, r, rw);
                        
                        let _ = EndPaint(hwnd, ps);
                    }
                    WM_NCHITTEST => return LRESULT(HTNOWHERE as isize),
                    WM_DESTROY => PostQuitMessage(0),
                    _ => {}
                }
                
                DefWindowProcW(hwnd, msg, w, l)
            }
        }

        impl<T: WindowName> WindowName for PaintCallback<T> {
            const NAME: PCWSTR = T::NAME;
            
            #[inline(always)]
            fn get_name() -> PCWSTR {
                T::get_name()
            }
        }

        impl<T> Window<T> {
            pub fn new<C: WindowCallback<T>>(t: T) -> Result<Self> {
                unsafe {
                    let name = C::get_name();
                    
                    let h_instance = GetModuleHandleW(None)?;
                    let class = &mut WNDCLASSEXW { cbSize: WNDCLASSEXW::SIZE, ..Default::default() };
                    
                    if GetClassInfoExW(h_instance, name, class).is_err() {
                        class.style = CS_HREDRAW | CS_VREDRAW;
                        class.hInstance = h_instance.into();
                        class.hCursor = LoadCursorW(None, IDC_ARROW)?;
                        class.hbrBackground = HBRUSH(GetStockObject(BLACK_BRUSH).0);
                        class.lpszClassName = name;
                        class.lpfnWndProc.replace(window_proc::<T, C>);
                        
                        RegisterClassExW(class);
                    }
                    
                    let arc = Arc::new(RwLock::new(t));
                    
                    let hwnd = CreateWindowExW(
                        WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TOOLWINDOW,
                        name, PCWSTR::null(),
                        WS_POPUP | WS_VISIBLE,
                        0, 0, GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN),
                        None, None,
                        h_instance, Some(Arc::as_ptr(&arc) as *const c_void)
                    )?;
                    
                    SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA | LWA_COLORKEY)?;
                    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                    let _ = UpdateWindow(hwnd);
                    
                    Ok(Self {
                        hwnd,
                        arc,
                    })
                }
            }
            
            #[inline(always)]
            pub fn dispatch_messages(&self) -> Result<()> {
                let msg = &mut MSG::default();
                
                loop {
                    unsafe {
                        let BOOL(b) = PeekMessageW(msg, self.hwnd, 0, 0, PM_NOREMOVE);
                        
                        if b == -1 {
                            return Err(WError::from_win32());
                        }
                        
                        if b == 0 {
                            break;
                        }
                        
                        let _ = TranslateMessage(msg);
                        DispatchMessageW(msg);
                    }
                }
                
                Ok(())
            }
            
            #[inline(always)]
            pub unsafe fn get_arc(&self) -> Arc<RwLock<T>> {
                self.arc.clone()
            }
            
            #[inline(always)]
            pub fn try_read(&self, f: impl FnOnce(&T)) -> bool {
                self.arc.try_read().map(|arc| f(&arc)).is_ok()
            }
            
            #[inline(always)]
            pub fn try_write(&self, f: impl FnOnce(&mut T)) -> bool {
                self.arc.try_write().map(|mut arc| f(&mut *arc)).is_ok()
            }
            
            #[inline(always)]
            pub fn redraw(&self) -> bool {
                unsafe { RedrawWindow(self.hwnd, None, None, RDW_INVALIDATE | RDW_ERASE).as_bool() }
            }
            
            #[inline(always)]
            pub fn get_hwnd(&self) -> HWND {
                self.hwnd
            }
        }
    }
    
    error_enum!(Error
        WindowError WError
        LockError &'static str
    );
    
    type Result<T> = std::result::Result<T, Error>;
    
    struct LyricsArc {
        lines: Lines,
        layout: LyricsLayout,
    }
    
    impl WindowName for LyricsArc {
        const NAME: PCWSTR = w!("LyricsOSD");
    }
    
    fn translate_position(pos: LyricsPosition) -> DRAW_TEXT_FORMAT {
        use LyricsPosition::*;
        
        match pos {
            TopLeft => DT_TOP | DT_LEFT,
            TopCenter => DT_TOP | DT_CENTER,
            TopRight => DT_TOP | DT_RIGHT,
            CenterLeft => DT_VCENTER | DT_LEFT,
            Center => DT_VCENTER | DT_CENTER,
            CenterRight => DT_VCENTER | DT_RIGHT,
            BottomLeft => DT_BOTTOM | DT_LEFT,
            BottomCenter => DT_BOTTOM | DT_CENTER,
            BottomRight => DT_BOTTOM | DT_RIGHT,
        }
    }
    
    impl Paint for LyricsArc {
        unsafe fn paint(_hwnd: ::windows::Win32::Foundation::HWND, hdc: ::windows::Win32::Graphics::Gdi::HDC, mut r: ::windows::Win32::Foundation::RECT, rw: &std::sync::RwLock<Self>) {
            match rw.try_write() {
                Ok(mut arc) => {
                    let arc = &mut *arc;
                    let dt = translate_position(arc.layout.position) | DT_SINGLELINE | DT_INTERNAL;
                    
                    r.top += arc.layout.margin;
                    r.left += arc.layout.margin;
                    r.bottom -= arc.layout.margin;
                    r.right -= arc.layout.margin;
                    
                    let big = 32;
                    let small = 24;
                    
                    if arc.layout.visible {
                        set_font_size(hdc, big, || {
                            if let Some(s) = arc.lines.curr() {
                                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                                
                                draw_text(hdc, match dt {
                                    _dt if dt & DT_TOP == DT_TOP => RECT { top: r.top + small, ..r },
                                    _dt if dt & DT_VCENTER == DT_VCENTER => r,
                                    _dt if dt & DT_BOTTOM == DT_BOTTOM => RECT { bottom: r.bottom - small, ..r },
                                    _ => unimplemented!(),
                                }, s, dt);
                            }
                        });
                        
                        set_font_size(hdc, small, || {
                            SetTextColor(hdc, COLORREF(0x00777777));
                            
                            if let Some(s) = arc.lines.prev() {
                                draw_text(hdc, match dt {
                                    _dt if dt & DT_TOP == DT_TOP => r,
                                    _dt if dt & DT_VCENTER == DT_VCENTER => RECT { bottom: r.bottom - big, ..r },
                                    _dt if dt & DT_BOTTOM == DT_BOTTOM => RECT { bottom: r.bottom - small - big, ..r },
                                    _ => unimplemented!(),
                                }, s, dt);
                            }
                            
                            if let Some(s) = arc.lines.next() {
                                draw_text(hdc, match dt {
                                    _dt if dt & DT_TOP == DT_TOP => RECT { top: r.top + small + big, ..r },
                                    _dt if dt & DT_VCENTER == DT_VCENTER => RECT { top: r.top + big, ..r },
                                    _dt if dt & DT_BOTTOM == DT_BOTTOM => r,
                                    _ => unimplemented!(),
                                }, s, dt);
                            }
                        });
                    }
                }
                Err(e) => error!(e, "Failed to read lock")
            }
        }
    }
    
    #[repr(transparent)]
    pub struct WindowsLyrics(Window<LyricsArc>);
    
    impl LyricsTrait for WindowsLyrics {
        type Error = Error;
        
        fn new(layout: LyricsLayout) -> Result<Self> where Self: Sized {
            Ok(Self(Window::new::<PaintCallback<_>>(
                LyricsArc {
                    lines: Lines::new(),
                    layout,
                })
            ?))
        }
        
        fn set_layout(&mut self, layout: LyricsLayout) -> Result<()> {
            let Self(w) = self;
            
            if w.try_write(|arc| if arc.layout != layout { arc.layout = layout; w.redraw(); }) {
                Ok(())
            }
            else {
                Err(Error::LockError("Write Lock Error"))
            }
        }
        
        fn set_lyrics(&mut self, lyrics: Vec<(Duration, String)>) -> Result<()> {
            let Self(w) = self;
            
            if w.try_write(|arc| arc.lines.set(lyrics)) {
                w.redraw();
                Ok(())
            }
            else {
                Err(Error::LockError("Write Lock Error"))
            }
        }
        
        fn update(&mut self, time: Duration) -> Result<()> {
            let Self(w) = self;
            
            if w.try_write(|arc| if arc.lines.update(time) { w.redraw(); }) {
                Ok(())
            }
            else {
                Err(Error::LockError("Write Lock Error"))
            }
        }
        
        fn clear(&mut self) -> Result<()> {
            let Self(w) = self;
            
            if w.try_write(|arc| arc.lines.clear()) {
                w.redraw();
                Ok(())
            }
            else {
                Err(Error::LockError("Write Lock Error"))
            }
        }
        
        fn refresh(&mut self) -> std::result::Result<(), Self::Error> {
            let Self(w) = self;
            
            w.dispatch_messages()?;
            
            Ok(())
        }
    }
}
