// src/api
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl};
use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

const USER_AGENT: &str = "ai.openspace.tactic/0.0.1";
// TODO Config?
const AUTH0_CLIENT_ID: &str = "B85VbSiRcD92gcDOhgfQG6CPueV2HgwH";
const AUTH0_DOMAIN: &str = "login.openspace.ai";

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(USER_AGENT)
        .build()
        .expect("client")
});

static OAUTH_CLIENT: LazyLock<BasicClient> = LazyLock::new(|| {
    BasicClient::new(
        ClientId::new(AUTH0_CLIENT_ID.to_string()),
        None,
        AuthUrl::new("https://{AUTH0_DOMAIN}/authorize".to_string()).expect("auth url"),
        None,
    )
    .set_redirect_uri(RedirectUrl::new("https://openspace.ai".to_string()).expect("redirect url"))
});

pub fn http_client() -> Client {
    HTTP_CLIENT.clone()
}

pub fn login() {
    // 1) Get the OAuth client
    let client = OAUTH_CLIENT.clone();

    // 2) PKCE
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // 3) Build Auth
    let (auth_url, _csrf) = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge)
        // Add whatever scopes OpenSpace’s API requires:
        .add_scope(Scope::new("openid".into()))
        .add_scope(Scope::new("profile".into()))
        .add_scope(Scope::new("email".into()))
        .url();

    println!("opening Browser");
}
