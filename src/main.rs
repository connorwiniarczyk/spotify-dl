#![allow(unused_imports, dead_code)]

mod record;
mod spotify;
use record::RecordSink;

use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::spotify_id::SpotifyId;
use librespot::core::spotify_id::SpotifyItemType;
use librespot::metadata::{
    Metadata,
    Track,
};

use sanitise_file_name::sanitise;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor};

use console::Style;
use console::Term;

use futures_executor::block_on;

use tokio;
use std::io::Write;
use std::env;
use std::path::Path;
use std::process::Command;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref terminal: Term = Term::stdout();
    pub static ref highlight: Style  = Style::new().cyan();
}

// pub static mut CREDS: Option<spotify::Credentials> = None;

fn enter_working_directory() {
    let home = std::env::home_dir().expect("could not find home dir");
    let workdir = home.join("Music/spotify-dl");
    std::fs::create_dir_all(&workdir).unwrap();
    std::env::set_current_dir(&workdir).unwrap();
}

pub fn print_session_header(session: &Session) {
    let version_major = env!("CARGO_PKG_VERSION_MAJOR");
    let version_minor = env!("CARGO_PKG_VERSION_MINOR");

    println!("Spotify-DL");
    println!("----------");
    println!("version: {}", format!("{}.{}", version_major, version_minor));
    println!("logged in as {}", highlight.apply_to(session.username()));
    println!("will download songs to {}", highlight.apply_to(std::env::current_dir().unwrap().display()));
    println!("paste spotify links below to download them, or type 'help' for more options");
    println!();
}

pub fn usage() {
    println!();
    println!("Commands");
    println!("--------");
    println!("download <link>      - download the contents of the Playlist, Album or Track");
    println!("export <path> <link> - download the contents of <link> and then copy them to <path>");
    println!("help                 - print this message");
    println!();
}

async fn download(id: SpotifyId, session: &Session, export_path: Option<&Path>) -> Result<(), ()> {
    let checkmark = console::style("✔".to_string()).for_stdout().green();
    let error     = console::style("✘".to_string()).for_stdout().red();
    let dot       = console::style("·".to_string()).for_stdout().yellow().bright();

    if let Some(p) = export_path {
        std::fs::create_dir_all(p).expect("failed to create export directory");
    }

    let tracks = spotify::get_tracks_to_download(id, session);

    let size = tracks.len();
	for (mut i, track_id) in tracks.into_iter().enumerate() {
    	i += 1;
    	let path = Path::new(&track_id.to_base62().unwrap()).with_extension("ogg");
    	if path.exists() {
        	println!("{} ({:02}/{:02}) {} : exists", checkmark, i, size, &track_id.to_base62().unwrap());
    	}

    	else {
        	println!("{} ({:02}/{:02}) {}", dot, i, size, &track_id.to_base62().unwrap());
    		let res = spotify::record_track(track_id, session.clone()).await;

    		match res {
        		Ok(name) => {
            		terminal.clear_last_lines(1).unwrap();
                	println!("{} ({:02}/{:02}) {} : {}", checkmark, i, size, &track_id.to_base62().unwrap(), name);
        		},

        		Err(message) => {
            		terminal.clear_last_lines(1).unwrap();
                	println!("{} ({:02}/{:02}) {} : {}", error, i, size, &track_id.to_base62().unwrap(), message);
        		},
    		}
    	}

    	if let Some(p) = export_path {
        	let metadata = Track::get(session, &track_id).await.unwrap();
        	let dest = p.join(sanitise(&metadata.name)).with_extension("ogg");
			if std::fs::copy(path, dest).is_err() {
    			println!("copy failed");
			}
    	}

	}

	println!();
	Ok(())
}

async fn handle_command(cmd: String, ctx: &Session) -> Result<(), ()>{
    let mut iter = cmd.split(" ");

    match iter.next().ok_or(())? {
        "" => return Ok(()),
        "h" | "?" | "help" => usage(),
        "q" | "quit" | "exit" => return Err(()),

        "logout" => {
            let _ = std::fs::remove_file("access_token.txt");
            return Err(());
        },

        "d" | "download" => {
            let arg = iter.next().expect("no arg after download command");
            let id = spotify::parse_link(arg).expect("invalid spoitfy link");
            let result = download(id, ctx, None).await;
            return result
        },

        "e" | "export" => {
            let path = iter.next().expect("export requires 2 arguments");
            let link = iter.next().expect("export requires 2 arguments");
            let id = spotify::parse_link(link).expect("invalid spoitfy link");
            download(id, ctx, Some(Path::new(&path))).await?;
        },

        arg => {
            let id = spotify::parse_link(arg)?;
            return download(id, ctx, None).await;
        }
    }

    Ok(())
}

fn test_ffmpeg() -> Result<(), ()> {
    let output = Command::new("ffmpeg").arg("-version").output().map_err(|_| ())?;

    if output.status.code() == Some(0) {
        Ok(())
    } else {
        Err(())
    }
}

#[tokio::main]
async fn main() {
	terminal.clear_screen().unwrap();
	enter_working_directory();

    if test_ffmpeg().is_err() {
        println!("ffmpeg is not installed properly, please fix that by installing it from here:");
        println!("https://ffmpeg.org/download.html");
        println!();
        return;
    }

    let session = Session::new(SessionConfig::default(), None);
    let _creds = spotify::connect(&session).expect("failed to log in");

    print_session_header(&session);

	let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                if handle_command(line, &session).await.is_err() {
                    break;
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}
