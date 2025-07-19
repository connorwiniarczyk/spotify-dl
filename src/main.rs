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

use std::time;

use futures_executor::block_on;

use std::process::Stdio;
use std::process::Command;

use librespot::metadata::{
    Metadata,
    Playlist,
    Track,
    // Album,
    // Artist,
};

use lazy_static::lazy_static;
use regex::Regex;

use std::io::Write;
use std::env;

use tokio;

fn get_stored_access_token() -> Option<String> {
	return std::fs::read_to_string("access_token.txt").ok();
}

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

fn run_track_downloader(id: SpotifyId, session: &Session) -> Result<(), ()> {
    let track = block_on(Track::get(session, &id)).or(Err(()))?;

    let mut artists = String::new();

    let mut i = 0;
    while i < track.artists.len() {
        artists.push_str(track.artists[i].name.as_str());
        i += 1;
        if i < track.artists.len() {
            artists.push_str(",");
        }
    }

    let uri = format!("spotify:track:{}", id.to_base62().unwrap());
    let output_path = format!("{}.ogg", track.name);

    println!("downloading {} [{}]", track.name, artists);

    // let mut downloader = Command::new("../target/debug/spotify-dl-downloader")
    let mut downloader = Command::new("spotify-dl-downloader")
        .arg(uri)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to open process");

    let downloader_out: Stdio = downloader
        .stdout
        .take()
        .unwrap()
        .try_into()
        .unwrap();

    let mut converter = Command::new("ffmpeg")
        .arg("-f").arg("ogg")
        .arg("-i").arg("pipe:")
        .arg("-metadata").arg(&format!("title={}", track.name))
        .arg("-metadata").arg(&format!("album={}", track.album.name))
        .arg("-metadata").arg(&format!("artist={}", artists))
        .arg(output_path)
        .stdin(downloader_out)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to open ffmpeg");

	let mut downloader_done = false;
	let mut converter_done = false;

	let time_begin = std::time::Instant::now();

	while !downloader_done && !converter_done {
    	match downloader.try_wait() {
        	Ok(Some(status)) => {
            	downloader_done = true;
            	if !status.success() {
                	converter.kill();
            	}
        	},
        	Ok(None)    => { },
        	Err(e)      => { println!("error: {:?}", e); },
    	};

    	match converter.try_wait() {
        	Ok(Some(status)) => {
            	converter_done = true;
            	if !status.success() {
                	downloader.kill();
            	}
        	},
        	Ok(None)    => { },
        	Err(e)      => { },
    	};

    	if time_begin.elapsed().as_secs() > 20 {
        	println!("timed out, aborting");
        	converter.kill();
        	downloader.kill();
        	return Err(());
    	}

    	std::thread::sleep(time::Duration::from_millis(100));
	}

	return Ok(())
}

fn get_tracks_to_download(id: SpotifyId, session: &Session) -> Vec<SpotifyId> {
    let mut output = Vec::new();

	match id.item_type {
    	SpotifyItemType::Playlist => {

            let playlist = block_on(Playlist::get(&session, &id)).expect("failed to fetch playlist");
            let mut count = 0;

            let _ = std::fs::create_dir(&playlist.name());
            env::set_current_dir(env::current_dir().unwrap().join(&playlist.name()));

			for track in playlist.tracks() {
    			output.push(*track);
    			count += 1;
			}
            println!("found playlist: {} with {} songs", playlist.name(), count);

    	},
    	SpotifyItemType::Album    => println!("album"),
    	SpotifyItemType::Artist   => println!("artist"),
    	SpotifyItemType::Track    => println!("track"),
    	SpotifyItemType::Episode  => println!("episode"),
    	SpotifyItemType::Show     => println!("show"),
    	SpotifyItemType::Local    => println!("local"),
    	SpotifyItemType::Unknown  => println!("unknown"),
	}

	return output;
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

	let id = parse_spotify_link(&arg).expect("invalid link");
	let tracks = get_tracks_to_download(id, &session);

	for id in tracks {
        if run_track_downloader(id, &session).is_err() {
            println!("failed");
        }
	}
}
