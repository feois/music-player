extends Node

const DELIMETER := "::::"
const ENDLINE := ";;;;"

func listen(node: Node) -> void:
	Thread.new().start(
		func ():
			var s := ""
			
			while true:
				s += OS.read_string_from_stdin()
				
				if s.ends_with(ENDLINE + "\n"):
					s = s.rstrip(ENDLINE + "\n")
					
					node.call_thread_safe(&"command", s)
					
					if s == "EXIT":
						break
					
					s = ""
	)
