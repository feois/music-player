extends Node

@warning_ignore("unused_signal")
signal command(string: String)

var thread: Thread

func _ready() -> void:
	thread = Thread.new()
	
	thread.start(
		func ():
			while true:
				var s := OS.read_string_from_stdin().c_unescape()
				
				s = s.substr(0, s.length() - 1)
				
				call_thread_safe(&"emit_signal", &"command", s)
	)


func _exit_tree() -> void:
	print("EXIT")
	thread.wait_to_finish()
