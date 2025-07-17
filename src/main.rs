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

// use librespot::metadata::{
// Metadata,
// Playlist,
// Track,
// Album,
// Artist,
// };

use lazy_static::lazy_static;
use regex::Regex;

use std::io::Write;
use std::env;

use tokio;

fn get_stored_access_token() -> Option<String> {
	return std::fs::read_to_string("access_token.txt").ok();
}

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

fn try_automatic_login(session: &Session) -> Result<(), ()> {
    let token_cache = "access_token.txt";

    let token = std::fs::read_to_string(token_cache).or(Err(()))?;
    let creds = Credentials::with_access_token(token);

    block_on(session.connect(creds, false)).or(Err(()))?;
    Ok(())
}

fn try_manual_login(session: &Session) -> Result<(), ()> {
    let client_id = "c85b2435db4948bab5fcd3386b77170c";

    let mut privelages = Vec::new();
    privelages.push("playlist-read-private");
    privelages.push("streaming");

    let oauth_token = oauth::get_access_token(client_id, "http://localhost:8888/callback", privelages).or(Err(()))?;

    if let Ok(mut out) = std::fs::File::create("access_token.txt") {
    	println!("saving access token to access_token.txt");
    	out.write_all(oauth_token.access_token.as_bytes()).or(Err(()))?;
    } else {
    	println!("could not save access token");
    }

    let creds = Credentials::with_access_token(oauth_token.access_token);
    block_on(session.connect(creds, false)).or(Err(()))?;

    Ok(())
}

fn get_track(input: Option<&str>) -> Option<SpotifyId> {
    let spotify_formatted_uri = reformat_spotify_uri(input?).ok()?;
    return SpotifyId::from_uri(&spotify_formatted_uri).ok()

	// let id = SpotifyId::from_uri(&reformat_spotify_uri(&uri).unwrap()).expect("invalid uri");
}

#[tokio::main]
async fn main() {
    // let mut args = env::args();
    // args.next();

    let arg = "https://open.spotify.com/track/0qOjRPOrzbMT3KkruSmL2P?si=c732c3b04f3645a9";

    let config = SessionConfig::default();
    let session = Session::new(config, None);

    if try_automatic_login(&session).is_err() {
        println!("failed automatic login, please login manually");
        try_manual_login(&session).expect("failed to login");
    }

    let backend = audio_backend::find(Some("pipe".to_string())).expect("couldn't open the pipe audio backend");
    // let backend = audio_backend::find(Some("rodio".to_string())).expect("couldn't open the pipe audio backend");
    let track = get_track(Some(arg)).expect("failed to get track");


    let mut player_config = playback_config::PlayerConfig::default();
    player_config.passthrough = true;
    let audio_format = playback_config::AudioFormat::default();

    // let track = SpotifyId::from_base62(&args.next().unwrap()).unwrap();
    // let backend = audio_backend::find(Some("pipe".to_string())).unwrap();

    // let (session, creds) = Session::connect(session_config, credentials, None, false).await.expect("error connecting");

    let mut player = player::Player::new(player_config, session, Box::new(mixer::NoOpVolume), move || {
        backend(None, audio_format)
    });

    println!("playing track");

    player.load(track, true, 0);
    player.await_end_of_track().await;

	// if let Some(uri) = args.next() {
 //    	let id = SpotifyId::from_uri(&reformat_spotify_uri(&uri).unwrap()).expect("invalid uri");
 //    	match id.item_type {
 //        	SpotifyItemType::Album    => println!("album"),
 //        	SpotifyItemType::Artist   => println!("artist"),
 //        	SpotifyItemType::Track    => println!("track"),
 //        	SpotifyItemType::Playlist => println!("playlist"),
 //        	SpotifyItemType::Episode  => println!("episode"),
 //        	SpotifyItemType::Show     => println!("show"),
 //        	SpotifyItemType::Local    => println!("local"),
 //        	SpotifyItemType::Unknown  => println!("unknown"),
 //    	}
	// }
}
