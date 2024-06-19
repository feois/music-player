class_name Song
extends RefCounted

var path: String
var title: String
var artist: String
var album: String
var lyrics: String

func filter(words: String) -> bool:
	for word in words.split(" ", false):
		var low := word.to_lower()
		
		if not (title.to_lower().contains(low) or artist.to_lower().contains(low) or album.to_lower().contains(low)):
			return false
	
	return true


func serialize() -> Dictionary:
	return {
		path = path,
		title = title,
		artist = artist,
		album = album,
		lyrics = lyrics,
	}


static func deserialize(dict: Dictionary) -> Song:
	var song := Song.new()
	
	song.path = dict.path
	song.title = dict.title
	song.artist = dict.artist
	song.album = dict.album
	song.lyrics = dict.lyrics
	
	return song
