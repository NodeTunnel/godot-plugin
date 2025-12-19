mod node_tunnel_peer;
mod relay_client;
pub mod protocol;
mod transport;

use godot::prelude::*;

struct NodeTunnel;

#[gdextension]
unsafe impl ExtensionLibrary for NodeTunnel {}
