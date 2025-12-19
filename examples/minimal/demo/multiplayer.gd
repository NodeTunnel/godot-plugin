## multiplayer.gd (auto-load Multiplayer)

extends Node

const APP_ID = "w5wbqd0970k84q4jo9o1"
const RELAYS = [
	{"addr": "45.33.64.148:8080", "prefix": "E"},
	{"addr": "127.0.0.1:8080", "prefix": "L"}
]

@onready var nt_peer = NodeTunnelPeer.new()

func setup_connection(relay_idx: int):
	var addr = RELAYS[relay_idx]["addr"]
	print("authenticating with relay: ", addr)
	
	nt_peer.connect_to_relay(addr, APP_ID)
	multiplayer.multiplayer_peer = nt_peer
	
	await nt_peer.authenticated
	print("authenticated")

func update_room_metadata():
	nt_peer.update_room("players: 1")

func get_relay_addr(prefix: String) -> String:
	for relay in RELAYS:
		if relay["prefix"] == prefix:
			return relay["addr"]
	return ""

func _process(delta: float) -> void:
	if Input.is_action_just_pressed("ui_right"):
		update_room_metadata()
