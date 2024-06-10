class_name Selector
extends Button


signal double_click

@export var group := &""


func _ready() -> void:
	SelectorManager.register(self)
	pressed.connect(select)
	focus_entered.connect(select)


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		if event.double_click and (event.button_mask & button_mask) != 0:
			double_click.emit()


func select() -> void:
	SelectorManager.select(self)
