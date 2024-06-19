use std::mem::MaybeUninit;

use serde::{Serialize, Deserialize};


pub struct History<T> {
    vec: Vec<MaybeUninit<T>>,
    len: usize,
    offset: usize,
    lookback: usize,
    capacity: usize,
}


#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
struct SerializedHistory<T> {
    vec: Vec<T>,
    lookback: usize,
}


impl<T: Eq> History<T> {
    #[inline(always)]
    pub fn new(capacity: usize) -> Self {
        Self {
            vec: (0..capacity).map(|_| MaybeUninit::uninit()).collect(),
            len: 0,
            offset: 0,
            lookback: 0,
            capacity,
        }
    }
    
    // Must only be called before any push
    #[inline(always)]
    pub unsafe fn set_capacity(&mut self, capacity: usize) {
        if capacity > self.capacity {
            self.vec.extend((self.capacity..capacity).map(|_| MaybeUninit::uninit()));
        }
        else if capacity < self.capacity {
            self.vec.drain(0..(self.capacity - capacity));
        }
        
        self.capacity = capacity;
    }
    
    #[inline(always)]
    fn get_offset_index(&self, i: usize) -> usize {
        let i = i + self.offset;
        
        if i >= self.capacity {
            i - self.capacity
        }
        else {
            i
        }
    }
    
    #[inline(always)]
    pub fn get(&self, i: usize) -> Option<&T> {
        (i < self.len).then(|| self.get_unchecked(i))
    }
    
    #[inline(always)]
    pub fn get_unchecked(&self, i: usize) -> &T {
        unsafe { self.vec[self.get_offset_index(i)].assume_init_ref() }
    }
    
    #[inline(always)]
    pub fn push(&mut self, t: T) {
        self.len -= self.lookback;
        self.lookback = 0;
        
        if !self.get_latest().is_some_and(|u| u == &t) {
            let i = self.get_offset_index(self.len);
            
            self.vec[i].write(t);
            
            if self.len < self.capacity {
                self.len += 1;
            }
            else {
                self.offset += 1;
            }
        }
    }
    
    #[inline(always)]
    pub fn look_back(&mut self) -> Option<&T> {
        if self.can_look_back() {
            self.lookback += 1;
            self.get(self.len - 1 - self.lookback)
        }
        else {
            None
        }
    }
    
    #[inline(always)]
    pub fn advance(&mut self) -> Option<&T> {
        if self.is_current_latest() {
            None
        }
        else {
            self.lookback -= 1;
            
            self.get_current()
        }
    }
    
    pub fn can_look_back(&self) -> bool {
        self.lookback + 1 < self.len
    }
    
    #[inline(always)]
    pub fn get_current(&self) -> Option<&T> {
        self.get(self.len - 1 - self.lookback)
    }
    
    #[inline(always)]
    pub fn get_latest(&self) -> Option<&T> {
        (self.len > 0).then(|| self.get_unchecked(self.len - 1))
    }
    
    #[inline(always)]
    pub fn is_current_latest(&self) -> bool {
        self.lookback == 0
    }
}


impl<T: Clone + Serialize + Eq> Serialize for History<T> {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let s = SerializedHistory {
            vec: (0..self.len).map(|i| self.get_unchecked(i).clone()).collect(),
            lookback: self.lookback,
        };
        
        s.serialize(serializer)
    }
}


impl<'de, T: Deserialize<'de>> Deserialize<'de> for History<T> {
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let SerializedHistory { vec, lookback } = SerializedHistory::deserialize(deserializer)?;
        let len = vec.len();
        
        Ok(History {
            vec: vec.into_iter().map(MaybeUninit::new).collect(),
            len,
            offset: 0,
            lookback,
            capacity: len,
        })
    }
}
