use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::oauth;

use librespot::core::spotify_id::SpotifyId;
use librespot::core::spotify_id::SpotifyItemType;

use librespot::playback::audio_backend;
use librespot::playback::config as playback_config;
use librespot::playback::player;
use librespot::playback::mixer;

use futures_executor::block_on;

use std::env;

use regex::Regex;
use lazy_static::lazy_static;


fn reformat_spotify_uri(input: &str) -> Result<String, ()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"open\.spotify\.com/(?P<type>.+?)/(?P<id>.+?)$"#).unwrap();
    }

    let captures = RE.captures(input).ok_or(())?;
    let audio_type = captures.name("type").ok_or(())?.as_str();

    let id_full = captures.name("id").ok_or(())?.as_str();
    let id = id_full.split("?").next().unwrap();

    return Ok(format!("spotify:{}:{}", audio_type, id));

}

fn get_track(input: &str) -> Option<SpotifyId> {
    let spotify_formatted_uri = reformat_spotify_uri(input).ok()?;
    return SpotifyId::from_uri(&spotify_formatted_uri).ok()

	// let id = SpotifyId::from_uri(&reformat_spotify_uri(&uri).unwrap()).expect("invalid uri");
}

#[tokio::main]
async fn main() {
    let test_url = "https://open.spotify.com/track/0qOjRPOrzbMT3KkruSmL2P?si=c732c3b04f3645a9";

    let token = env::var("SPOTIFY_DL_ACCESS_TOKEN").expect("no access token defined in the environment");
    let creds = Credentials::with_access_token(token);

    let config = SessionConfig::default();
    let session = Session::new(config, None);
    block_on(session.connect(creds, false)).expect("failed to connect");

    let backend = audio_backend::find(Some("pipe".to_string())).expect("couldn't open the pipe audio backend");
    // let backend = audio_backend::find(Some("rodio".to_string())).expect("couldn't open the pipe audio backend");

    let track = get_track(test_url).expect("failed to get track");

    let mut player_config = playback_config::PlayerConfig::default();
    player_config.passthrough = true;
    let audio_format = playback_config::AudioFormat::default();

    let mut player = player::Player::new(player_config, session, Box::new(mixer::NoOpVolume), move || {
        backend(None, audio_format)
    });

    player.load(track, true, 0);
    player.await_end_of_track().await;

}
