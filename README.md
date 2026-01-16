<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/d5091de1-874a-4ea6-afec-cf359401b1f0" />
    <source media="(prefers-color-scheme: light)" srcset="https://github.com/user-attachments/assets/943bdb6f-4837-4054-8668-ad4023d2a567" />
    <img
      src="https://github.com/user-attachments/assets/943bdb6f-4837-4054-8668-ad4023d2a567"
      alt="Logo"
      width="500"
      height="250"
    />
  </picture>
</p>

<h2 align="center">
  A simple Godot addon that streamlines multiplayer
</h2>

### Features
- Peer-to-peer multiplayer without dedicated servers or port-forwarding
- Works with Godot's high-level multiplayer API
- Uses relay servers to route game data between peers
- Perfect for indie games that don't want to manage dedicated servers

### Prerequisites
- NodeTunnel is still in beta, you may encounter some issues.
- Relay servers are a bulletproof way to connect behind NAT, but latency will always be higher than direct connection, and likely higher than a dedicated server.
- NodeTunnel is built for session-based games (games like Lethal Company, Phasmophobia, etc.)

*Note: The NodeTunnel server & client is very fast and adds <1ms of overhead, nearly identical to `ENetMultiplayerPeer`. Higher latency is due to the extra trip that a packet has to make, not a flaw in NodeTunnel itself.*

### Installation
1. Download the latest release of NodeTunnel under [releases](https://github.com/NodeTunnel/godot-plugin/releases/latest)
2. Download `nodetunnel.zip` if on Windows or Mac, **Linux users should download `nodetunnel.tar.gz`**
3. Decompress the downloaded file and drag n' drop the `nodetunnel` folder into `res://addons` in Godot
4. Make sure the NodeTunnel plugin is enabled in project settings

*In the future, I hope to get NodeTunnel on the Godot Asset Library!*

## Using NodeTunnel
NodeTunnel's API is very easy to use. Because NodeTunnel integrates with Godot's high-level multiplayer API, converting your `ENetMultiplayerPeer` setup is super simple. All of your game logic
will work identically with NodeTunnel, and the only thing that needs changed is making connections. Here's a basic run-down of how it works.

### Connecting to Relay Server
Before hosting or joining a room, you must first connect and authenticate with a relay server. You can do this in a `_ready` function in a script where you handle multiplayer setup. Note that you must first get your application token from [nodetunnel.io](https://nodetunnel.io).

Here's an example:
```python
# multiplayer_example.gd

var peer: NodeTunnelPeer

func _ready() -> void:
	peer = NodeTunnelPeer.new()
	peer.connect_to_relay("us_east.nodetunnel.io:8080", "application_token")
	multiplayer.multiplayer_peer = peer
	
	print("Authenticating")
	await peer.authenticated
	print("Authenticated!")
```
*Note: While you should be careful to not leak your application token, it's not the end of the world. It exists as a "filter" for rooms.*

### Hosting a Room
After authenticating with the relay server, you may then host a room. Doing so is easy:
```python
# multiplayer_example.gd

var peer: NodeTunnelPeer

func host_room() -> void:
	peer.host_room(true)
	
	print("Hosting room...")
	await peer.room_connected
	print("Connected to room: ", peer.room_id)
```
After creating a room, the `room_connected` signal will eventually emit after the relay server has processed the request. `room_id` will be filled as well, which can be shared to allow other players to join.

### Joining a Room
After authenticating with the relay server, you may then join a room after receiving a room ID.
```python
# multiplayer_example.gd

var peer: NodeTunnelPeer
# Get this from a LineEdit or something
var room_id: String

func join_room() -> void:
	peer.join_room(room_id)

  print("Joining room...")
  var connected_room_id = await peer.room_connected
  print("Connected to room: ", connected_room_id)
```
Notice that `NodeTunnelPeer.room_connected` runs on both hosting and joining clients.

### Handling Errors
Whenever the relay server encounters an error with a function the client called, it will emit the `error` signal. **It is highly recommended that you implement some sort of error handling.**
Here's a basic example that prints out any errors:
```python
# multiplayer_example.gd

var peer: NodeTunnelPeer

func _ready() -> void:
	peer = NodeTunnelPeer.new()
	
	peer.error.connect(
		func(error_msg):
			push_error("NodeTunnel Error: ", error_msg)
	)
	
	peer.connect_to_relay("us_east.nodetunnel.io:8080", "application_token")
  ...
```
Notice that the error signal is connected before calling any other functions. `connect_to_relay` can result in an error.

### What Next?
After joining or hosting a room, everything remains the same as `ENetMultiplayerPeer`. Use `multiplayer.peer_connected` signals, `MultiplayerSynchronizers`, Spawners, etc.!
