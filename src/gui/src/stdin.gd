extends Node

signal command(string: String)

var thread: Thread

func _ready() -> void:
	thread = Thread.new()
	
	#return
	
	thread.start(
		func ():
			while true:
				var s := OS.read_string_from_stdin()
				call_thread_safe(&"emit_signal", &"command", s)
	)


func _exit_tree() -> void:
	print("EXIT")
	thread.wait_to_finish()
