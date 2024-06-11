use std::{borrow::Borrow, collections::HashSet, hash::Hash, sync::mpsc::{channel, Receiver}, thread, time::{Duration, Instant}};

pub use rdev::Key;
pub use std::time::SystemTime as Time;

use crate::BooleanConditional;


struct KeyCombination(HashSet<Key>);

impl KeyCombination {
    #[inline(always)]
    pub fn new() -> Self {
        Self(HashSet::new())
    }
    
    #[inline(always)]
    pub fn add(&mut self, key: Key) {
        self.0.insert(key);
    }
    
    #[inline(always)]
    pub fn remove(&mut self, key: Key) {
        self.0.remove(&key);
    }
    
    #[inline(always)]
    pub fn includes(&self, comb: &KeyCombination) -> bool {
        self.0.is_superset(&comb.0)
    }
}

impl FromIterator<Key> for KeyCombination {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = Key>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Event {
    Pressed(Key, Time),
    Released(Key, Time),
}

pub struct EventListener {
    receiver: Receiver<rdev::Event>,
    keys: KeyCombination,
    combinations: Vec<(KeyCombination, Duration, Option<Instant>, bool)>,
}

impl Event {
    #[inline(always)]
    pub fn pressed(&self) -> bool {
        match self {
            Event::Pressed(..) => true,
            _ => false,
        }
    }
    
    #[inline(always)]
    pub fn released(&self) -> bool {
        !self.pressed()
    }
    
    #[inline(always)]
    pub fn key(&self) -> Key {
        match self {
            Event::Pressed(k, _) | Event::Released(k, _) => *k,
        }
    }
    
    #[inline(always)]
    pub fn time(&self) -> Time {
        match self {
            Event::Pressed(_, t) | Event::Released(_, t) => *t,
        }
    }
}

impl PartialOrd for Event {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time().partial_cmp(&other.time())
    }
}

impl Ord for Event {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time().cmp(&other.time())
    }
}

impl TryFrom<rdev::Event> for Event {
    type Error = ();
    
    #[inline(always)]
    fn try_from(value: rdev::Event) -> Result<Self, Self::Error> {
        match value.event_type {
            rdev::EventType::KeyPress(k) => Ok(Event::Pressed(k, value.time)),
            rdev::EventType::KeyRelease(k) => Ok(Event::Released(k, value.time)),
            _ => Err(()),
        }
    }
}

impl EventListener {
    #[inline(always)]
    pub fn listen() -> Self {
        let (s, r) = channel();
        
        thread::spawn(move || rdev::listen(move |e| { s.send(e).expect("Listener dropped"); }).expect("Failed to listen"));
        
        Self {
            receiver: r,
            keys: KeyCombination::new(),
            combinations: Vec::new(),
        }
    }
    
    #[inline(always)]
    pub fn poll_events(&mut self) {
        let now = Instant::now();
        
        for e in self.receiver.try_iter().filter_map(|e| Event::try_from(e).ok()) {
            if e.released() {
                self.keys.remove(e.key());
            }
            else {
                self.keys.add(e.key());
            }
        }
        
        for (comb, duration, t, flag) in &mut self.combinations {
            let overtime = !t.is_some_and(|t| now - t < *duration);
            
            *flag = self.keys.includes(comb).elsedo(|| { t.take(); }) && overtime.ifdo(|| { t.replace(now); });
        }
    }
    
    #[inline(always)]
    pub fn register_combination<T: Borrow<Key>>(&mut self, keys: impl IntoIterator<Item = T>, duration: Duration) -> usize {
        let id = self.combinations.len();
        
        self.combinations.push((keys.into_iter().map(|t| *t.borrow()).collect(), duration, None, false));
        
        id
    }
    
    #[inline(always)]
    pub fn register_once_combination<T: Borrow<Key>>(&mut self, keys: impl IntoIterator<Item = T>) -> usize {
        self.register_combination(keys, Duration::MAX)
    }
    
    #[inline(always)]
    pub fn is_pressed(&self, combination: usize) -> bool {
        self.combinations[combination].3
    }
    
    #[inline(always)]
    pub fn iter_pressed(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.combinations.len()).filter(|&i| self.is_pressed(i))
    }
}
