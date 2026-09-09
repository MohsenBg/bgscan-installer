use reqwest::blocking::Client;
use reqwest::tls::Certificate;

pub fn build_client(user_agent: &str) -> reqwest::Result<Client> {
    let certs: Vec<Certificate> = webpki_root_certs::TLS_SERVER_ROOT_CERTS
        .iter()
        .map(|cert| Certificate::from_der(cert.as_ref()))
        .collect::<Result<_, _>>()?;

    Client::builder()
        .tls_certs_only(certs)
        .user_agent(user_agent)
        .build()
}
