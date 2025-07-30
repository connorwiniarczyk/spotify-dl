use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::oauth;

use librespot::core::spotify_id::SpotifyId;
use librespot::core::spotify_id::SpotifyItemType;

use librespot::playback::config as playback_config;
use librespot::playback::player;
use librespot::playback::player::PlayerEvent;
use librespot::playback::mixer;

use futures_executor::block_on;

use librespot::metadata::{
    Metadata,
    Playlist,
    Track,
    Album,
    // Artist,
};

use lazy_static::lazy_static;
use regex::Regex;

use std::io::Write;
use std::env;

use tokio;

mod record;
use record::RecordSink;

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

fn try_automatic_login(session: &Session) -> Result<(), ()> {
    let token_cache = "access_token.txt";

    let token = std::fs::read_to_string(token_cache).or(Err(()))?;
    env::set_var("SPOTIFY_DL_ACCESS_TOKEN", &token);
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

    env::set_var("SPOTIFY_DL_ACCESS_TOKEN", &oauth_token.access_token);

    let creds = Credentials::with_access_token(oauth_token.access_token);
    block_on(session.connect(creds, false)).or(Err(()))?;

    Ok(())
}

fn get_tracks_to_download(id: SpotifyId, session: &Session) -> Vec<SpotifyId> {
    let mut output = Vec::new();

	match id.item_type {
    	SpotifyItemType::Playlist => {
            let playlist = block_on(Playlist::get(&session, &id)).expect("failed to fetch playlist");
            println!("{:#?}", playlist);

            let mut count = 0;
			for track in playlist.tracks() {
    			output.push(*track);
    			count += 1;
			}
            println!("found playlist: {} with {} songs", playlist.name(), count);
    	},

    	SpotifyItemType::Album => {
            let album = block_on(Album::get(&session, &id)).expect("failed to fetch playlist");
            println!("{:#?}", album);
            let mut count = 0;
			for track in album.tracks() {
    			output.push(*track);
    			count += 1;
			}
            println!("found album: {} with {} songs", album.name, count);
    	},

    	SpotifyItemType::Track    => {
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

async fn record_track(track: SpotifyId, session: Session) -> Result<(), ()> {
    let mut player_config = playback_config::PlayerConfig::default();
    player_config.passthrough = true;
    let metadata = Track::get(&session, &track).await.unwrap();

    let path = format!("spotify-dl/{}.ogg", track.to_base62().unwrap());

    if std::fs::Path::exists(path) {
        println!("skipping {}, already exists");
    }

    let player = player::Player::new(player_config, session, Box::new(mixer::NoOpVolume), move || {
        RecordSink::create(metadata)
    });

    player.load(track, true, 0);

    let mut channel = player.get_player_event_channel();
    while let Some(event) = channel.recv().await {
        match event {
            PlayerEvent::Playing      {..} => {},
            PlayerEvent::TrackChanged {..} => {},
            PlayerEvent::Unavailable  {..} => {
                println!("spotify:track:{} is unavailable, aborting", track.to_base62().unwrap());
                break;
            },

            PlayerEvent::Paused {..} => {
                println!("player paused, aborting");
                player.stop();
                break;
            },

            PlayerEvent::EndOfTrack {..} => {
                player.stop();
                break;
            },

            PlayerEvent::TimeToPreloadNextTrack {..} => (),
            event => println!("{:?}", event),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    _ = args.next();
    let arg = args.next().expect("paste a spotify link as an argument");

    let config = SessionConfig::default();
    let session = Session::new(config, None);

    if try_automatic_login(&session).is_err() {
        println!("failed automatic login, please login manually");
        try_manual_login(&session).expect("failed to login");
    }

    let _ = std::fs::create_dir("spotify-dl");

	let id = parse_spotify_link(&arg).expect("invalid link");
	let tracks = get_tracks_to_download(id, &session);

	println!("downloading {} tracks with ids:", tracks.len());
	for id in tracks.iter() {
    	println!("spotify:track:{}", id.to_base62().unwrap());
	}
	println!();

	for id in tracks {
        if record_track(id, session.clone()).await.is_err() {
            println!("failed");
        }
	}
}
