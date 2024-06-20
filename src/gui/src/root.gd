class_name Root
extends Panel


const SHUFFLE_MODULATE := Color(1, 1, 1)
const NO_SHUFFLE_MODULATE := Color(0.5, 0.5, 0.5)


enum TagMode {
	NONE,
	PATH,
	TITLE,
	ALBUM,
	ARTIST,
	LYRICS,
}

enum PlayState {
	IDLE,
	PLAY,
	PAUSE,
}


var songs := {}
var artists := {}
var albums := {}

var inputs := {}
var grouped_by_artists := true
var last_played: Song:
	set(value):
		last_played = value
		
		if value:
			show_song(value)
		else:
			%SongDetailsBox.visible = false
var last_focus: Control
var opening_playlist_path: String
var playing_playlist: Playlist:
	set(value):
		if playing_playlist != value:
			print("DELETE_ALL")
			
			for item in value.root.get_children():
				prints("APPEND", item.get_text(3))
			
			if playing_playlist:
				playlists.set_tab_icon(playlists.get_children().find(playing_playlist), null)
			
			playlists.set_tab_icon(playlists.get_children().find(value), preload("res://src/play.svg"))
			
			playing_playlist = value
var current_playlist: Playlist:
	get: return playlists.get_current_tab_control() as Playlist


var mute := false:
	set(value):
		mute = value
		
		if is_node_ready():
			%VolumeIcon.texture = preload("res://src/mute.svg") if mute else preload("res://src/volume.svg")
var play_state := PlayState.IDLE:
	set(value):
		play_state = value
		
		var generic_control := true
		
		match value:
			PlayState.IDLE:
				song_position = 0.0
				%ControlBar/PlayPauseResume.texture = preload("res://src/big_play.svg")
				%ControlBar/PlayPauseResume.enabled = last_played != null
				generic_control = false
			
			PlayState.PLAY:
				%ControlBar/PlayPauseResume.texture = preload("res://src/pause.svg")
			
			PlayState.PAUSE:
				%ControlBar/PlayPauseResume.texture = preload("res://src/big_play.svg")
		
		for control in [
				%ControlBar/ToBegin,
				%ControlBar/Rewind,
				%ControlBar/FastForward,
				%ControlBar/StopSong,
				%ControlBar/ToEnd,
		]:
			control.enabled = generic_control
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
var cache_path := "user://"
var cache_file_path: String:
	get: return cache_path.path_join("gui.json")
var playlists_dir_path: String:
	get: return cache_path.path_join("playlists")

@onready var default_library_path := OS.get_environment("USERPROFILE" if OS.has_feature("windows") else "HOME").path_join("Music")
@onready var library_path := default_library_path
@onready var library: Tree = %Library
@onready var treeroot: TreeItem = $Tree.create_item()
@onready var song_progress: SongProgress = %SongProgress
@onready var playlists: TabContainer = %Playlists/TabContainer


const ARG_CACHE_PATH := "--cache-path="
const ARG_SONG_PATH := "--song-path="
const ARG_SONG_DURATION := "--song-duration="
const ARG_SONG_POSITION := "--song-position="
const ARG_LAST_SONG := "--last-song="


func _ready() -> void:
	get_tree().set_auto_accept_quit(false)
	library.create_item()
	
	var song_path = null
	var state := PlayState.IDLE
	
	for arg in OS.get_cmdline_args():
		if arg.begins_with(ARG_CACHE_PATH):
			cache_path = arg.lstrip(ARG_CACHE_PATH)
		
		if arg.begins_with(ARG_SONG_PATH):
			song_path = arg.lstrip(ARG_SONG_PATH)
			state = PlayState.PLAY
		
		if arg.begins_with(ARG_SONG_DURATION):
			song_duration = float(arg.lstrip(ARG_SONG_DURATION))
		
		if arg.begins_with(ARG_SONG_POSITION):
			song_position = float(arg.lstrip(ARG_SONG_POSITION))
		
		if arg.begins_with(ARG_LAST_SONG):
			song_path = arg.lstrip(ARG_LAST_SONG)
		
		if arg == "--paused":
			play_state = PlayState.PAUSE
	
	song_progress.modulate = Color(0, 0, 0, 0)
	%SongDetailsBox.visible = false
	playlists.get_tab_bar().drag_to_rearrange_enabled = true
	
	if read_cache():
		for song in songs.values():
			add_song(song)
	else:
		scan_directory(library_path)
	
	if song_path != null and songs.has(song_path):
		last_played = songs[song_path]
	
	play_state = state
	
	if playlists.get_child_count() < 1:
		_on_new_playlist_name_text_submitted("Default")
	
	$SettingsPanel/Settings.initialize(self)
	
	get_viewport().gui_focus_changed.connect(func (control: Control) -> void:
		if control != %Search:
			last_focus = control)
	
	Stdin.listen(self)


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
	
	if Input.is_action_just_pressed(&"hide details") and last_played:
		%SongDetailsBox.visible = not %SongDetailsBox.visible
	
	if Input.is_action_just_pressed(&"hide playlists"):
		%Playlists.visible = not %Playlists.visible
	
	if Input.is_action_just_pressed(&"search"):
		%Search.grab_focus()
	
	song_progress.modulate = Color(1, 1, 1, 1) if play_state != PlayState.IDLE and song_duration > 0 else Color(0, 0, 0, 0)
	
	if play_state == PlayState.PLAY:
		song_position += delta
		
		if song_duration > 0 and song_position > song_duration:
			if %ControlBar/Repeat.texture == preload("res://src/repeat_one.svg"):
				song_position = 0
			else:
				play_state = PlayState.IDLE


func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		prints("EXIT")


func read_cache() -> bool:
	if FileAccess.file_exists(cache_file_path):
		var file := FileAccess.open(cache_file_path, FileAccess.READ)
		
		if file:
			var json = JSON.parse_string(file.get_as_text())
			
			if json:
				for dict in json.songs:
					var song := Song.deserialize(dict)
					
					songs[song.path] = song
				
				library_path = json.path
				
				song_progress.background = json.progress_background
				grouped_by_artists = json.grouping
				
				for array in json.playlists:
					var playlist := preload("res://src/playlist.tscn").instantiate() as Playlist
					
					playlist.scene_root = self
					
					playlists.add_child(playlist)
					playlist.deserialize(array)
				
				var pn := json.playlists_names as Array
				
				for i in range(pn.size()):
					playlists.set_tab_title(i, pn[i])
				
				playlists.current_tab = json.focused_playlist
				playing_playlist = playlists.get_child(json.playing_playlist)
				
				return true
	
	return false


func save_cache() -> void:
	FileAccess.open(cache_file_path, FileAccess.WRITE).store_string(JSON.stringify({
		songs = serialize_songs(),
		path = library_path,
		progress_background = song_progress.background,
		grouping = grouped_by_artists,
		focused_playlist = playlists.current_tab,
		playing_playlist = playlists.get_children().find(playing_playlist),
		playlists = serialize_playlists(),
		playlists_names = playlists_names(),
	}, "\t"))


func serialize_songs() -> Array[Dictionary]:
	var array: Array[Dictionary] = []
	
	for song in songs:
		array.append(songs[song].serialize())
	
	return array


func serialize_playlists() -> Array[Array]:
	var array: Array[Array] = []
	
	for playlist in playlists.get_children():
		array.append(playlist.serialize())
	
	return array


func playlists_names() -> Array[String]:
	var array: Array[String] = []
	
	for i in range(playlists.get_tab_count()):
		array.append(playlists.get_tab_title(i))
	
	return array


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


func clear_library() -> void:
	artists.clear()
	albums.clear()
	library.clear()
	library.create_item()


func get_artist(artist: String) -> TreeItem:
	if artists.has(artist):
		return artists[artist]
	
	var item := treeroot.create_child()
	
	item.set_text(0, artist)
	item.set_meta(&"albums", {})
	
	reparent_item(item, library.get_root())
	
	artists[artist] = item
	
	return item


func get_artist_album(artist: TreeItem, album: String) -> TreeItem:
	var d := artist.get_meta(&"albums", {}) as Dictionary
	
	if d.has(album):
		return d[album]
	
	var item := treeroot.create_child()
	
	item.set_text(0, album)
	
	reparent_item(item, artist)
	
	d[album] = item
	
	artist.set_meta(&"albums", d)
	
	return item


func get_album(album: String) -> TreeItem:
	if albums.has(album):
		return albums[album]
	
	var item := treeroot.create_child()
	
	item.set_text(0, album)
	item.set_meta(&"artists", {})
	
	reparent_item(item, library.get_root())
	
	albums[album] = item
	
	return item


func get_album_artist(album: TreeItem, artist: String) -> TreeItem:
	var d := album.get_meta(&"artists") as Dictionary
	
	if d.has(artist):
		return d[artist]
	
	var item := treeroot.create_child()
	
	item.set_text(0, artist)
	
	reparent_item(item, album)
	
	d[artist] = item
	
	album.set_meta(&"artists", d)
	
	return item


func get_song(item: TreeItem) -> Song:
	return item.get_meta(&"songres") as Song if item.has_meta(&"songres") else null


func add_song(song: Song) -> void:
	if %Search.text.is_empty() or song.filter(%Search.text):
		var item := treeroot.create_child()
		var parent := (
			get_artist_album(get_artist(song.artist), song.album)
			if grouped_by_artists
			else get_album_artist(get_album(song.album), song.artist))
		
		item.set_text(0, song.title)
		item.add_button(0, preload("res://src/small_plus.svg"))
		item.set_meta(&"songres", song)
		item.set_tooltip_text(0, song.path)
		
		reparent_item(item, parent)


func read_tags(args: String) -> void:
	var mode := TagMode.NONE
	var song := Song.new()
	
	for section in args.split(Stdin.DELIMETER):
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
	var args := string.split(Stdin.DELIMETER, false, 2)
	
	match args[0]:
		"EXIT":
			save_cache()
			get_tree().quit()
		
		"TAGOF":
			read_tags(string)
		
		"PLAY":
			last_played = songs[args[1]]
			song_position = 0
			play_state = PlayState.PLAY
		
		"VOLUME":
			var volume := float(args[1]) * 100
			
			%VolumeLabel.text = str(roundi(volume)) + "%"
			%Volume.value = 100 - volume
		
		"MUTE":
			mute = true
		
		"UNMUTE":
			mute = false
		
		"DURATION":
			song_duration = float(args[1])
		
		"REPEAT":
			match args[1]:
				"none":
					%ControlBar/Repeat.texture = preload("res://src/no_repeat.svg")
					%ControlBar/Repeat.tooltip_text = "No repeat (Toggle to repeat all songs in playlist)"
				
				"all":
					%ControlBar/Repeat.texture = preload("res://src/repeat.svg")
					%ControlBar/Repeat.tooltip_text = "Repeat all songs in playlist (Toggle to repeat one song infinitely)"
				
				"one":
					%ControlBar/Repeat.texture = preload("res://src/repeat_one.svg")
					%ControlBar/Repeat.tooltip_text = "Repeat one song infinitely (Toggle to stop after every song)"
				
				"stop":
					%ControlBar/Repeat.texture = preload("res://src/stop.svg")
					%ControlBar/Repeat.tooltip_text = "Stop after every song is finished (Toggle to disable repeating)"
		
		"SHUFFLE":
			%ControlBar/Shuffle.modulate = SHUFFLE_MODULATE
		
		"NO_SHUFFLE":
			%ControlBar/Shuffle.modulate = NO_SHUFFLE_MODULATE
		
		"RESUME":
			play_state = PlayState.PLAY
		
		"PAUSE":
			play_state = PlayState.PAUSE
		
		"STOP":
			play_state = PlayState.IDLE
		
		"REWIND":
			song_position -= float(args[1])
		
		"FAST_FORWARD":
			song_position += float(args[1])


func show_song(song: Song) -> void:
	%SongDetailsBox.visible = true
	%Title.text = song.title
	%Artist.text = "" if song.artist == "No Artist" else song.artist
	%TitleArtistConnector.visible = not %Artist.text.is_empty()
	%Album.text = "" if song.album == "No Album" else song.album
	%Lyrics.text = "" if song.lyrics == "No Lyrics" else song.lyrics


func play_song(index: int) -> void:
	prints("PLAY", index)
	
	song_position = 0
	last_played = songs[current_playlist.root.get_child(index).get_text(3)] as Song
	play_state = PlayState.PLAY


func _on_library_item_activated() -> void:
	var item := library.get_selected()
	var song := get_song(item)
	
	if song:
		if Input.is_action_pressed(&"silent_add"):
			current_playlist.add_song(song)
		else:
			var index := current_playlist.root.get_child_count()
			
			playing_playlist = current_playlist
			current_playlist.add_song(song)
			play_song(index)
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
	songs.clear()
	clear_library()
	scan_directory(library_path)


func _on_play_pause_resume_pressed() -> void:
	match play_state:
		PlayState.IDLE:
			play_state = PlayState.PLAY
			print("REPLAY")
		
		PlayState.PLAY:
			play_state = PlayState.PAUSE
			print("PAUSE")
		
		PlayState.PAUSE:
			play_state = PlayState.PLAY
			print("RESUME")


func _on_repeat_pressed() -> void:
	print("TOGGLE_REPEAT")


func _on_shuffle_pressed() -> void:
	match %ControlBar/Shuffle.modulate:
		NO_SHUFFLE_MODULATE:
			%ControlBar/Shuffle.modulate = SHUFFLE_MODULATE
			print("SHUFFLE")
		
		SHUFFLE_MODULATE:
			%ControlBar/Shuffle.modulate = NO_SHUFFLE_MODULATE
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


func _on_stop_song_pressed() -> void:
	play_state = PlayState.IDLE
	print("STOP")


func _on_library_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		if event.pressed and event.button_index == MOUSE_BUTTON_RIGHT:
			$LibraryGroup.set_item_checked(0, grouped_by_artists)
			$LibraryGroup.set_item_checked(1, not grouped_by_artists)
			$LibraryGroup.popup(Rect2i(get_local_mouse_position(), Vector2.ZERO))
	
	if event is InputEventWithModifiers:
		if event.is_action_pressed(&"ui_accept") and event.shift_pressed:
			var item := library.get_selected()
			
			if item:
				current_playlist.add_song(get_song(item))


func _on_library_group_id_pressed(id: int) -> void:
	if grouped_by_artists:
		if id == 1:
			grouped_by_artists = false
			
			clear_library()
			
			for song in songs:
				add_song(songs[song])
	elif id == 0:
		grouped_by_artists = true
		
		clear_library()
		
		for song in songs:
			add_song(songs[song])


func _on_search_text_changed(_new_text: String) -> void:
	clear_library()
	
	for song in songs:
		add_song(songs[song])


func _on_search_text_submitted(_new_text: String) -> void:
	if last_focus:
		last_focus.grab_focus()


func _on_new_playlist_pressed() -> void:
	$NewPlaylistDialog.popup_centered()
	$NewPlaylistDialog/NewPlaylistName.text = ""
	$NewPlaylistDialog/NewPlaylistName.grab_focus()


func _on_delete_playlist_pressed() -> void:
	if playlists.get_child_count() > 1:
		if playing_playlist == current_playlist:
			print("DELETE_ALL")
		
		current_playlist.queue_free()


func _on_new_playlist_name_text_submitted(new_text: String) -> void:
	var playlist := preload("res://src/playlist.tscn").instantiate() as Playlist
	var index := playlists.get_child_count()
	
	playlist.scene_root = self
	playlists.add_child(playlist)
	playlists.set_tab_title(index, new_text)
	playlists.current_tab = index
	
	_on_new_playlist_dialog_close_requested()


func _on_new_playlist_dialog_confirmed() -> void:
	_on_new_playlist_name_text_submitted($NewPlaylistDialog/NewPlaylistName.text)


func _on_new_playlist_dialog_close_requested() -> void:
	$NewPlaylistDialog.visible = false


func _on_playlist_name_text_submitted(new_text: String) -> void:
	playlists.set_tab_title(playlists.current_tab, new_text)
	_on_rename_playlist_dialog_close_requested()


func _on_rename_playlist_dialog_confirmed() -> void:
	_on_playlist_name_text_submitted($RenamePlaylistDialog/PlaylistName.text)


func _on_rename_playlist_dialog_close_requested() -> void:
	$RenamePlaylistDialog.visible = false


func _on_rename_playlist_pressed() -> void:
	$RenamePlaylistDialog.popup_centered()
	$RenamePlaylistDialog/PlaylistName.text = ""
	$RenamePlaylistDialog/PlaylistName.grab_focus()


func _on_save_playlist_pressed() -> void:
	$SavePlaylistDialog.popup_centered()


func _on_save_playlist_dialog_file_selected(path: String) -> void:
	var file := FileAccess.open(path, FileAccess.WRITE)
	
	if file:
		file.store_string(JSON.stringify(current_playlist.serialize(), '\t'))


func _on_open_playlist_dialog_file_selected(path: String) -> void:
	opening_playlist_path = path
	$ImportPlaylistNameDialog.popup_centered()
	$ImportPlaylistNameDialog/ImportPlaylistName.text = ""
	$ImportPlaylistNameDialog/ImportPlaylistName.grab_focus()


func _on_open_playlist_pressed() -> void:
	$OpenPlaylistDialog.popup_centered()


func _on_import_playlist_name_text_submitted(new_text: String) -> void:
	var file := FileAccess.open(opening_playlist_path, FileAccess.READ)
	
	if file:
		var json = JSON.parse_string(file.get_as_text())
		
		if json:
			var playlist := preload("res://src/playlist.tscn").instantiate() as Playlist
			var index := playlists.get_child_count()
			
			playlist.scene_root = self
			
			playlists.add_child(playlist)
			playlists.set_tab_title(index, new_text)
			playlists.current_tab = index
			
			playlist.deserialize(json)
			
			_on_import_playlist_name_dialog_close_requested()


func _on_import_playlist_name_dialog_confirmed() -> void:
	_on_import_playlist_name_text_submitted($ImportPlaylistNameDialog/ImportPlaylistName.text)


func _on_import_playlist_name_dialog_close_requested() -> void:
	$ImportPlaylistNameDialog.visible = false


func _on_previous_song_pressed() -> void:
	print("PREV")


func _on_to_end_pressed() -> void:
	play_state = PlayState.IDLE
	print("SKIP")


func _on_library_button_clicked(item: TreeItem, _column: int, id: int, mouse_button_index: int) -> void:
	current_playlist.add_song(get_song(item))


func _on_volume_icon_pressed() -> void:
	if %VolumeIcon.texture == preload("res://src/mute.svg"):
		mute = false
		print("UNMUTE")
	else:
		mute = true
		print("MUTE")
