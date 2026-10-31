// SPDX-License-Identifier: Apache-2.0

//! TLS configuration for the reqwest 0.11 client required by `tendermint-rpc`.
//!
//! reqwest 0.11 is tied to rustls 0.21 / rustls-webpki 0.101, which no longer receive
//! certificate distrust updates. The verifier here keeps reqwest on rustls 0.21 for the
//! handshake but does all certificate parsing and verification with rustls 0.23 /
//! rustls-webpki 0.103, so the Tendermint RPC client follows the same distrust list as the
//! workspace's rustls 0.23 clients.

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use rustls::{
    client::{danger::ServerCertVerifier as _, WebPkiServerVerifier},
    crypto::WebPkiSupportedAlgorithms,
    pki_types::{CertificateDer, ServerName, UnixTime},
    RootCertStore, SignatureScheme,
};
use rustls_0_21::client::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};

/// Signature schemes allowed in TLS 1.3 handshakes (RFC 8446, section 4.2.3).
const TLS13_SCHEMES: &[SignatureScheme] = &[
    SignatureScheme::ECDSA_NISTP256_SHA256,
    SignatureScheme::ECDSA_NISTP384_SHA384,
    SignatureScheme::ECDSA_NISTP521_SHA512,
    SignatureScheme::RSA_PSS_SHA256,
    SignatureScheme::RSA_PSS_SHA384,
    SignatureScheme::RSA_PSS_SHA512,
    SignatureScheme::ED25519,
    SignatureScheme::ED448,
];

/// Builds a rustls 0.21 client config for `reqwest_0_11::ClientBuilder::use_preconfigured_tls`
/// that validates server certificates with rustls-webpki 0.103.
///
/// Trust anchors come from the OS store. The bundled Mozilla roots (`webpki-roots`) are used
/// only when the OS store is absent, i.e. it yields no certificates and reports no errors.
///
/// # Errors
/// Returns an error if the OS store is configured but unusable: it is unreadable (for example
/// a bad `SSL_CERT_FILE`) or none of its certificates parse. Also returns an error if the
/// certificate verifier cannot be built.
pub fn client_config() -> anyhow::Result<rustls_0_21::ClientConfig> {
    client_config_with_roots(root_store()?)
}

fn client_config_with_roots(roots: RootCertStore) -> anyhow::Result<rustls_0_21::ClientConfig> {
    let mut config = rustls_0_21::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(Verifier::new(roots)?))
        .with_no_client_auth();
    // reqwest only sets ALPN on configs it builds itself; keep its default of offering HTTP/2
    // with HTTP/1.1 as the fallback.
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(config)
}

/// Loads the OS root certificates, falling back to the bundled Mozilla roots if there are none.
fn root_store() -> anyhow::Result<RootCertStore> {
    let native = rustls_native_certs::load_native_certs();
    let errors: Vec<_> = native.errors.iter().map(ToString::to_string).collect();
    let found = native.certs.len();
    let mut roots = RootCertStore::empty();
    let (valid, invalid) = roots.add_parsable_certificates(native.certs);

    if valid > 0 {
        if !errors.is_empty() {
            tracing::warn!(
                errors = errors.join("; "),
                "some OS root certificate sources could not be read"
            );
        }
        tracing::debug!(valid, invalid, "using OS root certificates");
        return Ok(roots);
    }
    if found > 0 {
        anyhow::bail!("none of the {found} OS root certificates could be parsed");
    }
    if !errors.is_empty() {
        anyhow::bail!(
            "failed to load native root certificates: {}",
            errors.join("; ")
        );
    }

    tracing::warn!("no OS root certificates found; using the bundled Mozilla roots (webpki-roots)");
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Ok(roots)
}

/// Adapts a rustls 0.23 [`WebPkiServerVerifier`] to the rustls 0.21 verifier trait.
struct Verifier {
    inner: Arc<WebPkiServerVerifier>,
    algorithms: WebPkiSupportedAlgorithms,
}

impl Verifier {
    fn new(roots: RootCertStore) -> anyhow::Result<Self> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let algorithms = provider.signature_verification_algorithms;
        let inner =
            WebPkiServerVerifier::builder_with_provider(Arc::new(roots), provider).build()?;
        Ok(Self { inner, algorithms })
    }

    /// Verifies a handshake signature with rustls-webpki 0.103 instead of the rustls 0.21
    /// default, which would re-parse the end-entity certificate with rustls-webpki 0.101.
    fn verify_signature(
        &self,
        message: &[u8],
        cert: &rustls_0_21::Certificate,
        dss: &rustls_0_21::DigitallySignedStruct,
        tls13: bool,
    ) -> Result<HandshakeSignatureValid, rustls_0_21::Error> {
        let unadvertised = || {
            rustls_0_21::Error::PeerMisbehaved(
                rustls_0_21::PeerMisbehaved::SignedHandshakeWithUnadvertisedSigScheme,
            )
        };

        let scheme = SignatureScheme::from(dss.scheme.get_u16());
        if tls13 && !TLS13_SCHEMES.contains(&scheme) {
            return Err(unadvertised());
        }
        let algs = self
            .algorithms
            .mapping
            .iter()
            .find(|(s, _)| *s == scheme)
            .map(|(_, algs)| *algs)
            .ok_or_else(unadvertised)?;
        // TLS 1.3 binds a scheme to a single algorithm; TLS 1.2 may need to try several.
        let algs = if tls13 {
            algs.get(..1).unwrap_or_default()
        } else {
            algs
        };

        let cert = CertificateDer::from(cert.0.as_slice());
        let cert = webpki::EndEntityCert::try_from(&cert).map_err(invalid_certificate)?;
        let mut error = None;
        for alg in algs {
            // Only an algorithm/key-type mismatch moves on to the next candidate; any other
            // error (e.g. a bad signature) is final, as in rustls 0.23's `verify_tls12_signature`.
            match cert.verify_signature(*alg, message, dss.signature()) {
                Ok(()) => return Ok(HandshakeSignatureValid::assertion()),
                Err(e @ webpki::Error::UnsupportedSignatureAlgorithmForPublicKeyContext(_)) => {
                    error = Some(e);
                }
                Err(e) => return Err(invalid_certificate(e)),
            }
        }
        Err(error.map_or_else(unadvertised, invalid_certificate))
    }
}

impl ServerCertVerifier for Verifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls_0_21::Certificate,
        intermediates: &[rustls_0_21::Certificate],
        server_name: &rustls_0_21::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        ocsp_response: &[u8],
        now: SystemTime,
    ) -> Result<ServerCertVerified, rustls_0_21::Error> {
        let server_name = match server_name {
            rustls_0_21::ServerName::DnsName(name) => ServerName::try_from(name.as_ref())
                .map_err(|e| rustls_0_21::Error::General(e.to_string()))?,
            rustls_0_21::ServerName::IpAddress(ip) => ServerName::IpAddress((*ip).into()),
            _ => {
                return Err(rustls_0_21::Error::General(
                    "unsupported server name type".to_string(),
                ))
            }
        };
        let now = now
            .duration_since(UNIX_EPOCH)
            .map_err(|e| rustls_0_21::Error::General(e.to_string()))?;
        let intermediates: Vec<_> = intermediates
            .iter()
            .map(|cert| CertificateDer::from(cert.0.as_slice()))
            .collect();

        self.inner
            .verify_server_cert(
                &CertificateDer::from(end_entity.0.as_slice()),
                &intermediates,
                &server_name,
                ocsp_response,
                UnixTime::since_unix_epoch(now),
            )
            .map(|_| ServerCertVerified::assertion())
            .map_err(convert_error)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls_0_21::Certificate,
        dss: &rustls_0_21::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls_0_21::Error> {
        self.verify_signature(message, cert, dss, false)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls_0_21::Certificate,
        dss: &rustls_0_21::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls_0_21::Error> {
        self.verify_signature(message, cert, dss, true)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls_0_21::SignatureScheme> {
        self.algorithms
            .supported_schemes()
            .into_iter()
            .map(|scheme| rustls_0_21::SignatureScheme::from(u16::from(scheme)))
            .collect()
    }
}

/// Converts a rustls 0.23 verification error to rustls 0.21.
///
/// Variants without data map to their rustls 0.21 equivalent, so rustls 0.21 sends the matching
/// TLS alert. Variants with data, including the `*Context` variants rustls-webpki 0.103 returns
/// for expired certificates and name mismatches, are wrapped in `CertificateError::Other` so
/// their details stay in the error message.
fn convert_error(error: rustls::Error) -> rustls_0_21::Error {
    use rustls::CertificateError as E;
    use rustls_0_21::CertificateError as E21;

    let error = match error {
        rustls::Error::InvalidCertificate(error) => error,
        error => return rustls_0_21::Error::General(error.to_string()),
    };
    let error = match error {
        E::BadEncoding => E21::BadEncoding,
        E::Expired => E21::Expired,
        E::NotValidYet => E21::NotValidYet,
        E::Revoked => E21::Revoked,
        E::UnhandledCriticalExtension => E21::UnhandledCriticalExtension,
        E::UnknownIssuer => E21::UnknownIssuer,
        E::BadSignature => E21::BadSignature,
        E::NotValidForName => E21::NotValidForName,
        E::InvalidPurpose => E21::InvalidPurpose,
        E::ApplicationVerificationFailure => E21::ApplicationVerificationFailure,
        error => E21::Other(Arc::new(rustls::Error::InvalidCertificate(error))),
    };
    rustls_0_21::Error::InvalidCertificate(error)
}

fn invalid_certificate(error: webpki::Error) -> rustls_0_21::Error {
    let error = match error {
        webpki::Error::BadDer | webpki::Error::BadDerTime => {
            rustls_0_21::CertificateError::BadEncoding
        }
        webpki::Error::InvalidSignatureForPublicKey => rustls_0_21::CertificateError::BadSignature,
        error => rustls_0_21::CertificateError::Other(Arc::new(error)),
    };
    rustls_0_21::Error::InvalidCertificate(error)
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read as _, Write as _},
        net::{SocketAddr, TcpListener, TcpStream},
        thread,
    };

    use rcgen::{
        date_time_ymd, BasicConstraints, CertificateParams, CertifiedIssuer, DnType,
        ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose, SigningKey as _,
    };
    use rustls::{
        pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer},
        version::{TLS12, TLS13},
        SupportedProtocolVersion,
    };
    use rustls_0_21::{
        internal::msgs::codec::Codec as _, CertificateError, ClientConnection,
        DigitallySignedStruct, ProtocolVersion,
    };

    use super::*;

    const MESSAGE: &[u8] = b"ibc-eureka-utils tls test message";

    /// A certificate authority whose root is trusted only by the clients a test gives it to.
    struct TestCa(CertifiedIssuer<'static, KeyPair>);

    impl TestCa {
        /// Creates a CA named `name`. CAs in one test need distinct names: a verifier only reports
        /// an unknown issuer if no trusted root has the issuer's name.
        fn new(name: &str) -> Self {
            let mut params = CertificateParams::default();
            params.distinguished_name.push(DnType::CommonName, name);
            params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
            params.key_usages = vec![KeyUsagePurpose::KeyCertSign];
            Self(CertifiedIssuer::self_signed(params, KeyPair::generate().unwrap()).unwrap())
        }

        fn roots(&self) -> RootCertStore {
            let mut roots = RootCertStore::empty();
            roots.add(self.0.der().clone()).unwrap();
            roots
        }

        /// Issues a server certificate for `names` (DNS names or IP addresses) to `key`.
        fn issue(
            &self,
            names: &[&str],
            key: &KeyPair,
            customize: impl FnOnce(&mut CertificateParams),
        ) -> CertificateDer<'static> {
            let names = names.iter().map(ToString::to_string).collect::<Vec<_>>();
            let mut params = CertificateParams::new(names).unwrap();
            params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
            customize(&mut params);
            params.signed_by(key, &self.0).unwrap().der().clone()
        }
    }

    /// Accepts a single connection on localhost and runs a rustls 0.23 server handshake on it,
    /// restricted to `version`, presenting `cert` with `key`.
    fn serve(
        cert: CertificateDer<'static>,
        key: &KeyPair,
        version: &'static SupportedProtocolVersion,
    ) -> SocketAddr {
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.serialize_der()));
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_protocol_versions(&[version])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut conn = rustls::ServerConnection::new(Arc::new(config)).unwrap();
            // Rejections show up on the client side; the server just stops on any error.
            while conn.is_handshaking() {
                if conn.complete_io(&mut sock).is_err() {
                    return;
                }
            }
            let mut tls = rustls::Stream::new(&mut conn, &mut sock);
            let mut buf = [0; 4];
            if tls.read_exact(&mut buf).is_ok() {
                let _ = tls.write_all(&buf);
            }
        });
        addr
    }

    /// Connects to `addr` as `server_name` with the config [`client_config`] builds for
    /// `roots`, completes the handshake and exchanges a message.
    fn connect(
        addr: SocketAddr,
        server_name: &str,
        roots: RootCertStore,
    ) -> Result<ProtocolVersion, rustls_0_21::Error> {
        let mut config = client_config_with_roots(roots).unwrap();
        // The test server speaks raw bytes, not HTTP.
        config.alpn_protocols.clear();
        let server_name = rustls_0_21::ServerName::try_from(server_name).unwrap();
        let mut conn = ClientConnection::new(Arc::new(config), server_name).unwrap();
        let mut sock = TcpStream::connect(addr).unwrap();
        let tls_error = |e: std::io::Error| {
            *e.into_inner()
                .and_then(|e| e.downcast::<rustls_0_21::Error>().ok())
                .expect("expected a TLS error")
        };
        while conn.is_handshaking() {
            conn.complete_io(&mut sock).map_err(tls_error)?;
        }
        let mut tls = rustls_0_21::Stream::new(&mut conn, &mut sock);
        tls.write_all(b"ping").unwrap();
        let mut buf = [0; 4];
        tls.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"ping");
        Ok(conn.protocol_version().unwrap())
    }

    /// Serves a certificate issued by `ca` for `cert_names` over `version`, and connects to it
    /// as `server_name` trusting `client_roots`.
    fn handshake(
        ca: &TestCa,
        cert_names: &[&str],
        customize: impl FnOnce(&mut CertificateParams),
        version: &'static SupportedProtocolVersion,
        server_name: &str,
        client_roots: RootCertStore,
    ) -> Result<ProtocolVersion, rustls_0_21::Error> {
        let key = KeyPair::generate().unwrap();
        let cert = ca.issue(cert_names, &key, customize);
        connect(serve(cert, &key, version), server_name, client_roots)
    }

    fn assert_certificate_error(
        result: Result<ProtocolVersion, rustls_0_21::Error>,
        expected: &str,
    ) {
        match result {
            Err(rustls_0_21::Error::InvalidCertificate(e)) => {
                assert!(
                    format!("{e:?}").contains(expected),
                    "{expected} not in {e:?}"
                );
            }
            other => panic!("expected an invalid certificate error, got {other:?}"),
        }
    }

    /// `DigitallySignedStruct` has no public constructor in rustls 0.21, so decode it from its
    /// wire format: a u16 scheme followed by a u16-length-prefixed signature.
    fn dss(scheme: SignatureScheme, signature: &[u8]) -> DigitallySignedStruct {
        let mut wire = u16::from(scheme).to_be_bytes().to_vec();
        wire.extend(u16::try_from(signature.len()).unwrap().to_be_bytes());
        wire.extend(signature);
        DigitallySignedStruct::read_bytes(&wire).unwrap()
    }

    /// A P-256 certificate and its key's DER-encoded ECDSA signature of [`MESSAGE`].
    fn signed_message(ca: &TestCa) -> (rustls_0_21::Certificate, Vec<u8>) {
        let key = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).unwrap();
        let cert = ca.issue(&["localhost"], &key, |_| {});
        (
            rustls_0_21::Certificate(cert.to_vec()),
            key.sign(MESSAGE).unwrap(),
        )
    }

    #[test]
    fn client_config_is_accepted_by_reqwest() {
        // `use_preconfigured_tls` takes `impl Any` and only fails in `build` if the config's
        // rustls version doesn't match reqwest's, which would make `from_rpc_url` panic.
        reqwest_0_11::ClientBuilder::new()
            .use_preconfigured_tls(client_config().unwrap())
            .build()
            .unwrap();
    }

    #[test]
    fn completes_tls12_and_tls13_handshakes() {
        let ca = TestCa::new("test CA");
        for (version, expected) in [
            (&TLS12, ProtocolVersion::TLSv1_2),
            (&TLS13, ProtocolVersion::TLSv1_3),
        ] {
            let negotiated = handshake(
                &ca,
                &["localhost"],
                |_| {},
                version,
                "localhost",
                ca.roots(),
            )
            .unwrap();
            assert_eq!(negotiated, expected);
        }
    }

    #[test]
    fn verifies_ip_address_server_names() {
        let ca = TestCa::new("test CA");
        for version in [&TLS12, &TLS13] {
            handshake(
                &ca,
                &["127.0.0.1"],
                |_| {},
                version,
                "127.0.0.1",
                ca.roots(),
            )
            .unwrap();
            // Same address, but the certificate is for a different one.
            assert_certificate_error(
                handshake(
                    &ca,
                    &["127.0.0.2"],
                    |_| {},
                    version,
                    "127.0.0.1",
                    ca.roots(),
                ),
                "NotValidForName",
            );
        }
    }

    #[test]
    fn rejects_wrong_hostname() {
        let ca = TestCa::new("test CA");
        for version in [&TLS12, &TLS13] {
            assert_certificate_error(
                handshake(
                    &ca,
                    &["localhost"],
                    |_| {},
                    version,
                    "example.com",
                    ca.roots(),
                ),
                "NotValidForName",
            );
        }
    }

    #[test]
    fn rejects_expired_certificate() {
        let ca = TestCa::new("test CA");
        let expired = |params: &mut CertificateParams| {
            params.not_before = date_time_ymd(2000, 1, 1);
            params.not_after = date_time_ymd(2001, 1, 1);
        };
        for version in [&TLS12, &TLS13] {
            assert_certificate_error(
                handshake(
                    &ca,
                    &["localhost"],
                    expired,
                    version,
                    "localhost",
                    ca.roots(),
                ),
                "Expired",
            );
        }
    }

    #[test]
    fn rejects_unknown_issuer() {
        let ca = TestCa::new("test CA");
        let other_ca = TestCa::new("other test CA");
        for version in [&TLS12, &TLS13] {
            assert!(matches!(
                handshake(
                    &ca,
                    &["localhost"],
                    |_| {},
                    version,
                    "localhost",
                    other_ca.roots()
                ),
                Err(rustls_0_21::Error::InvalidCertificate(
                    CertificateError::UnknownIssuer
                ))
            ));
        }
    }

    #[test]
    fn accepts_valid_signature() {
        let ca = TestCa::new("test CA");
        let (cert, signature) = signed_message(&ca);
        let dss = dss(SignatureScheme::ECDSA_NISTP256_SHA256, &signature);
        let verifier = Verifier::new(ca.roots()).unwrap();
        verifier
            .verify_tls12_signature(MESSAGE, &cert, &dss)
            .unwrap();
        verifier
            .verify_tls13_signature(MESSAGE, &cert, &dss)
            .unwrap();
    }

    #[test]
    fn rejects_tampered_signature_as_bad_signature() {
        let ca = TestCa::new("test CA");
        let (cert, mut signature) = signed_message(&ca);
        *signature.last_mut().unwrap() ^= 1;
        let dss = dss(SignatureScheme::ECDSA_NISTP256_SHA256, &signature);
        let verifier = Verifier::new(ca.roots()).unwrap();
        // TLS 1.2 maps ECDSA_NISTP256_SHA256 to both the P-256 and P-384 algorithms; the P-256
        // failure must be reported rather than the P-384 key-type mismatch that follows it.
        for result in [
            verifier.verify_tls12_signature(MESSAGE, &cert, &dss),
            verifier.verify_tls13_signature(MESSAGE, &cert, &dss),
        ] {
            assert!(matches!(
                result,
                Err(rustls_0_21::Error::InvalidCertificate(
                    CertificateError::BadSignature
                ))
            ));
        }
    }

    #[test]
    fn rejects_tls12_only_scheme_in_tls13() {
        let ca = TestCa::new("test CA");
        let (cert, signature) = signed_message(&ca);
        let dss = dss(SignatureScheme::ECDSA_SHA1_Legacy, &signature);
        assert!(matches!(
            Verifier::new(ca.roots())
                .unwrap()
                .verify_tls13_signature(MESSAGE, &cert, &dss),
            Err(rustls_0_21::Error::PeerMisbehaved(
                rustls_0_21::PeerMisbehaved::SignedHandshakeWithUnadvertisedSigScheme
            ))
        ));
    }

    #[test]
    fn converts_certificate_errors() {
        assert!(matches!(
            convert_error(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer
            )),
            rustls_0_21::Error::InvalidCertificate(CertificateError::UnknownIssuer)
        ));
        assert!(matches!(
            convert_error(rustls::Error::InvalidCertificate(
                rustls::CertificateError::NotValidForNameContext {
                    expected: ServerName::try_from("example.com").unwrap(),
                    presented: vec!["DnsName(\"other.com\")".to_string()],
                }
            )),
            rustls_0_21::Error::InvalidCertificate(CertificateError::Other(_))
        ));
        assert!(matches!(
            convert_error(rustls::Error::NoCertificatesPresented),
            rustls_0_21::Error::General(_)
        ));
    }
}
