use libp2p::{identity::Keypair, noise, tcp, yamux, Multiaddr, PeerId, Swarm};
use log::{error, info};
use std::time::Duration;

pub struct WakuLightNodeConfig {
    pub peers: Vec<Multiaddr>,
    pub keypair: Keypair,
}

impl WakuLightNodeConfig {
    pub fn new(keypair: Option<Keypair>, peers: Vec<Multiaddr>) -> Self {
        Self {
            keypair: keypair.unwrap_or(Keypair::generate_ed25519()),
            peers,
        }
    }
}

pub struct WakuLightNode {
    pub swarm: Swarm<libp2p::swarm::dummy::Behaviour>,
}

impl WakuLightNode {
    pub fn new_with_config(config: WakuLightNodeConfig) -> Result<Self, Error> {
        let local_peer_id = PeerId::from(config.keypair.public());
        info!("Libp2p local peer id: {:?}", local_peer_id);

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(config.keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default().nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )
            .unwrap()
            .with_dns()
            .unwrap()
            .with_behaviour(|_key| libp2p::swarm::dummy::Behaviour {})
            .unwrap()
            .with_swarm_config(|config| {
                config
                    .with_notify_handler_buffer_size(
                        std::num::NonZeroUsize::new(20).expect("Not zero"),
                    )
                    .with_per_connection_event_buffer_size(64)
                    .with_idle_connection_timeout(Duration::from_secs(60 * 10))
            })
            .build();

        for peer in config.peers {
            swarm.dial(peer)?;
        }
        Ok(Self { swarm })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Multiaddr: {0}")]
    Multiaddr(#[from] libp2p::multiaddr::Error),
    #[error("Transport: {0}")]
    Transport(#[from] libp2p::TransportError<std::io::Error>),
    #[error("Dial: {0}")]
    Dial(#[from] libp2p::swarm::DialError),
}
