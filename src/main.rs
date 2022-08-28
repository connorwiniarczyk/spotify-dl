mod config;
use config::Config;

use std::{env, process::exit};
use tokio;
use inquire::Text;
use std::fs::File;
use std::io::{BufReader, BufRead};
use tokio::process::Command;

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
        Metadata, Playlist, Track
    },
};

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

async fn download_track(track: Track, track_id: SpotifyId, config: &Config) {
    let output_path = format!("{}.ogg", track.name);
    println!("{:#?}", track);
    println!("donwloading to {}", &output_path);
    let output = File::create(&output_path).expect("failed to open file");
    // let mut downloader = Command::new("spot-dl")
    //     .arg(&username)
    //     .arg(&password)
    //     .arg(&track_id.to_base62().unwrap())
    //     .stdout(output)
    //     .spawn()
    //     .expect("failed to open process");

    let mut converter = Command::new("ffmpeg")
        .arg("-i stdin")
        .arg(r#"-metadata title="" "#)
        .arg("");

    // downloader.wait().await;
}

fn read_creds() -> Result<(String, String), std::io::Error> {
    let file = File::open("creds.txt").unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let username = lines.next().unwrap().unwrap();
    let password = lines.next().unwrap().unwrap();

    Ok((username, password))
}


#[tokio::main]
async fn main() {
    let config = Config::generate();

    // let (username, password) = read_creds().unwrap();
    let creds = Credentials::with_password(&config.username, &config.password);
    let playlist_id = SpotifyId::from_uri(&config.uri).expect("not a valid playlist");
    let (session, conf) = Session::connect(SessionConfig::default(), creds, None, false).await.expect("failed to connect");

    let playlist = Playlist::get(&session, playlist_id).await.expect("failed to fetch playlist");

    for track_id in playlist.tracks {
        let track = Track::get(&session, track_id).await.unwrap();
        download_track(track, track_id, &config).await;
    }
}
        

