//! Verified SQL transport adapters. No custom certificate verification logic.
use super::*;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Additional transport settings for the Postgres and MySQL wire connectors.
#[derive(Clone, Debug, Default)]
pub struct ConnectionOptions {
    /// Require an encrypted connection; false preserves the target's own policy.
    pub tls: bool,
    /// Extra PEM CA certificates. Supplying a CA also requires TLS.
    pub ca_file: Option<PathBuf>,
}

impl ConnectionOptions {
    pub(super) fn enabled(&self) -> bool {
        self.tls || self.ca_file.is_some()
    }
    pub(super) fn check_kind(&self, kind: DriverKind) -> Result<(), SoplangError> {
        if self.enabled()
            && !matches!(
                kind,
                DriverKind::Postgres
                    | DriverKind::Cockroach
                    | DriverKind::Mysql
                    | DriverKind::Mariadb
            )
        {
            return Err(data_error("Xulashooyinka tls iyo shahaado_ca waxaa isticmaala postgres, cockroach, mysql, iyo mariadb. MongoDB iyo SQL Server waxay xulashada TLS ka akhriyaan cinwaankooda."));
        }
        Ok(())
    }
}

fn certificates(
    path: &Path,
) -> Result<Vec<rustls::pki_types::CertificateDer<'static>>, SoplangError> {
    const LIMIT: u64 = 1024 * 1024;
    let info = std::fs::metadata(path).map_err(driver_error)?;
    if !info.is_file() || info.len() > LIMIT {
        return Err(data_error(
            "Shahaadada CA waa inay noqotaa fayl aan ka weynayn 1 MiB.",
        ));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(driver_error)?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(driver_error)?;
    if bytes.len() as u64 > LIMIT {
        return Err(data_error("Faylka shahaadada CA wuu ka weyn yahay 1 MiB."));
    }
    let roots = rustls_pemfile::certs(&mut bytes.as_slice())
        .collect::<Result<Vec<_>, _>>()
        .map_err(driver_error)?;
    if roots.is_empty() {
        return Err(data_error("Faylka CA kuma jirto shahaado PEM sax ah."));
    }
    // Validate DER before passing the same roots to the platform verifier.
    let mut store = rustls::RootCertStore::empty();
    for root in &roots {
        store.add(root.clone()).map_err(driver_error)?;
    }
    Ok(roots)
}

pub(super) fn postgres(
    target: &str,
    options: &ConnectionOptions,
) -> Result<postgres::Client, SoplangError> {
    let mut config: postgres::Config = target.parse().map_err(driver_error)?;
    if options.enabled() {
        if config.get_ssl_mode() == postgres::config::SslMode::Disable {
            return Err(data_error("Xulashada TLS ma waafaqsana sslmode=disable."));
        }
        config.ssl_mode(postgres::config::SslMode::Require);
    }
    if config.get_ssl_mode() == postgres::config::SslMode::Disable {
        return config.connect(postgres::NoTls).map_err(driver_error);
    }
    let roots = options
        .ca_file
        .as_deref()
        .map(certificates)
        .transpose()?
        .unwrap_or_default();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier =
        rustls_platform_verifier::Verifier::new_with_extra_roots(roots, provider.clone())
            .map_err(driver_error)?;
    let config_tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(driver_error)?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();
    config
        .connect(tokio_postgres_rustls::MakeRustlsConnect::new(config_tls))
        .map_err(driver_error)
}

pub(super) fn mysql(
    target: &str,
    options: &ConnectionOptions,
) -> Result<mysql::Conn, SoplangError> {
    let parsed = mysql::Opts::from_url(target).map_err(driver_error)?;
    let mut ssl = parsed.get_ssl_opts().cloned();
    if options.enabled() && ssl.is_none() {
        ssl = Some(mysql::SslOpts::default());
    }
    if let Some(settings) = &ssl {
        if settings.accept_invalid_certs() || settings.skip_domain_validation() {
            return Err(data_error(
                "Xaqiijinta shahaadada TLS iyo magaca martigeliyaha lama damin karo.",
            ));
        }
    }
    let tls = ssl.is_some();
    let mut builder = mysql::OptsBuilder::from_opts(parsed);
    if tls {
        // Local-socket optimization would otherwise bypass the TLS handshake.
        if mysql::Opts::from(builder.clone()).get_socket().is_some() {
            return Err(data_error(
                "TLS laguma isticmaali karo xulashada socket; isticmaal cinwaan TCP ah.",
            ));
        }
        builder = builder.prefer_socket(false);
        let _ = rustls::crypto::ring::default_provider().install_default();
        if let Some(path) = &options.ca_file {
            certificates(path)?;
            ssl = ssl.map(|settings| settings.with_root_cert_path(Some(path.clone())));
        }
    }
    mysql::Conn::new(builder.ssl_opts(ssl)).map_err(driver_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_transport_options_fail_before_connecting() {
        let required = ConnectionOptions {
            tls: true,
            ..Default::default()
        };
        assert!(postgres("host=127.0.0.1 port=1 sslmode=disable", &required).is_err());
        assert!(mysql(
            "mysql://127.0.0.1:1/?danger_accept_invalid_certs=true",
            &Default::default()
        )
        .is_err());
        assert!(mysql(
            "mysql://127.0.0.1:1/?danger_skip_domain_validation=true",
            &Default::default()
        )
        .is_err());
        assert!(mysql("mysql://127.0.0.1:1/?socket=/tmp/not-a-socket", &required).is_err());
        assert!(required.check_kind(DriverKind::Sqlite).is_err());
        assert!(certificates(Path::new("/dev/null")).is_err());
    }
}
