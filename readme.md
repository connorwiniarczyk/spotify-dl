## spotify-dl

This is a cli tool for downloading Spotify tracks, albums, and playlists onto
a local disk using the [librespot](https://github.com/librespot-org/librespot)
library.

### Installation

Installing spotify-dl requires [ffmpeg](https://ffmpeg.org/) and the
[rust toolchain](https://www.rust-lang.org/tools/install).

```bash
cargo install --git https://github.com/connorwiniarczyk/spotify-dl
```

or

```bash
git clone https://github.com/connorwiniarczyk/spotify-dl
cd spotify-dl
cargo install --path .
```

### Usage

You must have a premium Spotify account to use this script. When you run it
for the first time it will prompt you for your username and password. It will
also give you the option to save these to a local file called `spotify-dl.conf`
so you won't have to type these again each time you run the tool.

```bash
spotify-dl https://open.spotify.com/album/6IGDCUkBJ3LEUoAcoTD46u
```

This will download each track from the album Yesterday's Tomorrow into a new
folder called Yesterday's Tomorrow. Files are automatically converted to the 
[FLAC](https://xiph.org/flac/) codec, but this can be changed by modifying the
source and recompiling.


```rust
async fn download_track(track: Track, track_id: SpotifyId, config: &Config, session: &Session) {
	// modify the .flac in this line (main.rs:72) to any file format understood by ffmpeg
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
```
