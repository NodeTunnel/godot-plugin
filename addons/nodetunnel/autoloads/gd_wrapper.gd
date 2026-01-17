extends Node

## A reference to the [NodeTunnelPeer] that this wrapper uses.
var peer := NodeTunnelPeer.new()

## Whether or not the peer has been connected to the relay
var authenticated = false

## Whether or not the peer is in a room
var in_room = false

## Creates a connecting between this client and a given relay server.
## Must have a valid application token from nodetunnel.io in order to connect
## to provided relay servers.
## [br][br]
## Sets [member MultiplayerAPI.multiplayer_peer] to a new [NodeTunnelPeer].
func init(relay_addr: String, relay_port: int, app_token: String) -> void:
	var addr = relay_addr + ":" + str(relay_port)
	peer.connect_to_relay(addr, app_token)
	multiplayer.multiplayer_peer = peer
	print("[NodeTunnel] Connecting...")
	
	await peer.authenticated
	authenticated = true
	
	print("[NodeTunnel] Connected!")

## Attempts to host a new room. Returns the room ID, which can be used to join this room.
## [br][br]
## [code]is_public[/code] will decide whether this room appears in room lists.
## [code]room_metadata[/code] is for data to be displayed alongside the room list.
func host_room(is_public: bool, room_metadata: String = "") -> String:
	if !authenticated:
		await peer.authenticated
	
	peer.host_room(is_public, room_metadata)
	
	print("[NodeTunnel] Hosting room...")
	
	await peer.room_connected
	in_room = true
	
	print("[NodeTunnel] Successfully hosted room!")
	
	return peer.room_id

## Attempts to join a room using the given [code]room_id[/code].
## [br][br]
## [code]join_metadata[/code] is for data to be sent to the room host to validate
## the connnection.
func join_room(room_id: String, join_metadata: String = "") -> void:
	if !authenticated:
		await peer.authenticated
	
	peer.join_room(room_id, join_metadata)
	
	print("[NodeTunnel] Joining room...")
	
	await peer.room_connected
	in_room = true
	
	print("[NodeTunnel] Connected to room!")

## Fetches a list of available rooms to join.
## The room must be public to connect to it.
func get_rooms() -> Array[NodeTunnelRoom]:
	if !authenticated:
		await peer.authenticated
	
	peer.get_rooms()
	
	print("[NodeTunnel] Fetching room list...")
	
	var rooms = await peer.rooms_received
	var out: Array[NodeTunnelRoom] = []
	
	for room in rooms:
		var nt_room = NodeTunnelRoom.new(room.id, room.metadata)
		out.append(nt_room)
	
	print("[NodeTunnel] Fetched " + str(out.size()) + " rooms.")
	
	return out

## Updates the room this peer is in with new metadata.
## Only runs if this peer is the host of the room.
func update_room(new_metadata: String) -> void:
	if !in_room:
		push_error("[NodeTunnel] Attempted to update room before connecting to one.")
		return
	
	if !multiplayer.is_server():
		push_error("[NodeTunnel] Attempted to update a room that this peer does not own.")
		return
	
	peer.update_room(new_metadata)

## Sets the join validation to a new callable.
## Return true in this callable to allow the join.
## Return false to prevent the connecting peer from joining the room.
## Only takes affect on the hosting client.
func set_join_validation(callable: Callable) -> void:
	peer.join_validation = callable

func _handle_error(msg: String) -> void:
	push_error("[NodeTunnel] Error Encountered: ", msg)

func _ready() -> void:
	peer.error.connect(_handle_error)
