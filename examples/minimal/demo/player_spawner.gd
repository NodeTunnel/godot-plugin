extends MultiplayerSpawner

@export var player_scene: PackedScene

func _ready() -> void:
	if !multiplayer.is_server():
		return
	
	var server_player = player_scene.instantiate()
	server_player.name = "1"
	add_child(server_player)
	
	multiplayer.peer_connected.connect(
		func(pid):
			var player = player_scene.instantiate()
			player.name = str(pid)
			add_child(player)
	)
