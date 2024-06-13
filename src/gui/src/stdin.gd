extends Node

const DELIMETER := "::::"
const ENDLINE := ";;;;"

@warning_ignore("unused_signal")
signal command(string: String)

var thread: Thread

func _ready() -> void:
	thread = Thread.new()
	
	thread.start(
		func ():
			var s := ""
			
			while true:
				s += OS.read_string_from_stdin()
				
				if s.ends_with(ENDLINE + "\n"):
					s = s.substr(0, s.length() - ENDLINE.length() - 1)
					
					call_thread_safe(&"emit_signal", &"command", s)
					
					if s == "EXIT":
						break
					
					s = ""
	)
