use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::ServerConfig;
use tracing::info;

pub fn key_and_cert(key_path: Option<PathBuf>, cert_path: Option<PathBuf>) -> Result<ServerConfig> {
    let (key, certs) = if let (Some(k), Some(c)) = (key_path, cert_path) {
        load_from_disk(k, c)
    } else {
        info!("generating self-signed certificate");
        gen_self_signed()
    }?;

    Ok(ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?)
}

fn load_from_disk(key_path: PathBuf, cert_path: PathBuf) -> Result<(PrivateKeyDer<'static>, Vec<CertificateDer<'static>>)> {
    let key = fs::read(key_path).context("failed to read private key")?;
    let key = rustls_pemfile::private_key(&mut &*key)
        .context("could not read private key in PKCS #1 format")?
        .ok_or_else(|| anyhow::Error::msg("no private keys found"))?;

    let cert_chain = fs::read(cert_path).context("failed to read certificate chain")?;

    let cert_chain = rustls_pemfile::certs(&mut &*cert_chain)
        .collect::<Result<_, _>>()
        .context("invalid PEM-Encoded certificate")?;

    Ok((key, cert_chain))
}

fn gen_self_signed() -> Result<(PrivateKeyDer<'static>, Vec<CertificateDer<'static>>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

    Ok((key.into(), vec![cert.cert.into()]))
}
