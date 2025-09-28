// src/api
use std::sync::LazyLock;
use reqwest::Client;
use std::time::Duration;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("ai.openspace.tactic/0.0.1")
        .build()
        .expect("client")
});

// optional helper: cheap to clone; both forms are fine
pub fn http() -> &'static Client { &HTTP_CLIENT }
// or:
// pub fn http() -> Client { HTTP_CLIENT.clone() }
