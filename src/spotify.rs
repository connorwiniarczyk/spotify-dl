use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::oauth;

use librespot::playback::config as playback_config;
use librespot::playback::player;
use librespot::playback::player::PlayerEvent;
use librespot::playback::mixer;

use librespot::core::spotify_id::SpotifyId;
use librespot::core::spotify_id::SpotifyItemType;

use librespot::metadata::{
    Metadata,
    Playlist,
    Track,
    Album,
    // Artist,
};

use futures_executor::block_on;

use lazy_static::lazy_static;
use regex::Regex;

use std::io::Write;
use std::env;
use std::path::Path;

use crate::RecordSink;

pub fn parse_link(input: &str) -> Result<SpotifyId, ()> {
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

pub fn get_stored_credentials() -> Result<Credentials, ()> {
    let token_cache = "access_token.txt";

    let token = std::fs::read_to_string(token_cache).or(Err(()))?;
    env::set_var("SPOTIFY_DL_ACCESS_TOKEN", &token);
    let creds = Credentials::with_access_token(token);

    // block_on(session.connect(creds, false)).or(Err(()))?;
    Ok(creds)
}

pub fn try_manual_login() -> Result<Credentials, ()> {
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

    env::set_var("SPOTIFY_DL_ACCESS_TOKEN", &oauth_token.access_token);
    let creds = Credentials::with_access_token(oauth_token.access_token);
    Ok(creds)
}

pub fn connect(session: &Session) -> Result<(), ()> {
    if let Ok(creds) = get_stored_credentials() {
        if block_on(session.connect(creds, true)).is_ok() {
			return Ok(());
        }
    }

    if let Ok(creds) = try_manual_login() {
        if block_on(session.connect(creds, true)).is_ok() {
			return Ok(());
        }
    }

    return Err(())
}

pub fn get_tracks_to_download(id: SpotifyId, session: &Session) -> Vec<SpotifyId> {
    let mut output = Vec::new();

	match id.item_type {
    	SpotifyItemType::Playlist => {
            let playlist = block_on(Playlist::get(&session, &id)).expect("failed to fetch playlist");
            let mut count = 0;
			for track in playlist.tracks() {
    			output.push(*track);
    			count += 1;
			}
            println!("found playlist: {} with {} songs", playlist.name(), count);
    	},

    	SpotifyItemType::Album => {
            let album = block_on(Album::get(&session, &id)).expect("failed to fetch album");
            let mut count = 0;
			for track in album.tracks() {
    			output.push(*track);
    			count += 1;
			}
            println!("found album: {} with {} songs", album.name, count);
    	},

    	SpotifyItemType::Track    => {
            let track = block_on(Track::get(&session, &id)).expect("failed to fetch track");
            println!("found track: {}", track.name);
        	output.push(id);
    	},

    	SpotifyItemType::Episode  => println!("episode"),
    	SpotifyItemType::Show     => println!("show"),
    	SpotifyItemType::Local    => println!("local"),
    	SpotifyItemType::Artist   => println!("artist"),
    	SpotifyItemType::Unknown  => println!("unknown"),
	}

	return output;
}

pub async fn record_track(track: SpotifyId, session: Session) -> Result<String, String> {
    let mut player_config = playback_config::PlayerConfig::default();
    player_config.passthrough = true;
    let metadata = Track::get(&session, &track).await.unwrap();
    let path = Path::new(&track.to_base62().unwrap()).with_extension("ogg");

    if std::path::Path::new(&path).exists() {
        return Err("already exists".into());
    }

    let name = metadata.name.clone();

    let player = player::Player::new(player_config, session, Box::new(mixer::NoOpVolume), move || {
        RecordSink::create(&path, metadata)
    });

    player.load(track, true, 0);

    let mut channel = player.get_player_event_channel();
    while let Some(event) = channel.recv().await {
        match event {
            PlayerEvent::Playing      {..} => {},
            PlayerEvent::TrackChanged {..} => {},
            PlayerEvent::TimeToPreloadNextTrack {..} => (),

            PlayerEvent::Unavailable  {..} => {
                return Err(format!("unavailable, you can try again later - {}", name));
            },

            PlayerEvent::Paused {..} => {
                player.stop();
                return Err(format!("received the pause command, aborting - {}", name));
            },

            PlayerEvent::EndOfTrack {..} => {
                player.stop();
                return Ok(name);
            },

            event => println!("{:?}", event),
        }
    }

    Ok(format!("{}", name))
}
