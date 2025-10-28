/// Transport layer wrapper around renet
///
/// Provides UDP-based networking with reliability built on top.
/// This is the lowest level of the networking stack.

use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use renet_netcode::{
    NetcodeServerTransport, NetcodeClientTransport,
    ServerAuthentication, ClientAuthentication, ServerConfig,
};
use renet::RenetServer;
use anyhow::Result;

/// Transport configuration shared between client and server
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Protocol identifier (must match between client and server)
    pub protocol_id: u64,
    /// Maximum transmission unit size (bytes)
    pub max_packet_size: usize,
    /// Send rate (packets per second)
    pub send_rate: u32,
    /// Receive rate (packets per second)
    pub receive_rate: u32,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Disconnect timeout
    pub timeout: Duration,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            protocol_id: 0x4553_4F4E_414E_4345, // "RESONANCE" in hex
            max_packet_size: 8192,
            send_rate: 60,
            receive_rate: 60,
            heartbeat_interval: Duration::from_secs(1),
            timeout: Duration::from_secs(10),
        }
    }
}

/// Server-side network transport
pub struct ServerTransport {
    transport: NetcodeServerTransport,
    #[allow(dead_code)]
    config: TransportConfig,
    bind_addr: SocketAddr,
}

impl ServerTransport {
    pub fn new(
        bind_addr: SocketAddr,
        config: TransportConfig,
        max_clients: usize,
    ) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?;

        let server_config = ServerConfig {
            current_time,
            max_clients,
            protocol_id: config.protocol_id,
            public_addresses: vec![bind_addr],
            authentication: ServerAuthentication::Unsecure,
        };

        let transport = NetcodeServerTransport::new(server_config, socket)?;

        Ok(Self { transport, config, bind_addr })
    }

    pub fn update(&mut self, delta: Duration, server: &mut RenetServer) -> Result<()> {
        self.transport.update(delta, server)?;
        Ok(())
    }

    pub fn send_packets(&mut self, server: &mut RenetServer) -> Result<()> {
        let _ = self.transport.send_packets(server);
        Ok(())
    }

    pub fn recv_packets(&mut self, _server: &mut RenetServer) -> Result<()> {
        // recv_packets doesn't exist, the transport updates in update()
        Ok(())
    }

    pub fn addr(&self) -> SocketAddr {
        self.bind_addr
    }
}

/// Client-side network transport
pub struct ClientTransport {
    transport: NetcodeClientTransport,
    #[allow(dead_code)]
    config: TransportConfig,
}

impl ClientTransport {
    pub fn new(
        server_addr: SocketAddr,
        client_id: u64,
        config: TransportConfig,
    ) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?;

        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: config.protocol_id,
            server_addr,
            user_data: None,
        };

        let transport = NetcodeClientTransport::new(
            current_time,
            authentication,
            socket,
        )?;

        Ok(Self { transport, config })
    }

    pub fn update(&mut self, delta: Duration, client: &mut renet::RenetClient) -> Result<()> {
        self.transport.update(delta, client)?;
        Ok(())
    }

    pub fn send_packets(&mut self, client: &mut renet::RenetClient) -> Result<()> {
        let _ = self.transport.send_packets(client);
        Ok(())
    }

    pub fn recv_packets(&mut self, _client: &mut renet::RenetClient) -> Result<()> {
        // recv_packets doesn't exist, the transport updates in update()
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        // Connection status is managed by the underlying client
        true // Simplified for now
    }

    pub fn disconnect(&mut self) {
        self.transport.disconnect();
    }
}
