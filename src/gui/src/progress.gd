class_name SongProgress
extends ProgressBar


signal seek(pct: float)
signal update_text(pct: float)


const BACKGROUND_NEVER := "never"
const BACKGROUND_HOVER := "hover"
const BACKGROUND_ALWAYS := "always"


var is_pressed := false
var background := BACKGROUND_ALWAYS
var hovering := false


func get_percent(pos: Vector2) -> float:
	return clampf(pos.x / get_rect().size.x, 0, 1)


func set_text(text: String) -> void:
	$PanelContainer/Label.text = text


func _process(_delta: float) -> void:
	match background:
		BACKGROUND_NEVER:
			$Panel.visible = false
		
		BACKGROUND_HOVER:
			$Panel.visible = hovering
		
		BACKGROUND_ALWAYS:
			$Panel.visible = true
	
	$PanelContainer.visible = hovering


func _input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		if is_pressed:
			seek.emit(get_percent(event.position - get_global_rect().position))


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		match event.button_index:
			MOUSE_BUTTON_LEFT, MOUSE_BUTTON_RIGHT:
				is_pressed = event.pressed
				
				if is_pressed:
					seek.emit(get_percent(event.position))
				
				accept_event()
	
	if event is InputEventMouseMotion:
		$PanelContainer.position.x = event.position.x - $PanelContainer.size.x / 2
		update_text.emit(get_percent(event.position))


func _on_mouse_entered() -> void:
	hovering = true


func _on_mouse_exited() -> void:
	hovering = false
