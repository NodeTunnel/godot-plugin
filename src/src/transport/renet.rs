use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};
use renet::{ConnectionConfig, DefaultChannel, RenetClient};
use renet_netcode::{ClientAuthentication, NetcodeClientTransport};
use crate::protocol::version::PROTOCOL_VERSION;
use crate::transport::error::TransportError;
use crate::transport::types::{Channel, Packet};

pub struct RenetTransport {
    client: RenetClient,
    transport: NetcodeClientTransport,
}

impl RenetTransport {
    pub fn new(server_addr: SocketAddr) -> Result<Self, TransportError> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(TransportError::BindFailed)?;
        socket.set_nonblocking(true)
            .map_err(TransportError::SetNonBlockingFailed)?;

        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let client_id = current_time.as_millis() as u64;

        let authentication = ClientAuthentication::Unsecure {
            server_addr,
            client_id,
            user_data: None,
            protocol_id: PROTOCOL_VERSION,
        };

        let client = RenetClient::new(ConnectionConfig::default());
        let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

        Ok(Self { client, transport })
    }

    pub fn update(&mut self, delta_time: Duration) -> Result<(), TransportError> {
        self.client.update(delta_time);
        self.transport.update(delta_time, &mut self.client)?;
        self.transport.send_packets(&mut self.client)?;

        Ok(())
    }

    pub fn recv_packets(&mut self) -> Vec<Packet> {
        let mut recv_packets = Vec::new();

        let channels = [Channel::Reliable, Channel::Unreliable];

        for channel in channels.iter() {
            while let Some(message) = self.client.receive_message(DefaultChannel::from(*channel)) {
                let packet = Packet {
                    data: Vec::from(message),
                    channel: *channel,
                };

                recv_packets.push(packet);
            }
        }

        recv_packets
    }

    pub fn send_to_server(&mut self, data: Vec<u8>, channel: Channel) -> Result<(), TransportError> {
        self.client.send_message(DefaultChannel::from(channel), data);
        self.transport.send_packets(&mut self.client)?;

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    pub fn disconnect_from_server(&mut self) {
        self.client.disconnect();
        self.transport.disconnect();
    }
}
