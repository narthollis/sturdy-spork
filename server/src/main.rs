mod tls;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use quinn::crypto::rustls::QuicServerConfig;
use thiserror::Error;
use tracing::{error, info_span};
use crate::tls::{key_and_cert};

#[derive(Parser, Debug)]
struct Args {
    /// Address to listen on
    #[clap(long = "listen", default_value = "[::1]:4433")]
    listen: SocketAddr,
    /// TLS private key in PEM format
    #[clap(short = 'k', long = "key", requires = "cert")]
    key: Option<PathBuf>,
    /// TLS certificate in PEM format
    #[clap(short = 'c', long = "cert", requires = "key")]
    cert: Option<PathBuf>,
}

#[derive(Error, Debug)]
pub enum MyError {
    #[error("was not passed cert and key")]
    NoCertAndKey()
}

fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    ).expect("failed to setup up tracing subscriber");

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install CryptoProvider");

    let args = Args::parse();

    run(args)
}

const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-29"];

#[tokio::main]
async fn run(args: Args) -> anyhow::Result<()> {
    let mut server_crypto = key_and_cert(args.key, args.cert)?;

    server_crypto.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();

    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = quinn::Endpoint::server(server_config, args.listen)?;
    eprintln!("listening on {}", endpoint.local_addr()?);

    while let Some(conn) = endpoint.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_connection(conn).await {
                error!(err = e.as_ref() as &dyn std::error::Error, "connection failed")
            }
        });
    }

    Ok(())
}

async fn handle_connection(conn: quinn::Incoming) -> anyhow::Result<()> {
    let connection = conn.await?;
    let _span = info_span!(
        "connection",
        remote = %connection.remote_address(),
        protocol = %connection
            .handshake_data()
            .unwrap()
            .downcast::<quinn::crypto::rustls::HandshakeData>().unwrap()
            .protocol
            .map_or_else(|| "<none>".into(), |x| String::from_utf8_lossy(&x).into_owned())
    )
        .entered();

    Ok(())
}
