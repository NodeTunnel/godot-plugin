extends VBoxContainer

@onready var room_name = $RoomName
@onready var room_region = $Region
@onready var room_id_in = $RoomId

func _on_host_pressed() -> void:
	await Multiplayer.setup_connection(room_region.selected)
	Multiplayer.nt_peer.host_room(true, "")
	print(Multiplayer.nt_peer.room_id)
	await Multiplayer.nt_peer.room_connected
	print(Multiplayer.nt_peer.room_id)
	
	multiplayer.peer_connected.connect(
		func(pid):
			print(pid, " has joined.")
	)

func _on_join_pressed() -> void:
	var id = room_id_in.text
	var addr = Multiplayer.get_relay_addr(id[0])
	
	await Multiplayer.setup_connection(addr)
	Multiplayer.nt_peer.join_room(id, '{"password": "123"}')
	await Multiplayer.nt_peer.room_connected
	print("joined room!!!")
