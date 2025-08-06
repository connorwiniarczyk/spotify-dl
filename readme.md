# Spotify-DL

Spotify-DL is a command line tool that downloads songs from Spotify.
It uses
[librespot](https://github.com/librespot-org/librespot)
to implement a custom Spotify client that "plays" the tracks you select
into local ogg/vorbis files instead of to your audio hardware.
To use it, simply log into your premium account and paste a link to a track, album, or playlist,
Spotify-DL will download all of the tracks pointed to by the link to a folder located in
`$HOME/Music/spotify-dl`.

Files are named after their unique spotify id, rather than their title, which makes it
easier to detect and skip duplicates, but they are downloaded with metadata tags
for title, album, and artist, which makes them identifiable in most music software.
Once downloaded, tracks can be copied into a separate folder with human readable file
names by using the `export` command.

[![asciicast](https://asciinema.org/a/731843.svg)](https://asciinema.org/a/731843)

## Installation

Spotify-DL can be installed using cargo like so:

```bash
cargo install --git https://github.com/connorwiniarczyk/spotify-dl
```

Of built from source like so:

```bash
git clone https://github.com/connorwiniarczyk/spotify-dl
cd spotify-dl
cargo build --release
```

This requires the rust toolchain to be installed. Which can be done by
following the instructions here:
https://rustup.rs/

Running Spotify-DL requires the `ffmpeg` binary, which can be installed here:
https://ffmpeg.org/download.html
