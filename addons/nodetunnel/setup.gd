@tool
extends EditorPlugin

var update_check = preload("utils/update_check.gd").new()

func _enter_tree():
	var base_dir = get_script().resource_path.get_base_dir()
	add_autoload_singleton("NodeTunnel", base_dir.path_join("nodetunnel.gd"))
	
	add_child(update_check)
	update_check.check_update(get_plugin_version())

func _exit_tree():
	remove_autoload_singleton("NodeTunnel")
	update_check.queue_free()
