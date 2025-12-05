extends MultiplayerSpawner

@export var player_scene: PackedScene

func _ready() -> void:
	multiplayer.peer_connected.connect(
		func(pid):
			if !multiplayer.is_server():
				return
			
			spawn_player(pid)
	)

func spawn_player(peer_id: int):
	var player = player_scene.instantiate()
	player.name = str(peer_id)
	get_parent().add_child(player)

## We could *technically* use multiplayer.multiplayer_peer.room_connected instead of
## creating a new signal. However, it's not safe to do as Godot defaults to OfflineMultiplayerPeer.
## You will get errors if you try to access multiplayer_peer before updating multiplayer.multiplayer_peer
## to NodeTunnelPeer.
func _on_node_tunnel_demo_connected_to_room() -> void:
	if !multiplayer.is_server():
		return
	
	spawn_player(1)
