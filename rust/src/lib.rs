mod node_tunnel_peer;
pub mod renet_packet_peer;
mod packet_type;
mod relay_client;
mod version;

use godot::prelude::*;

struct NodeTunnel;

#[gdextension]
unsafe impl ExtensionLibrary for NodeTunnel {}
