use lightyear_examples_common::settings::{
    ClientSettings, ClientTransports, Conditioner, ServerSettings, ServerTransports, Settings,
    SharedSettings, WebTransportCertificateSettings,
};
use std::net::Ipv4Addr;
use std::string::ToString;
use lightyear::prelude::CompressionConfig;

pub(crate) fn get_settings() -> Settings {
        Settings {
            server: ServerSettings {
                headless: false,
                inspector: true,
                conditioner: Some(Conditioner {
                    latency_ms: 200,
                    jitter_ms: 20,
                    packet_loss: 0.05,
                }),
                transport: vec![
                    ServerTransports::Udp { local_port: 5001 },
                ],
            },
            client: ClientSettings {
                inspector: true,
                client_id: 0,
                client_port: 0, // 0 means that the OS will assign a random port
                server_addr: Ipv4Addr::LOCALHOST,
                server_port: 5001, // change the port depending on the transport used
                transport: ClientTransports::Udp,
                conditioner: None,
            },
            shared: SharedSettings {
                protocol_id: 0,
                private_key: [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
                compression: CompressionConfig::None,
            },
        }
}
