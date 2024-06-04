class_name Song
extends Selector


var path: String

var title := "":
	set(value):
		title = value
		text = title
var album := ""
var artist := ""
var lyrics := ""
var synced := ""
