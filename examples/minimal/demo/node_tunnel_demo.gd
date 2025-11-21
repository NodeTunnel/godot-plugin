extends Node2D

signal connected_to_room

var relay_addr = "127.0.0.1:8080"

## get this either from nodetunnel's website
## or if you're self hosting, pick any app id and put it in the
## server's config.toml.
const APP_ID = "c1xz9woxf6bi42m"

@onready var peer := NodeTunnelPeer.new()

func _ready() -> void:
	## OPTIONAL: in the event that multiple relays are available, nodetunnel provides
	## a service to pick the closest (least ms) one. Note that connecting peers must also
	## be on the same relay server as the host.
	#var discover = RelayDiscovery.new("http://127.0.0.1:8090")
	#add_child(discover)
	#print(await discover.find_best_relay(APP_ID))
	
	peer.connect_to_relay(relay_addr, APP_ID)
	
	## this signal fires once the client has successfully been added to a room, whether they
	## hosted the room or joined it.
	peer.room_connected.connect(
		func(room_id):
			push_warning("Connected to room!")
			%RoomId.text = room_id
			%AppId.text = APP_ID
			DisplayServer.clipboard_set(room_id)
			
			$UI/HUD.show()
			
			connected_to_room.emit()
	)
	
	## this signal fires when the client is forcibly disconnected from the relay server
	## mostly caused by the room host kicking the clients
	peer.forced_disconnect.connect(
		func():
			print("Client forced to disconnect!")
			## should probably reload the scene or quit to the main menu
			## at this point, NodeTunnel has fully disconnected from the relay server
			## calling peer.host_room or peer.join_room will not work without first
			## re-authenticating with the relay server
			get_tree().reload_current_scene()
	)
	
	## handle errros
	peer.error.connect(
		func(msg):
			print("Relay error: ", msg)
	)
	
	## normal high-level multiplayer API from here on out!
	multiplayer.peer_connected.connect(
		func(pid):
			print("Peer ", pid, " has joined!")
			%ConnectedPeers.text = "Connected Peers: " + str(multiplayer.get_peers().size())
	)
	
	$UI/ConnectionControls.hide()
	$UI/HUD.hide()
	$UI/Connecting.show()
	
	## connect_to_relay will need to be called regardless of whether we are hosting or joining, 
	## so putting it in _ready is fine.
	peer.connect_to_relay(relay_addr, APP_ID)
	## make sure to set the scene's multiplayer peer, or else NodeTunnel will never connect.
	multiplayer.multiplayer_peer = peer
	
	## wait until the client has authenticated with the relay server before allowing users to
	## host or join rooms.
	await peer.authenticated
	
	$UI/Connecting.hide()
	## in this case, hiding/showing UI/ConnectionControls prevents users from hosting/joining
	## rooms early. this is one of many ways to accomplish this.
	$UI/ConnectionControls.show()

func _on_host_pressed() -> void:
	## sends a request to the relay server to start hosting a room
	## can result in an error, which should be handled by the relay_error signal
	peer.host_room()
	$UI/ConnectionControls.hide()

func _on_join_pressed() -> void:
	## sends a request to the relay server to join a room
	## can result in an error, which should be handled by the relay_error signal
	peer.join_room(%HostID.text)
	$UI/ConnectionControls.hide()

func _on_disconnect_pressed() -> void:
	## close the connection
	peer.close()

func _exit_tree() -> void:
	## close the connection
	peer.close()

func _process(delta: float) -> void:
	if Input.is_action_just_pressed("ui_accept"):
		hello_world.rpc()

@rpc("any_peer", "reliable")
func hello_world():
	print("Hello world!")
