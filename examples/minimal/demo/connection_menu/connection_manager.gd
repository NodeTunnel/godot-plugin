extends VBoxContainer

@onready var room_name = $RoomName
@onready var room_region = $Region
@onready var room_id_in = $RoomId

func _on_host_pressed() -> void:
	await Multiplayer.setup_connection(room_region.selected)
	Multiplayer.nt_peer.host_room(true, room_name.text, 4)

func _on_join_pressed() -> void:
	var id = room_id_in.text
	var addr = Multiplayer.get_relay_addr(id[0])
	if addr.is_empty():
		push_error("malformed room ID, relay ID invalid: ", id)
	
	await Multiplayer.setup_connection(addr)
	Multiplayer.nt_peer.join_room(id)
