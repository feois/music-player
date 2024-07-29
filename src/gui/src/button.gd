@tool
extends TextureRect


signal pressed

@export var enabled := true:
	set(value):
		enabled = value
		$Button.value = !value
@export var margin: int:
	set(value):
		margin = value
		$MarginContainer.add_theme_constant_override(&"margin_left", value)
		$MarginContainer.add_theme_constant_override(&"margin_right", value)
		$MarginContainer.add_theme_constant_override(&"margin_top", value)
		$MarginContainer.add_theme_constant_override(&"margin_bottom", value)


func _process(_delta: float) -> void:
	$MarginContainer/TextureRect.texture = texture
	$MarginContainer/TextureRect.expand_mode = expand_mode
	$Button.tooltip_text = tooltip_text


func _on_button_pressed() -> void:
	pressed.emit()
