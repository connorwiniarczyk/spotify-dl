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

use librespot::metadata::{
    Metadata,
    Playlist,
    Track,
    // Album,
    // Artist,
};



use std::env;

use regex::Regex;
use lazy_static::lazy_static;


fn parse_spotify_link(input: &str) -> Result<SpotifyId, ()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"open\.spotify\.com/(?P<type>.+?)/(?P<id>.+?)$"#).unwrap();
    }

    let captures = RE.captures(input).ok_or(())?;
    let audio_type = captures.name("type").ok_or(())?.as_str();

    let id_full = captures.name("id").ok_or(())?.as_str();
    let id = id_full.split("?").next().unwrap();

    let uri = format!("spotify:{}:{}", audio_type, id);

    return SpotifyId::from_uri(&uri).or(Err(()))
}

#[tokio::main]
async fn main() {
    let token = env::var("SPOTIFY_DL_ACCESS_TOKEN").expect("no access token defined in the environment");
    let creds = Credentials::with_access_token(token);
    let config = SessionConfig::default();
    let session = Session::new(config, None);
    session.connect(creds, false).await.expect("failed to connect");

	let mut args = std::env::args();
	let _ = args.next();

	let uri = args.next().unwrap();
    let track = SpotifyId::from_uri(&uri).unwrap();

    // let track = SpotifyId::from_uri(&args.next().unwrap()).unwrap();
    // let test_url = "https://open.spotify.com/track/0qOjRPOrzbMT3KkruSmL2P?si=c732c3b04f3645a9";
    // let test_url = "https://open.spotify.com/track/4LtVrrqpn48l4Iq7KIshmi?si=aa634c29c0694ae7";
    // let test_uri = "https://open.spotify.com/track/4LtVrrqpn48l4Iq7KIshmi?si=aa634c29c0694ae7";
    
    // let track_id_base62 = "spotify:track:4LtVrrqpn48l4Iq7KIshmi";
    // let track = SpotifyId::from_uri(track_id_base62).unwrap();

    // let track = parse_spotify_link(test_url).unwrap();

    let backend = audio_backend::find(Some("pipe".to_string())).expect("couldn't open the pipe audio backend");

    // let track = SpotifyId::from_base62(&track_id_base62).unwrap();
    // let track = parse_spotify_link(test_url).unwrap();
    // let info = Track::get(&session, &track).await.unwrap();

    // println!("track : {}", info.name);

    let mut player_config = playback_config::PlayerConfig::default();
    player_config.passthrough = true;
    let audio_format = playback_config::AudioFormat::default();

    let mut player = player::Player::new(player_config, session, Box::new(mixer::NoOpVolume), move || {
        backend(None, audio_format)
    });

    player.load(track, true, 0);
    player.await_end_of_track().await;

}
