@tool
extends EditorPlugin

var update_check = preload("updater/update_check.gd").new()

func _enter_tree():
	add_child(update_check)
	update_check.check_update(get_plugin_version())

func _exit_tree():
	update_check.queue_free()
