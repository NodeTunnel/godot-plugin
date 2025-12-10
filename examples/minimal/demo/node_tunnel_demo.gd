extends Node2D

signal connected_to_room

## Currently, the provided relay servers allow any app id to connect,
## so pick whatever you want. Just make sure it's unique enough so someone else doesn't
## accidentally use your app ID and rooms from another game.
const APP_ID = "nodetunnel_demo"
const RELAY_ADDRESS = "45.33.64.148:8080"

@onready var peer := NodeTunnelPeer.new()

func _ready() -> void:
	# This signal fires once the client has successfully been added to a room, whether they
	# hosted the room or joined it.
	peer.room_connected.connect(
		func(room_id):
			push_warning("Connected to room!")
			%RoomId.text = room_id
			%AppId.text = APP_ID
			DisplayServer.clipboard_set(room_id)
			
			$UI/HUD.show()
			
			connected_to_room.emit()
	)
	
	# This signal fires when the client is forcibly disconnected from the relay server,
	# mostly caused by the room host kicking the clients.
	peer.forced_disconnect.connect(
		func():
			print("Client forced to disconnect!")
			# We should probably reload the scene or quit to the main menu at this
			# point, because NodeTunnel has fully disconnected from the relay server.
			# Calling peer.host_room or peer.join_room will not work without first
			# re-authenticating with the relay server.
			get_tree().reload_current_scene()
	)
	
	# Handle errors.
	peer.error.connect(
		func(msg):
			print("Relay error: ", msg)
	)
	
	# Normal high-level multiplayer API from here on out!
	multiplayer.peer_connected.connect(
		func(pid):
			print("Peer ", pid, " has joined!")
			%ConnectedPeers.text = "Connected Peers: " + str(multiplayer.get_peers().size())
	)
	
	$UI/ConnectionControls.hide()
	$UI/HUD.hide()
	$UI/Connecting.show()
	
	# connect_to_relay will need to be called regardless of whether we are hosting or joining, 
	# so putting it in _ready is fine.
	peer.connect_to_relay(RELAY_ADDRESS, APP_ID)
	# Make sure to set the scene's multiplayer peer, or else NodeTunnel will never connect.
	multiplayer.multiplayer_peer = peer
	
	# Wait until the client has authenticated with the relay server before allowing users to
	# host or join rooms.
	await peer.authenticated
	
	# Handle the response for when a room list is sent to the client.
	# This is triggered from peer.get_rooms()
	peer.rooms_received.connect(
		func(rooms):
			for room in rooms:
				print("- ", room)
	)
	
	$UI/Connecting.hide()
	# In this case, hiding/showing UI/ConnectionControls prevents users from hosting/joining
	# rooms early. This is one of many ways to accomplish this.
	$UI/ConnectionControls.show()

func _on_host_pressed() -> void:
	# Sends a request to the relay server to start hosting a room.
	# Can result in an error, which should be handled by the relay_error signal.
	# Use -1 for unlimited players.
	peer.host_room(true, "My Game", -1)
	$UI/ConnectionControls.hide()

func _on_join_pressed() -> void:
	# Sends a request to the relay server to join a room.
	# Can result in an error which should be handled by the relay_error signal.
	peer.join_room(%HostID.text)
	$UI/ConnectionControls.hide()

func _on_disconnect_pressed() -> void:
	# Close the connection.
	peer.close()
	
	# Reload the scene as in our handler for the forced_disconnect signal.
	get_tree().reload_current_scene()

func _on_room_list_timer_timeout() -> void:
	peer.get_rooms()
