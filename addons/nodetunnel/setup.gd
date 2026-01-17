@tool
extends EditorPlugin

var update_check = preload("updater/update_check.gd").new()
var gd_wrapper_name = "NodeTunnel"
var gd_wrapper_path = "autoloads/gd_wrapper.gd"

func _enter_tree():
	add_autoload_singleton(gd_wrapper_name, gd_wrapper_path)
	
	add_child(update_check)
	update_check.check_update(get_plugin_version())

func _exit_tree():
	remove_autoload_singleton(gd_wrapper_name)
	update_check.queue_free()
