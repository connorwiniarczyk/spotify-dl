use std::{env, process::exit};
use tokio;

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

#[tokio::main]
async fn main() {
    let mut args = env::args();
    args.next();

    let username = args.next().unwrap();
    let password = args.next().unwrap();
    let credentials = Credentials::with_password(&username, &password);

    let session_config = SessionConfig::default();

    let mut player_config = PlayerConfig::default();
    player_config.passthrough = true;

    let audio_format = AudioFormat::default();
    let track = SpotifyId::from_base62(&args.next().unwrap()).unwrap();
    let backend = audio_backend::find(Some("pipe".to_string())).unwrap();

    let (session, creds) = Session::connect(session_config, credentials, None, false).await.expect("error connecting");
    let (mut player, _) = Player::new(player_config, session, Box::new(NoOpVolume), move || {
        backend(None, audio_format)
    });

    player.load(track, true, 0);
    player.await_end_of_track().await;
}
