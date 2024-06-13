extends Panel


const CACHE_FILE := "user://cache.json"
const PLAY_SYMBOL := "⏵"
const PAUSE_SYMBOL := "⏸"

enum TagMode {
	NONE,
	PATH,
	TITLE,
	ALBUM,
	ARTIST,
	LYRICS,
	DURATION,
}

class Song extends RefCounted:
	var path: String
	var title: String
	var artist: String
	var album: String
	var lyrics: String
	var duration: int


var songs := {}
var artists := {}
var song_items := {}
var uninitalized_songs := {}
var inputs := {}

var is_pausing := false

var settings_library_path: TreeItem

@onready var default_library_path := OS.get_environment("USERPROFILE" if OS.has_feature("windows") else "HOME").path_join("Music")
@onready var library_path := default_library_path:
	set(value):
		library_path = value
		
		if settings_library_path:
			settings_library_path.set_text(1, value)
@onready var library: Tree = %Library
@onready var treeroot: TreeItem = $Tree.create_item()


func _ready() -> void:
	get_tree().set_auto_accept_quit(false)
	library.create_item()
	Stdin.command.connect(command)
	
	init_settings()
	
	if read_cache():
		for song in songs.values():
			add_song(song)
	else:
		scan_directory(library_path)


func _physics_process(_delta: float) -> void:
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
					song.duration = song_dict.duration
					
					songs[path] = song
				
				library_path = json.path
				
				return true
	
	return false


func save_cache() -> void:
	FileAccess.open(CACHE_FILE, FileAccess.WRITE).store_string(JSON.stringify({
		songs = serialize_songs(),
		path = library_path,
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
			duration = song.duration,
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
	
	song_items[song.path] = item
	
	reparent_item(item, get_album(get_artist(song.artist), song.album))


func read_tags(string: String) -> void:
	var mode := TagMode.NONE
	var song := Song.new()
	
	for section in string.split(Stdin.DELIMETER):
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
				
				"Duration":
					mode = TagMode.DURATION
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
				
				TagMode.DURATION:
					song.duration = int(section)
			
			mode = TagMode.NONE
	
	add_song(song)


func command(string: String) -> void:
	if string == "EXIT":
		save_cache()
		get_tree().quit()
	
	if string.begins_with("TAGOF"):
		read_tags(string)
	
	if string.begins_with("VOLUME"):
		var volume := string.split(Stdin.DELIMETER)[1]
		
		%VolumeLabel.text = volume + "%"
		%Volume.value = 100 - int(volume)


func _on_library_item_activated() -> void:
	var item := library.get_selected()
	var song := get_song(item)
	
	if song:
		prints("PLAY", song.path)
		
		%Title.text = song.title
		%Artist.text = song.artist
		%Album.text = song.album
		%Lyrics.text = song.lyrics
		%PlayPauseResume.text = PAUSE_SYMBOL
	else:
		item.collapsed = !item.collapsed


func _on_library_item_selected() -> void:
	library.scroll_to_item(library.get_selected())


func init_settings() -> void:
	var settings := $SettingsPanel/Panel/Settings as Tree
	var root := settings.create_item()
	
	settings_library_path = root.create_child()
	settings_library_path.set_text(0, "Music library path")
	settings_library_path.set_editable(1, true)
	settings_library_path.set_selectable(0, false)
	settings_library_path.add_button(0, preload("res://src/small-icon.svg"))
	settings_library_path.add_button(1, preload("res://src/small-icon.svg"))


func _on_settings_pressed() -> void:
	settings_library_path.set_text(1, library_path)
	
	$SettingsPanel.visible = true


func _on_settings_button_clicked(item: TreeItem, column: int, _id: int, _mouse_button_index: int) -> void:
	match item:
		settings_library_path:
			if column == 0:
				library_path = default_library_path
			else:
				$SettingsPanel/LibraryPathSelector.popup_centered()


func _on_library_path_selector_dir_selected(dir: String) -> void:
	library_path = dir


func _on_settings_panel_close_requested() -> void:
	$SettingsPanel.visible = false


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
