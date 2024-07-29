use std::{error::Error, path::Path, time::Duration};

use id3::{Tag, TagLike};
use serde_derive::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Song {
    path: String,
    title: Option<String>,
    album: Option<String>,
    artists: Vec<String>,
    lyrics: Option<String>,
}

impl Song {
    #[inline(always)]
    pub fn new(path: impl AsRef<Path> + Into<String>) -> Result<Self, impl Error> {
        Tag::read_from_path(path.as_ref()).map(|tag| Song {
            path: path.into(),
            title: tag.title().map(str::to_string),
            album: tag.album().map(str::to_string),
            artists: tag.artists().unwrap_or_default().into_iter().map(str::to_string).collect(),
            lyrics: tag.lyrics().find(|lyrics| lyrics.lang == "eng").map(|lyrics| &lyrics.text).filter(|s| !s.is_empty()).map(Clone::clone),
        })
    }
    
    #[inline(always)]
    pub fn synced_lyrics(path: impl AsRef<Path>) -> Result<Option<Vec<(Duration, String)>>, impl Error> {
        Tag::read_from_path(path).map(|tag|
            tag.synchronised_lyrics()
                .find(|l| l.lang == "eng")
                .map(|l| {
                    let mut v: Vec<_> = l.content.iter().map(|(t, s)| (Duration::from_millis((*t).into()), s.clone())).collect();
                    
                    v.sort_by_key(|&(d, _)| d);
                    
                    v
                }))
    }
}
