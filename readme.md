# Spotify-DL

Spotify-DL is a command line tool that downloads songs from Spotify.
It uses
[librespot](https://github.com/librespot-org/librespot)
to implement a custom Spotify client that "plays" the tracks you select
into a local ogg/vorbis file instead of to your speakers.

[![asciicast](https://asciinema.org/a/731843.svg)](https://asciinema.org/a/731843)

## Installation

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

## Usage

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
source and recompiling. (main.rs, line 72)
