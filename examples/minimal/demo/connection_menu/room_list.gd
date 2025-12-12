extends ScrollContainer

@export var room_info_scene: PackedScene

func _ready() -> void:
	list_rooms()

func list_rooms() -> void:
	for c in get_children():
		c.queue_free()
	
	for room in await NodeTunnel.fetch_rooms(Multiplayer.APP_ID):
		var scene = room_info_scene.instantiate()
		scene.room_info = room
		add_child(scene)

func _on_refresh_list_pressed() -> void:
	list_rooms()
