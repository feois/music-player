extends Node

@warning_ignore("unused_signal")
signal command(string: String)

var thread: Thread

func _ready() -> void:
	thread = Thread.new()
	
	thread.start(
		func ():
			var s := ""
			
			while s != "EXIT":
				s = OS.read_string_from_stdin()
				s = s.c_unescape()
				s = s.substr(0, s.length() - 1)
				
				call_thread_safe(&"emit_signal", &"command", s)
	)
