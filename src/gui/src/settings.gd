extends Panel


signal close


const PROGRESS_BACKGROUNDS := [
	SongProgress.BACKGROUND_NEVER,
	SongProgress.BACKGROUND_HOVER,
	SongProgress.BACKGROUND_ALWAYS,
]


var root: Root


func _process(_delta: float) -> void:
	%LibraryPathOpen.custom_minimum_size.x = %LibraryPathOpen.size.y


func initialize(r: Root) -> void:
	root = r
	
	%LibraryPath.text = r.library_path
	%ProgressBackground.selected = PROGRESS_BACKGROUNDS.find(r.song_progress.background)


func _on_close_pressed() -> void:
	initialize(root)
	close.emit()


func _on_library_path_open_pressed() -> void:
	$LibraryPathSelector.popup_centered()


func _on_library_path_selector_dir_selected(dir: String) -> void:
	%LibraryPath.text = dir


func _on_reset_library_path_pressed() -> void:
	%LibraryPath.text = root.default_library_path


func _on_save_pressed() -> void:
	root.library_path = %LibraryPath.text
	root.song_progress.background = PROGRESS_BACKGROUNDS[%ProgressBackground.selected]
	
	root.save_cache()


func _on_restore_all_pressed() -> void:
	_on_reset_library_path_pressed()
