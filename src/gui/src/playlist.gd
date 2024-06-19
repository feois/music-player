class_name Playlist
extends Tree


var scene_root: Root
var root: TreeItem
var count: int:
	get: return root.get_child_count()
var first: TreeItem:
	get: return root.get_first_child()
var last: TreeItem:
	get: return root.get_child(count - 1) if count > 0 else null
var is_playing: bool:
	get: return scene_root.playing_playlist == self


func _ready() -> void:
	set_column_title(0, "Title")
	set_column_title(1, "Artist")
	set_column_title(2, "Album")
	set_column_title(3, "Path")
	
	root = create_item()


func _get_drag_data(at_position: Vector2) -> Variant:
	return get_item_at_position(at_position)


func _can_drop_data(_at_position: Vector2, data: Variant) -> bool:
	return data != null


func _drop_data(at_position: Vector2, data: Variant) -> void:
	var item := data as TreeItem
	
	if at_position.y < get_item_area_rect(first).position.y:
		move_before(item, first)
	elif at_position.y > get_item_area_rect(last).end.y:
		move_after(item, last)
	else:
		var on_item := get_item_at_position(at_position)
		
		if item != on_item:
			var rect := get_item_area_rect(on_item)
			
			if at_position.y < rect.position.y + rect.size.y / 2:
				move_before(item, on_item)
			else:
				move_after(item, on_item)


func _input(event: InputEvent) -> void:
	if visible and not (get_viewport().gui_get_focus_owner() is LineEdit):
		if event.is_action_pressed(&"delete"):
			var selected := get_selected()
			
			if selected:
				for i in range(count):
					if root.get_child(i) == selected:
						root.remove_child(selected)
						selected.free()
						
						if is_playing:
							prints("DELETE", i)
						
						break


func move_before(item: TreeItem, before_item: TreeItem) -> void:
	if before_item.get_prev() != item:
		var from := -1
		var to := -1
		
		for i in range(count):
			match root.get_child(i):
				item:
					from = i
				
				before_item:
					to = i
			
			if from != -1 and to != -1:
				break
		
		if to > from:
			to -= 1
		
		item.move_before(before_item)
		
		if is_playing:
			prints("MOVE", from, to)


func move_after(item: TreeItem, after_item: TreeItem) -> void:
	if after_item.get_next() != item:
		var from := -1
		var to := -1
		
		for i in range(count):
			match root.get_child(i):
				item:
					from = i
				
				after_item:
					to = i + 1
			
			if from != -1 and to != -1:
				break
		
		if to > from:
			to -= 1
		
		item.move_after(after_item)
		
		if is_playing:
			prints("MOVE", from, to)


func add_song(song: Song) -> void:
	var item := create_item()
	
	item.set_text(0, song.title)
	item.set_text(1, song.artist)
	item.set_text(2, song.album)
	item.set_text(3, song.path)
	
	if is_playing:
		prints("APPEND", song.path)


func validate(songs: Dictionary) -> void:
	for i in range(count - 1, -1, -1):
		var item := root.get_child(i)
		
		if songs.has(item.get_text(3)):
			var song := songs[item.get_text(3)] as Song
			
			item.set_text(0, song.title)
			item.set_text(1, song.artist)
			item.set_text(2, song.album)
		elif find_song(songs, i, item):
			root.remove_child(item)
			item.free()
			
			if is_playing:
				prints("DELETE", i)


func find_song(songs: Dictionary, i: int, item: TreeItem) -> bool:
	for path in songs:
		var song := songs[path] as Song
		
		if (song.title == item.get_text(0)
				and song.artist == item.get_text(1)
				and song.album == item.get_text(2)):
			item.set_text(3, song.path)
			
			if is_playing:
				prints("UPDATE", i, song.path)
			
			return false
	
	return true


func serialize() -> Array[Dictionary]:
	var array: Array[Dictionary] = []
	
	for item in root.get_children():
		array.append({
			title = item.get_text(0),
			artist = item.get_text(1),
			album = item.get_text(2),
			path = item.get_text(3),
		})
	
	return array


func deserialize(array: Array) -> void:
	for dict in array:
		var item := create_item()
		
		item.set_text(0, dict.title)
		item.set_text(1, dict.artist)
		item.set_text(2, dict.album)
		item.set_text(3, dict.path)


func _on_item_selected() -> void:
	scroll_to_item(get_selected())


func _on_item_activated() -> void:
	var item := get_selected()
	
	for i in range(count):
		if root.get_child(i) == item:
			scene_root.playing_playlist = self
			scene_root.play_song(i)
