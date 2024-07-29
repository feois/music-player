extends Node

func listen(root: Root) -> void:
	Thread.new().start(
		func ():
			var s := ""
			
			while true:
				var line := OS.read_string_from_stdin().trim_suffix("\n")
				
				if line.is_empty():
					if s == "EXIT":
						if not root.ready_to_exit:
							get_tree().create_timer(5).timeout.connect(func () -> void: get_tree().quit())
					
					root.call_thread_safe(&"command", s)
					
					if s == "EXIT":
						break
					
					s = ""
				else:
					s += line
	)
