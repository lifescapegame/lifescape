use anyhow::Result;
use bevy::prelude::*;
use bevy_renet::renet::{ClientAuthentication, RenetClient, RenetConnectionConfig};
use clap::Args;
use std::net::{SocketAddr, UdpSocket};
use std::time::SystemTime;

use super::{Channel, DEFAULT_PORT, PROTOCOL_ID};

pub(super) struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConnectionSettings::default());
    }
}

#[derive(Args, Clone, Debug, PartialEq)]
pub(crate) struct ConnectionSettings {
    /// Server IP address.
    #[clap(short, long, default_value_t = ConnectionSettings::default().ip)]
    pub(crate) ip: String,

    /// Server port.
    #[clap(short, long, default_value_t = ConnectionSettings::default().port)]
    pub(crate) port: u16,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: DEFAULT_PORT,
        }
    }
}

impl ConnectionSettings {
    pub(crate) fn create_client(&self) -> Result<RenetClient> {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let client_id = current_time.as_millis() as u64;
        let server_addr = SocketAddr::new(self.ip.parse()?, 0);
        let socket = UdpSocket::bind(server_addr)?;
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            server_addr,
            user_data: None,
        };
        let connection_config = RenetConnectionConfig {
            send_channels_config: Channel::config(),
            receive_channels_config: Channel::config(),
            ..Default::default()
        };

        RenetClient::new(
            current_time,
            socket,
            client_id,
            connection_config,
            authentication,
        )
        .map_err(From::from)
    }
}
