## just a simple script to test how long (ms) it takes a packet to travel 
## from A -> Relay -> B -> Relay -> Back to A
## expect to see responses around 80-100ms, remember, this is the time it takes
## for this peer to send a packet to the server, then the server to send a packet to the target peer
## then the target peer to send back to the relay server, then the relay server to send back to
## this peer. it's heavily dependant on the senders and targets distance from the relay server
extends Node

var pending_pings: Dictionary = {} # request_id -> send_time

@rpc("any_peer", "reliable")
func ping_request(request_id: int):
	var sender = multiplayer.get_remote_sender_id()
	ping_response.rpc_id(sender, request_id)

@rpc("any_peer", "reliable")
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
	if Input.is_action_just_pressed("ui_accept"):
		measure_rtt(2)
