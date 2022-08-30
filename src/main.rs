mod config;
use config::Config;

use futures::join;

use std::{env, process::exit};
use tokio;
use inquire::Text;
use std::fs::File;
use std::io::{BufReader, BufRead};
use tokio::process::{Command};
use std::process::Stdio;

use librespot::{
    core::{
        authentication::Credentials, config::SessionConfig, session::Session, spotify_id::SpotifyId,
    },
    playback::{
        audio_backend,
        config::{AudioFormat, PlayerConfig},
        mixer::NoOpVolume,
        player::Player,
    },
    metadata::{
        Metadata, Playlist, Track, Album, Artist
    },
};

struct TrackInfo {
    name: String,
    artists: Vec<String>,
    album: String,
}

impl TrackInfo {
    async fn get(track: &Track, session: &Session) -> Self {

        let album = Album::get(session, track.album).await.unwrap().name;

        let mut artists: Vec<String> = vec![];
        for artist_id in track.artists.iter() {
            let artist = Artist::get(session, artist_id.clone()).await.unwrap();
            artists.push(artist.name);
        }

        Self {
            name: track.name.clone(),
            artists,
            album,
        }
    }
}

fn get_creds() -> Result<(Credentials, bool), inquire::InquireError> {
    let username = Text::new("Spotify Username:")
        .prompt()?;

    let password = inquire::Password::new("Spotify Password: ")
        .prompt()?;

    let save = inquire::Confirm::new("Save?")
        .with_default(true)
        .with_help_message("Save these credentials to a local file so you don't have to type them again")
        .prompt()?;

    let creds = Credentials::with_password(&username, &password);

    Ok((creds, save))
}

async fn download_track(track: Track, track_id: SpotifyId, config: &Config, session: &Session) {
    let output_path = format!("{}.flac", track.name);

    let track_info = TrackInfo::get(&track, session).await;
    println!("{}", &track.name);
    // let output = File::create(&output_path).expect("failed to open file");
    let mut downloader = Command::new("spot-dl")
        .arg(&config.username)
        .arg(&config.password)
        .arg(&track_id.to_base62().unwrap())
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
        .arg("-metadata").arg(&format!("title={}", track_info.name))
        .arg("-metadata").arg(&format!("album={}", track_info.name))
        .arg("-metadata").arg(&format!("artist={}", track_info.artists[0])) // TODO: multiple artists
        .arg(&output_path)
        .stdin(downloader_out)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to open process");

    futures::join!(downloader.wait(), converter.wait());
}

fn read_creds() -> Result<(String, String), std::io::Error> {
    let file = File::open("creds.txt").unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let username = lines.next().unwrap().unwrap();
    let password = lines.next().unwrap().unwrap();

    Ok((username, password))
}

use std::env::current_dir;


#[tokio::main]
async fn main() {
    let config = Config::generate();
    let creds = Credentials::with_password(&config.username, &config.password);
    let (session, conf) = Session::connect(SessionConfig::default(), creds, None, false).await.expect("failed to connect");

    let source_id = SpotifyId::from_uri(&config.uri())
        .expect("not a valid playlist");

    let track_ids: Vec<SpotifyId> = match config.audio_type.as_str() {
        "playlist" => {
            let playlist = Playlist::get(&session, source_id).await.expect("failed to fetch playlist");
            println!("{}", &playlist.name);
            std::fs::create_dir(&playlist.name).expect("could not create directory for playlist");
            std::env::set_current_dir(current_dir().unwrap().join(&playlist.name));
            playlist.tracks
        }, 
        "album" => {
            let album = Album::get(&session, source_id).await.expect("failed to fetch album");
            println!("{}", &album.name);
            std::fs::create_dir(&album.name).expect("could not create directory for album");
            std::env::set_current_dir(current_dir().unwrap().join(&album.name));
            album.tracks
        }, 
        "track" => {
            vec![source_id]
        },
        "artist" => panic!("cannot download artists"),

        _ => panic!("not a valid audio type"),
    };

    // std::fs::create_dir(&playlist.name).expect("could not create directory for playlist");
    // std::env::set_current_dir(current_dir().unwrap().join(&playlist.name));

    for track_id in track_ids {
        let track = Track::get(&session, track_id).await.unwrap();
        download_track(track, track_id, &config, &session).await;
    }
}
        

