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
source and recompiling. (main.rs, line 72)

### How It Works

As far as I know, nothing in the librespot library allows you to directly
download audio from Spotify, but it does give you the option to use the
process's stdout as a playback device, allowing you to pipe the audio to
another program or save it directly to a file. It also gives you the option to
"play" the audio directly as ogg packets instead of decoding them first.

This tool includes a binary called `spotify-direct-play` which is a very simple
spotify client that takes a username, password, and track id as arguments and
"plays" the entirety of the track to stdout before exiting. spotify-dl spawns
an instance of spotify-direct-play for every track it is given, and directs its
output into an instance of ffmpeg which transcodes the audio into FLAC and tags
it with the appropriate metadata before saving it to a local file.
