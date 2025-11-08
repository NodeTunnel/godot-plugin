use renet::{ConnectionConfig, DefaultChannel, RenetClient};
use renet_netcode::{ClientAuthentication, NetcodeClientTransport};
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant, SystemTime};

pub struct ReceivedPacket {
    pub data: Vec<u8>,
    pub channel: DefaultChannel,
}

pub struct RenetPacketPeer {
    client: Option<RenetClient>,
    transport: Option<NetcodeClientTransport>,
    last_updated: Instant,
}

impl RenetPacketPeer {
    pub fn new() -> Self {
        Self {
            client: None,
            transport: None,
            last_updated: Instant::now(),
        }
    }

    pub fn connect(&mut self, server_addr: SocketAddr) -> Result<(), Box<dyn Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;

        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let client_id = current_time.as_millis() as u64;

        let authentication = ClientAuthentication::Unsecure {
            server_addr,
            client_id,
            user_data: None,
            protocol_id: 0,
        };

        let client = RenetClient::new(ConnectionConfig::default());
        let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

        self.client = Some(client);
        self.transport = Some(transport);
        self.last_updated = Instant::now();

        Ok(())
    }

    pub fn update(&mut self) -> Result<Vec<ReceivedPacket>, Box<dyn Error>> {
        let delta_time = Duration::from_millis(16);
        let mut received_packets = Vec::new();

        if let (Some(client), Some(transport)) = (self.client.as_mut(), self.transport.as_mut()) {
            client.update(delta_time);
            transport.update(delta_time, client)?;

            if client.is_connected() {
                while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
                    received_packets.push(ReceivedPacket {
                        data: message.to_vec(),
                        channel: DefaultChannel::ReliableOrdered,
                    });
                }
                while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
                    received_packets.push(ReceivedPacket {
                        data: message.to_vec(),
                        channel: DefaultChannel::Unreliable,
                    });
                }
            }
            
            transport.send_packets(client)?;
        }

        Ok(received_packets)
    }

    pub fn send(&mut self, data: &[u8], channel: DefaultChannel) -> Result<(), Box<dyn Error>> {
        if let Some(client) = self.client.as_mut() {
            if client.is_connected() {
                client.send_message(channel, data.to_vec());
                Ok(())
            } else {
                Err("Cannot send message: not connected to server".into())
            }
        } else {
            Err("Cannot send message: client not initialized".into())
        }
    }

    pub fn is_connected(&self) -> bool {
        self.client.as_ref().map_or(false, |c| c.is_connected())
    }

    pub fn disconnect(&mut self) {
        self.client = None;
        self.transport = None;
    }
}