## just a simple script to test how long (ms) it takes a packet to travel 
## from A -> Relay -> B -> Relay -> Back to A
extends Node

var pending_pings: Dictionary = {} # request_id -> send_time

@rpc("any_peer", "unreliable")
func ping_request(request_id: int):
	var sender = multiplayer.get_remote_sender_id()
	ping_response.rpc_id(sender, request_id)

@rpc("any_peer", "unreliable")
func ping_response(request_id: int):
	if request_id in pending_pings:
		var rtt_ms = (Time.get_ticks_msec() - pending_pings[request_id])
		print("RTT: %d ms" % rtt_ms)
		pending_pings.erase(request_id)

func measure_rtt(peer_id: int):
	var request_id = randi()
	pending_pings[request_id] = Time.get_ticks_msec()
	ping_request.rpc_id(peer_id, request_id)

func _process(_delta: float) -> void:
	if Input.is_action_just_pressed("ui_home"):
		measure_rtt(
			2
		)
