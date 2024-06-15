class_name Root
extends Panel


const CACHE_FILE := "user://cache.json"
const PLAY_SYMBOL := "⏵"
const PAUSE_SYMBOL := "⏸"

const SHUFFLE := preload("res://src/shuffle.png")
const NO_SHUFFLE := preload("res://src/no_shuffle.png")

enum TagMode {
	NONE,
	PATH,
	TITLE,
	ALBUM,
	ARTIST,
	LYRICS,
}


class Song extends RefCounted:
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

var is_playing := false
var is_pausing := false
var played_once := false
var song_duration := 0.0:
	set(value):
		song_duration = maxf(0, value)
		
		if is_node_ready():
			song_progress.max_value = song_duration
var song_position := 0.0:
	set(value):
		song_position = maxf(0, value)
		
		if is_node_ready() and song_duration != 0:
			song_progress.value = song_position
			%SongProgress/TextureRect.position.x = song_progress.size.x * song_position / song_duration - %SongProgress/TextureRect.size.x / 2

@onready var default_library_path := OS.get_environment("USERPROFILE" if OS.has_feature("windows") else "HOME").path_join("Music")
@onready var library_path := default_library_path
@onready var library: Tree = %Library
@onready var treeroot: TreeItem = $Tree.create_item()
@onready var song_progress: SongProgress = %SongProgress


func _ready() -> void:
	get_tree().set_auto_accept_quit(false)
	library.create_item()
	Stdin.command.connect(command)
	
	if read_cache():
		for song in songs.values():
			add_song(song)
	else:
		scan_directory(library_path)
	
	song_progress.modulate = Color(0, 0, 0, 0)
	%Player.visible = false
	
	$SettingsPanel/Settings.initialize(self)


func _physics_process(delta: float) -> void:
	var selected := library.get_selected()
	
	if selected && library.has_focus():
		var p := input(&"prev")
		var n := input(&"next")
		
		if p != n:
			var item: TreeItem
			
			if p:
				item = library.get_selected().get_prev_visible()
				
				if !item:
					item = library.get_root()
					
					while item.get_child_count() > 0:
						item = item.get_child(item.get_child_count() - 1)
			
			if n:
				item = library.get_selected().get_next_visible()
				
				if !item:
					item = library.get_root().get_first_child()
			
			item.select(0)
	
	if Input.is_action_just_pressed(&"hide details") and played_once:
		%Player.visible = not %Player.visible
	
	if Input.is_action_just_pressed(&"hide playlists"):
		%Playlists.visible = not %Playlists.visible
	
	%Repeat.custom_minimum_size.x = %ControlBar.size.y
	%Shuffle.custom_minimum_size.x = %ControlBar.size.y
	
	song_progress.modulate = Color(1, 1, 1, 1) if is_playing and song_duration > 0 else Color(0, 0, 0, 0)
	
	if is_playing and not is_pausing:
		song_position += delta
		
		if song_duration > 0 and song_position > song_duration:
			if %Repeat.icon == preload("res://src/repeat_one.png"):
				song_position = 0
			else:
				is_playing = false


func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		prints("EXIT")


func read_cache() -> bool:
	if FileAccess.file_exists(CACHE_FILE):
		var file := FileAccess.open(CACHE_FILE, FileAccess.READ)
		
		if file:
			var json = JSON.parse_string(file.get_as_text())
			
			if json:
				var songs_dict: Dictionary = json.songs
				
				for path in songs_dict:
					var song := Song.new()
					var song_dict: Dictionary = songs_dict[path]
					
					song.path = path
					song.title = song_dict.title
					song.artist = song_dict.artist
					song.album = song_dict.album
					song.lyrics = song_dict.lyrics.c_unescape()
					
					songs[path] = song
				
				library_path = json.path
				
				song_progress.background = json.progress_background
				
				return true
	
	return false


func save_cache() -> void:
	FileAccess.open(CACHE_FILE, FileAccess.WRITE).store_string(JSON.stringify({
		songs = serialize_songs(),
		path = library_path,
		progress_background = song_progress.background,
	}, "\t"))


func serialize_songs() -> Dictionary:
	var dict := {}
	
	for path in songs:
		var song := songs[path] as Song
		
		dict[path] = {
			title = song.title,
			artist = song.artist,
			album = song.album,
			lyrics = song.lyrics.c_escape(),
		}
	
	return dict


func scan_directory(path: String) -> void:
	var dir := DirAccess.open(path)
	
	if dir:
		var files := dir.get_files()
		var dirs := dir.get_directories()
		
		for file in files:
			match file.get_extension():
				"mp3":
					prints("READTAG", path.path_join(file))
		
		for directory in dirs:
			scan_directory(path.path_join(directory))


func input(event: StringName, d := 300000) -> bool:
	var t := Time.get_ticks_usec()
	
	if Input.is_action_just_pressed(event, true):
		inputs[event] = t
		return true
	else:
		return Input.is_action_pressed(event, true) && t - inputs[event] > d


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
	item.set_tooltip_text(0, song.path)
	
	song_items[song.path] = item
	
	reparent_item(item, get_album(get_artist(song.artist), song.album))


func read_tags(args: PackedStringArray) -> void:
	var mode := TagMode.NONE
	var song := Song.new()
	
	for section in args:
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
	
	add_song(song)


func command(string: String) -> void:
	var args := string.split(Stdin.DELIMETER)
	
	match args[0]:
		"EXIT":
			save_cache()
			get_tree().quit()
		
		"TAGOF":
			read_tags(args)
		
		"VOLUME":
			var volume := float(args[1]) * 100
			
			%VolumeLabel.text = str(roundi(volume)) + "%"
			%Volume.value = 100 - volume
		
		"DURATION":
			song_duration = float(args[1])
		
		"REPEAT":
			match args[1]:
				"none":
					%Repeat.icon = preload("res://src/no_repeat.png")
					%Repeat.tooltip_text = "No repeat (Toggle to repeat all songs in playlist)"
				
				"all":
					%Repeat.icon = preload("res://src/repeat.png")
					%Repeat.tooltip_text = "Repeat all songs in playlist (Toggle to repeat one song infinitely)"
				
				"one":
					%Repeat.icon = preload("res://src/repeat_one.png")
					%Repeat.tooltip_text = "Repeat one song infinitely (Toggle to disable repeating)"
		
		"SHUFFLE":
			%Shuffle.icon = SHUFFLE
		
		"NO_SHUFFLE":
			%Shuffle.icon = NO_SHUFFLE
		
		"RESUME":
			%PlayPauseResume.text = PAUSE_SYMBOL
			is_pausing = false
		
		"PAUSE":
			%PlayPauseResume.text = PLAY_SYMBOL
			is_pausing = true
		
		"REWIND":
			song_position -= float(args[1])
		
		"FAST_FORWARD":
			song_position += float(args[1])


func play_song(song: Song) -> void:
	prints("PLAY", song.path)
	
	%Player.visible = true
	%Title.text = song.title
	%Artist.text = "" if song.artist == "No Artist" else song.artist
	%TitleArtistConnector.visible = not %Artist.text.is_empty()
	%Album.text = "" if song.album == "No Album" else song.album
	%Lyrics.text = "" if song.lyrics == "No Lyrics" else song.lyrics
	%PlayPauseResume.text = PAUSE_SYMBOL
	
	played_once = true
	is_playing = true
	song_position = 0.0


func _on_library_item_activated() -> void:
	var item := library.get_selected()
	var song := get_song(item)
	
	if song:
		play_song(song)
	else:
		item.collapsed = !item.collapsed


func _on_library_item_selected() -> void:
	library.scroll_to_item(library.get_selected())


func _on_settings_pressed() -> void:
	$SettingsPanel/Settings.initialize()
	$SettingsPanel.popup_centered()


func _on_library_path_selector_dir_selected(dir: String) -> void:
	library_path = dir


func _on_reload_library_pressed() -> void:
	song_items.clear()
	artists.clear()
	songs.clear()
	library.clear()
	library.create_item()
	scan_directory(library_path)


func _on_play_pause_resume_pressed() -> void:
	is_pausing = not is_pausing
	
	print("PAUSE" if is_pausing else "RESUME")
	
	%PlayPauseResume.text = PLAY_SYMBOL if is_pausing else PAUSE_SYMBOL


func _on_repeat_pressed() -> void:
	print("TOGGLE_REPEAT")


func _on_shuffle_pressed() -> void:
	match %Shuffle.icon:
		NO_SHUFFLE:
			%Shuffle.icon = SHUFFLE
			print("SHUFFLE")
		
		SHUFFLE:
			%Shuffle.icon = NO_SHUFFLE
			print("NO_SHUFFLE")


func _on_rewind_pressed() -> void:
	print("REWIND")


func _on_fast_forward_pressed() -> void:
	print("FAST_FORWARD")


func _on_song_progress_seek(pct: float) -> void:
	song_position = pct * song_duration
	prints("SEEK", song_position)


func _on_to_begin_pressed() -> void:
	song_position = 0
	prints("SEEK", 0)


func _on_settings_close() -> void:
	$SettingsPanel.visible = false


func _on_close_app_pressed() -> void:
	print("EXIT_ALL")
