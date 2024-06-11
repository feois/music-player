extends ProgressBar


var is_pressed := false


func update(pos: Vector2) -> void:
	var target := 1 - clampf(pos.y / get_rect().size.y, 0, 1)
	
	value = (1 - target) * 100
	
	%VolumeLabel.text = str(roundi(value)) + "%"
	
	prints("VOLUME", target)


func _input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		if is_pressed:
			update(event.position - get_global_rect().position)


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		if event.pressed:
			if event.button_index == MOUSE_BUTTON_LEFT or event.button_index == MOUSE_BUTTON_RIGHT:
				is_pressed = true
				
				if is_pressed:
					update(event.position)
				
				accept_event()
			elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
				print("VOLINC")
			elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
				print("VOLDEC")
		else:
			is_pressed = false
