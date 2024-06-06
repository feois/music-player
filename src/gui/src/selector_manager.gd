extends Node


var selected := {}
var connectors := {}


func _ready() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS


func register(selector: Selector) -> void:
	if not selected.has(selector.group):
		selected[selector.group] = null
	
	if not connectors.has(selector.group):
		connectors[selector.group] = []


func connect_group(group: StringName, connector: Callable) -> void:
	connectors[group] += [connector]


func get_selector(group: StringName) -> Selector:
	return selected[group]


func select(selector: Selector, callback := false) -> Selector:
	if selected[selector.group] != selector:
		var old := unselect(selector.group, callback)
		
		selector.disabled = true
		selector.grab_focus()
		selected[selector.group] = selector
		
		for connector in connectors[selector.group]:
			connector.call(selector, old)
		
		return old
	
	return null


func unselect(group: StringName, callback := true) -> Selector:
	var selector = selected[group]
	
	if selector:
		selector.disabled = false
	
	selected[group] = null
	
	if callback:
		for connector in connectors[group]:
			connector.call(null, selector)
	
	return selector
