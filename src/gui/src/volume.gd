extends ProgressBar


var is_pressed := false


func update(pos: Vector2) -> void:
	var target := 1 - clampf(pos.y / get_rect().size.y, 0, 1)
	
	value = (1 - target) * 100
	
	%VolumeLabel.text = str(roundi(target * 100)) + "%"
	
	prints("VOLUME", target)


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
			
			MOUSE_BUTTON_WHEEL_UP:
				if event.pressed:
					print("VOLINC")
			
			MOUSE_BUTTON_WHEEL_DOWN:
				if event.pressed:
					print("VOLDEC")
