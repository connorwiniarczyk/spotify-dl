use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::oauth;

use tokio;


#[tokio::main]
async fn main() {

    let client_id = "c85b2435db4948bab5fcd3386b77170c";

	let mut privelages = Vec::new();
	privelages.push("playlist-read-private");
	privelages.push("streaming");

    let oauth_token = oauth::get_access_token(client_id, "http://localhost:8888/callback", privelages).expect("failed");
    let creds = Credentials::with_access_token(oauth_token.access_token);

    let config = SessionConfig::default();
    let session = Session::new(config, None);

    Session::connect(&session, creds, false).await.expect("failed to connect");

    println!("CONNECTED");
}
