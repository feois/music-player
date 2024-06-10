extends Panel


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
	var item: TreeItem


var songs := {}
var artists := {}
var inputs := {}

@onready var library: Tree = %Library
@onready var treeroot: TreeItem = $Tree.create_item()


func _ready() -> void:
	library.create_item()
	opendir(
		"/home/wilson/Music/Avicii"
		#"Z:/home/wilson/Music/Avicii"
		#"C:/Users/Admin/Music/Avicii"
	)
	Stdin.command.connect(command)


func _physics_process(_delta: float) -> void:
	var selected := library.get_selected()
	
	if selected && library.has_focus():
		var p := input(&"prev song")
		var n := input(&"next song")
		
		if p != n:
			if p:
				library.get_selected().get_prev_in_tree(true).select(0)
			
			if n:
				library.get_selected().get_next_in_tree(true).select(0)


func input(event: StringName, d := 300000) -> bool:
	var t := Time.get_ticks_usec()
	
	if Input.is_action_just_pressed(event):
		inputs[event] = t
		return true
	else:
		return Input.is_action_pressed(event) && t - inputs[event] > d


func opendir(path: String) -> void:
	var dir := DirAccess.open(path)
	
	if dir:
		var files := dir.get_files()
		var dirs := dir.get_directories()
		
		for file in files:
			if file.ends_with(".mp3"):
				var song := Song.new()
				
				song.path = path.path_join(file)
				song.item = treeroot.create_child()
				song.item.set_meta(&"songres", song)
				song.item.set_text(0, song.path)
				
				songs[song.path] = song
				
				prints("READTAG", song.path)
		
		for directory in dirs:
			opendir(path.path_join(directory))


func reparent_item(item: TreeItem, parent: TreeItem, f: Callable) -> void:
	if parent.get_child_count() > 0:
		var index := parent.get_children().bsearch_custom(item, f)
		
		if index < parent.get_child_count():
			item.move_before(parent.get_child(index))
			return
	
	treeroot.remove_child(item)
	parent.add_child(item)


func get_artist(artist: String) -> TreeItem:
	if artists.has(artist):
		return artists[artist]
	
	var item := treeroot.create_child()
	
	item.set_text(0, artist)
	
	reparent_item(item, library.get_root(), func (x: TreeItem, y: TreeItem) -> bool:
		return x.get_text(0) < y.get_text(0))
	
	artists[artist] = item
	
	return item


func get_song(item: TreeItem) -> Song:
	return item.get_meta(&"songres") as Song if item.has_meta(&"songres") else null


func add_song(song: Song) -> void:
	reparent_item(song.item, get_artist(song.artist), func (x: TreeItem, y: TreeItem) -> bool:
		return get_song(x).title < get_song(y).title)
	song.item.set_text(0, song.title)


func read_tag(string: String) -> void:
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
			
			mode = TagMode.NONE
	
	add_song(song)

func command(string: String) -> void:
	if string.begins_with("TAGOF"):
		read_tag(string)
	
	if string.begins_with("VOLUME"):
		var volume := string.split(DELIMETER)[1]
		
		%VolumeLabel.text = volume + "%"
		%Volume.value = int(volume)


func _on_library_item_activated() -> void:
	var item := library.get_selected()
	var song := get_song(item)
	
	if song:
		prints("PLAY", song.path)
	else:
		item.collapsed = !item.collapsed


func _on_library_item_selected() -> void:
	library.scroll_to_item(library.get_selected())
