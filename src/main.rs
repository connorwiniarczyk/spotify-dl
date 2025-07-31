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

struct AppState {
    session: Session,
    terminal: Term,
    highlight: Style,
}

impl AppState {
    pub fn new(terminal: Term) -> Self {
        terminal.clear_screen().unwrap();

		// init working directory
        let home = std::env::home_dir().expect("could not find home dir");
        let workdir = home.join("Music/spotify-dl");
    	std::fs::create_dir_all(&workdir).unwrap();
        std::env::set_current_dir(&workdir).unwrap();

        let session = Session::new(SessionConfig::default(), None);
        let highlight = Style::new().cyan();
        Self { session, terminal, highlight }
    }

    pub fn header(&self) {
        println!("Spotify-DL");
        println!("----------");
        println!("logged in as {}", self.highlight.apply_to(self.session.username()));
        println!("will download songs to {}", self.highlight.apply_to(std::env::current_dir().unwrap().display()));
        println!("paste spotify links below to download them, or type 'help' for more options");
        println!();
    }

    pub fn usage(&self) {
        println!();
        println!("Commands");
        println!("--------");
        println!("download <link>      - download the contents of the Playlist, Album or Track");
        println!("export <path> <link> - download the contents of <link> and then copy them to <path>");
        println!("help                 - print this message");
        println!();
    }

    pub fn login(&self) -> Result<(), ()> {
        if spotify::try_automatic_login(&self.session).is_err() {
            println!("to log in, open the link below and click agree");
            spotify::try_manual_login(&self.session).expect("failed to login");
            println!();
            Ok(())
        } else {
            Ok(())
        }
    }
}

fn download(id: SpotifyId, ctx: &mut AppState, export_path: Option<&Path>) -> Result<(), ()> {
    let checkmark = console::style("✔".to_string()).for_stdout().green();
    let error     = console::style("✘".to_string()).for_stdout().red();
    let dot       = console::style("·".to_string()).for_stdout().yellow().bright();

    if let Some(p) = export_path {
        std::fs::create_dir_all(p).expect("failed to create export directory");
    }

    let tracks = spotify::get_tracks_to_download(id, &ctx.session);
    let size = tracks.len();

	for (mut i, track_id) in tracks.into_iter().enumerate() {
    	i += 1;
    	let path = Path::new(&track_id.to_base62().unwrap()).with_extension("ogg");
    	if path.exists() {
        	println!("{} ({:02}/{:02}) {} : exists", checkmark, i, size, &track_id.to_base62().unwrap());
    	}

    	else {
        	println!("{} ({:02}/{:02}) {}", dot, i, size, &track_id.to_base62().unwrap());
    		let res = block_on(spotify::record_track(track_id, ctx.session.clone()));

    		match res {
        		Ok(name) => {
            		ctx.terminal.clear_last_lines(1).unwrap();
                	println!("{} ({:02}/{:02}) {} : {}", checkmark, i, size, &track_id.to_base62().unwrap(), name);
        		},

        		Err(message) => {
            		ctx.terminal.clear_last_lines(1).unwrap();
                	println!("{} ({:02}/{:02}) {} : {}", error, i, size, &track_id.to_base62().unwrap(), message);
        		},
    		}
    	}

    	if let Some(p) = export_path {
        	let metadata = block_on(Track::get(&ctx.session, &track_id)).unwrap();
        	let dest = p.join(sanitise(&metadata.name)).with_extension("ogg");
			if std::fs::copy(path, dest).is_err() {
    			// println!("copy failed");
			}
    	}
	}

	println!();
	Ok(())
}

fn handle_command(cmd: String, ctx: &mut AppState) -> Result<(), ()>{
    let mut iter = cmd.split(" ");

    match iter.next().ok_or(())? {
        "" => return Ok(()),
        "h" | "?" | "help" => ctx.usage(),

        "q" | "quit" | "exit" => return Err(()),

        "d" | "download" => {
            let arg = iter.next().expect("no arg after download command");
            let id = spotify::parse_link(arg).expect("invalid spoitfy link");
            return download(id, ctx, None)
        },

        "e" | "export" => {
            let path = iter.next().expect("export requires 2 arguments");
            let link = iter.next().expect("export requires 2 arguments");
            let id = spotify::parse_link(link).expect("invalid spoitfy link");
            download(id, ctx, Some(Path::new(&path)))?;
        },

        arg => {
            let id = spotify::parse_link(arg)?;
            return download(id, ctx, None);
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
    let mut ctx = AppState::new(Term::stdout());

    if test_ffmpeg().is_err() {
        println!("ffmpeg is not installed properly, please fix that by installing it from here:");
        println!("https://ffmpeg.org/download.html");
        println!();
        return;
    }

    ctx.login().expect("failed to log in");
    ctx.header();

	let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                if handle_command(line, &mut ctx).is_err() {
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
