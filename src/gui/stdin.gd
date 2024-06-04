extends Node

signal command(string: String)

var thread: Thread

func _ready() -> void:
	thread = Thread.new()
	
	thread.start(
		func ():
			while true:
				call_thread_safe(&"emit_signal", &"command", OS.read_string_from_stdin())
	)


func _exit_tree() -> void:
	print("EXIT")
	thread.wait_to_finish()
