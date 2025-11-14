mod node_tunnel_peer;
pub mod renet_packet_peer;
mod relay_client;
mod protocol;

use godot::prelude::*;

struct NodeTunnel;

#[gdextension]
unsafe impl ExtensionLibrary for NodeTunnel {}
