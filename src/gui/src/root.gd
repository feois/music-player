extends Control


const DELIMETER := "::::"

enum TagMode {
	NONE,
	PATH,
	TITLE,
	ALBUM,
	ARTIST,
	LYRICS,
	SYNCED,
}


var songs := {}


func _ready() -> void:
	Stdin.command.connect(command)
	opendir("/home/wilson/Music")
	
	SelectorManager.connect_group(&"song",
		func (song, _old):
			%Title.text = song.title
			%Artist.text = song.artist
			%Album.text = song.album
			%Lyrics.text = song.lyrics
	)


func opendir(path: String) -> void:
	var dir := DirAccess.open(path)
	
	if dir:
		var files := dir.get_files()
		var dirs := dir.get_directories()
		
		for file in files:
			if file.ends_with(".mp3"):
				var song: Song = preload("res://src/song.tscn").instantiate()
				
				song.path = path.path_join(file)
				song.text = file
				
				songs[song.path] = song
				
				print("READTAG " + song.path)
				
				%Library.add_child(song)
		
		for directory in dirs:
			opendir(path.path_join(directory))


func command(string: String) -> void:
	if string.begins_with("TAGOF"):
		var mode := TagMode.NONE
		var song: Song
		
		for section in string.split(DELIMETER):
			if mode == TagMode.NONE:
				match section:
					"TAGOF":
						mode = TagMode.PATH
					
					"Title":
						mode = TagMode.TITLE
					
					"Artist":
						mode = TagMode.ARTIST
					
					"Album":
						mode = TagMode.ALBUM
					
					"Lyrics":
						mode = TagMode.LYRICS
					
					"Synced":
						mode = TagMode.SYNCED
			else:
				match mode:
					TagMode.PATH:
						song = songs[section]
					
					TagMode.TITLE:
						song.title = section
					
					TagMode.ALBUM:
						song.album = section
					
					TagMode.ARTIST:
						song.artist = section
					
					TagMode.LYRICS:
						song.lyrics = section
					
					TagMode.SYNCED:
						song.synced = section
				
				mode = TagMode.NONE
	
	pass
