class_name Selector
extends Button


signal double_click

@export var group: StringName
@export var normal: StyleBox
@export var normal_color := Color.WHITE
@export var hover: StyleBox
@export var hover_color := Color.WHITE
@export var selected: StyleBox
@export var selected_color := Color.WHITE


func _ready() -> void:
	SelectorManager.register(self)
	pressed.connect(select)
	update_stylebox()
	add_theme_stylebox_override(&"focus", StyleBoxEmpty.new())


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		if event.double_click and (event.button_mask & button_mask) != 0:
			double_click.emit()


func select() -> void:
	SelectorManager.select(self)


func update_stylebox() -> void:
	_update(&"normal", normal, normal_color)
	_update(&"hover", hover, hover_color)
	_update(&"pressed", selected, selected_color)
	_update(&"disabled", selected, selected_color)


func _update(style_name: StringName, style: StyleBox, color: Color) -> void:
	var color_name: StringName = &"font_%s_color" % style_name
	
	if has_theme_stylebox_override(style_name):
		remove_theme_stylebox_override(style_name)
	
	if has_theme_color_override(color_name):
		remove_theme_color_override(color_name)
	
	if style:
		add_theme_stylebox_override(style_name, style)
	
	if color:
		add_theme_color_override(color_name, color)
