class_name SongProgress
extends ProgressBar


signal seek(pct: float)


const BACKGROUND_NEVER := "never"
const BACKGROUND_HOVER := "hover"
const BACKGROUND_ALWAYS := "always"


var is_pressed := false
var background := BACKGROUND_ALWAYS
var hovering := false


func update(pos: Vector2) -> void:
	seek.emit(clampf(pos.x / get_rect().size.x, 0, 1))


func _process(_delta: float) -> void:
	match background:
		BACKGROUND_NEVER:
			$Panel.visible = false
		
		BACKGROUND_HOVER:
			$Panel.visible = hovering
		
		BACKGROUND_ALWAYS:
			$Panel.visible = true


func _input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		if is_pressed:
			update(event.position - get_global_rect().position)


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		match event.button_index:
			MOUSE_BUTTON_LEFT, MOUSE_BUTTON_RIGHT:
				is_pressed = event.pressed
				
				if is_pressed:
					update(event.position)
				
				accept_event()


func _on_mouse_entered() -> void:
	hovering = true


func _on_mouse_exited() -> void:
	hovering = false
