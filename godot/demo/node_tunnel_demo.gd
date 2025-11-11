extends Node2D

@export var player_scene: PackedScene

@onready var host_id = $UI/HostID
@onready var peer := NodeTunnelPeer.new()

func _ready() -> void:
	peer.forced_disconnect.connect(
		func():
			$UI.show()
	)

func _on_host_pressed() -> void:
	peer.connect_to_relay("127.0.0.1:8080", "nodetunnel_demo")
	peer.host_room()
	multiplayer.multiplayer_peer = peer
	
	peer.room_connected.connect(
		func(room_id: String):
			DisplayServer.clipboard_set(room_id)
			
			var server_player = player_scene.instantiate()
			server_player.name = str(1)
			add_child(server_player)
	)
	
	multiplayer.peer_connected.connect(
		func(pid):
			var scene = player_scene.instantiate()
			scene.name = str(pid)
			add_child(scene)
	)
	
	multiplayer.peer_disconnected.connect(
		func(pid):
			var node = get_node_or_null(str(pid))
			
			if node != null:
				node.queue_free()
	)
	
	$UI.hide()

func _on_join_pressed() -> void:
	peer.connect_to_relay("127.0.0.1:8080", "nodetunnel_demo")
	peer.join_room(host_id.text)
	multiplayer.multiplayer_peer = peer
	
	$UI.hide()

func _process(_delta: float) -> void:
	if Input.is_action_just_pressed("ui_accept"):
		network_print.rpc("Hello world")
	
	if Input.is_action_just_pressed("ui_cancel"):
		multiplayer.multiplayer_peer.close()
		$UI.show()

@rpc("any_peer", "reliable")
func network_print(msg: String):
	print("Message For ", multiplayer.get_unique_id(), ": ", msg)
