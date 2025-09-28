// src/api
use std::sync::LazyLock;
use reqwest::Client;
use std::time::Duration;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("ai.openspace.tactic/0.0.1")
        .build()
        .expect("client")
});

pub fn http_client() -> Client { HTTP_CLIENT.clone() }
