


use librespot::playback::audio_backend::{ Sink, SinkError, SinkResult};


// use librespot::playback::config::AudioFormat;
use librespot::playback::decoder::AudioPacket;
use librespot::playback::convert::Converter;

use librespot::metadata::{
    // Metadata,
    // Playlist,
    Track,
    // Album,
    // Artist,
};

use std::process::Stdio;
use std::process::Command;
use std::io::Write;

pub struct RecordSink {
    process: std::process::Child,
    stream:  std::process::ChildStdin,
}

impl RecordSink {

    fn get_artists_string(track: &Track) -> String {
		let mut artists = String::new();
        let mut i = 0;
        while i < track.artists.len() {
            artists.push_str(track.artists[i].name.as_str());
            i += 1;
            if i < track.artists.len() {
                artists.push_str(", ");
            }
        }

        return artists
    }

    pub fn create(track: Track) -> Box<dyn Sink> {
        let path = format!("spotify-dl/{}.ogg", track.id.to_base62().unwrap());
        let artists = Self::get_artists_string(&track);

        let mut process = Command::new("ffmpeg")
            .arg("-f").arg("ogg")
            .arg("-i").arg("pipe:")
            .arg("-metadata").arg(&format!("title={}", track.name))
            .arg("-metadata").arg(&format!("album={}", track.album.name))
            .arg("-metadata").arg(&format!("artist={}", artists))
            .arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to open ffmpeg");

        let stream = process.stdin.take().unwrap();
        let output = Self { process, stream };
        Box::new(output)
    }
}

impl Sink for RecordSink {
    fn start(&mut self) -> SinkResult<()> {
        Ok(())
    }

    fn stop(&mut self) -> SinkResult<()> {
        // if let Err(_) = self.process.wait() {
        //     return Err(SinkError::OnWrite("failed to await ffmpeg".to_owned()));
        // }

        Ok(())
    }

    fn write(&mut self, packet: AudioPacket, _conveter: &mut Converter) -> SinkResult<()> {
        let AudioPacket::Raw(bytes) = packet else {
            panic!("found non-raw samples");
        };

        if self.stream.write(&bytes).is_err() {
            return Err(SinkError::OnWrite("failed to write bytes to ffmpeg".to_owned()));
        };

        Ok(())
    }
}
