extends Panel


const CACHE_FILE := "user://cache.json"
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

class Song extends Resource:
	var path: String
	var title: String
	var artist: String
	var album: String
	var lyrics: String


var songs := {}
var artists := {}
var song_items := {}
var uninitalized_songs := {}
var inputs := {}

@onready var library: Tree = %Library
@onready var treeroot: TreeItem = $Tree.create_item()


func _ready() -> void:
	get_tree().set_auto_accept_quit(false)
	library.create_item()
	Stdin.command.connect(command)
	
	if read_cache():
		for song in songs.values():
			add_song(song)
	else:
		scan_library()


func _physics_process(_delta: float) -> void:
	var selected := library.get_selected()
	
	if selected && library.has_focus():
		var p := input(&"prev")
		var n := input(&"next")
		
		if p != n:
			var item: TreeItem
			
			if p:
				item = library.get_selected().get_prev_visible(true)
			
			if n:
				item = library.get_selected().get_next_visible(true)
			
			if library.get_root().get_child_count() > 0 && !item:
				item = library.get_root().get_child(library.get_root().get_child_count() - 1)
			
			item.select(0)

func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		prints("EXIT")


func save_cache() -> void:
	FileAccess.open(CACHE_FILE, FileAccess.WRITE).store_string(JSON.stringify({
		"songs": serialize_songs()
	}, "\t"))


func input(event: StringName, d := 300000) -> bool:
	var t := Time.get_ticks_usec()
	
	if Input.is_action_just_pressed(event, true):
		inputs[event] = t
		return true
	else:
		return Input.is_action_pressed(event, true) && t - inputs[event] > d


func read_cache() -> bool:
	if FileAccess.file_exists(CACHE_FILE):
		var file := FileAccess.open(CACHE_FILE, FileAccess.READ)
		
		if file:
			var json = JSON.parse_string(file.get_as_text())
			
			if json:
				var songs_dict: Dictionary = json["songs"]
				
				for path in songs_dict:
					var song := Song.new()
					var song_dict: Dictionary = songs_dict[path]
					
					song.path = path
					song.title = song_dict["title"]
					song.artist = song_dict["artist"]
					song.album = song_dict["album"]
					song.lyrics = song_dict["lyrics"].c_unescape()
					
					songs[path] = song
				
				return true
	
	return false

func scan_library() -> void:
	opendir(
		"/home/wilson/Music"
		#"Z:/home/wilson/Music"
		#"C:/Users/Admin/Music"
	)


func serialize_songs() -> Dictionary:
	var dict := {}
	
	for path in songs:
		var song := songs[path] as Song
		
		dict[path] = {
			"title": song.title,
			"artist": song.artist,
			"album": song.album,
			"lyrics": song.lyrics.c_escape(),
		}
	
	return dict


func opendir(path: String) -> void:
	var dir := DirAccess.open(path)
	
	if dir:
		var files := dir.get_files()
		var dirs := dir.get_directories()
		
		for file in files:
			if file.ends_with(".mp3"):
				prints("READTAG", path.path_join(file))
		
		for directory in dirs:
			opendir(path.path_join(directory))


func reparent_item(item: TreeItem, parent: TreeItem) -> void:
	if parent.get_child_count() > 0:
		var index := parent.get_children().bsearch_custom(item, comparer)
		
		if index < parent.get_child_count():
			item.move_before(parent.get_child(index))
			return
	
	treeroot.remove_child(item)
	parent.add_child(item)


func comparer(x: TreeItem, y: TreeItem) -> bool:
	return x.get_text(0) < y.get_text(0)


func get_artist(artist: String) -> TreeItem:
	if artists.has(artist):
		return artists[artist]
	
	var item := treeroot.create_child()
	
	item.set_text(0, artist)
	item.set_meta(&"albums", {})
	
	reparent_item(item, library.get_root())
	
	artists[artist] = item
	
	return item


func get_album(artist: TreeItem, album: String) -> TreeItem:
	var d := artist.get_meta(&"albums", {}) as Dictionary
	
	if d.has(album):
		return d[album]
	
	var item := treeroot.create_child()
	
	item.set_text(0, album)
	
	reparent_item(item, artist)
	
	d[album] = item
	
	artist.set_meta(&"albums", d)
	
	return item


func get_song(item: TreeItem) -> Song:
	return item.get_meta(&"songres") as Song if item.has_meta(&"songres") else null


func add_song(song: Song) -> void:
	var item := treeroot.create_child()
	
	item.set_text(0, song.title)
	item.add_button(0, preload("res://src/small-icon.svg"))
	item.set_meta(&"songres", song)
	
	song_items[song.path] = item
	
	reparent_item(item, get_album(get_artist(song.artist), song.album))


func read_tag(string: String) -> void:
	var mode := TagMode.NONE
	var song := Song.new()
	
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
		else:
			match mode:
				TagMode.PATH:
					song.path = section
					songs[section] = song
				
				TagMode.TITLE:
					song.title = section
				
				TagMode.ALBUM:
					song.album = section
				
				TagMode.ARTIST:
					song.artist = section
				
				TagMode.LYRICS:
					song.lyrics = section
			
			mode = TagMode.NONE
	
	song_items[song.path].set_meta(&"songres", song)
	
	add_song(song)

func command(string: String) -> void:
	if string == "EXIT":
		save_cache()
		get_tree().quit()
	
	if string.begins_with("TAGOF"):
		read_tag(string)
	
	if string.begins_with("VOLUME"):
		var volume := string.split(DELIMETER)[1]
		
		%VolumeLabel.text = volume + "%"
		%Volume.value = 100 - int(volume)


func _on_library_item_activated() -> void:
	var item := library.get_selected()
	var song := get_song(item)
	
	if song:
		prints("PLAY", song.path)
	else:
		item.collapsed = !item.collapsed


func _on_library_item_selected() -> void:
	library.scroll_to_item(library.get_selected())
