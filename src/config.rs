use inquire::{Text, Password, Confirm};
use std::fs::File;
use std::io::{BufRead, BufReader};

use std::io::Error as IOError;
use inquire::error::InquireError;

use lazy_static::lazy_static;
use regex::Regex;

/// An Error type for generating the Config type
pub struct Error {
    message: String, 
}

impl From<IOError> for Error {
    fn from(value: IOError) -> Self {
        "io error".into()
    }
}

impl From<InquireError> for Error {
    fn from(value: InquireError) -> Self {
        todo!();
    }
}

impl From<&'static str> for Error {
    fn from(message: &'static str) -> Self {
        Self { message: message.to_string() }
    }
}

fn format_from_url(original: & str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"open\.spotify\.com/(?P<type>.+?)/(?P<id>.+?)$"#).unwrap();
    }

    let captures = RE.captures(original).unwrap();

    let audio_type = captures.name("type").unwrap().as_str();
    let id = captures.name("id").unwrap().as_str();

    format!("spotify:{}:{}", audio_type, id)
}

#[derive(Debug)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub uri: String,
}

impl Config {
    pub fn generate() -> Self {
        let mut output = ConfigBuilder::new();

        let mut args = std::env::args();
        args.next();
        let url = args.next();
        output.uri = url.map(|x| format_from_url(&x));

        output.read_creds_file("./creds.txt");
        output.prompt_user();

        output.try_into().unwrap()

    }
}

impl TryFrom<ConfigBuilder> for Config {
    type Error = &'static str;
    fn try_from(value: ConfigBuilder) -> Result<Self, Self::Error> {
        let out = Self {
            username: value.username.ok_or("no username")?,
            password: value.password.ok_or("no password")?,
            uri: value.uri.ok_or("uri")?,
        };

        Ok(out)
    }
}

#[derive(Debug)]
pub struct ConfigBuilder {
    pub username: Option<String>,
    pub password: Option<String>,
    pub uri: Option<String>,
}


impl ConfigBuilder { 
    pub fn new() -> Self {
        format_from_url("open.spotify.com/abcd/efg");
        Self {username: None, password: None, uri: None}
    }

    // pub fn generate() -> Self {
    //     let mut output = Self::new();

    //     let mut args = std::env::args();
    //     args.next();
    //     let url = args.next();
    //     output.uri = url.map(|x| format_from_url(&x));

    //     output.read_creds_file("./creds.txt");
    //     output.prompt_user();

    //     return output;
    // }

    pub fn read_creds_file(&mut self, path: &str) -> Result<(), Error> {
        let file = File::open("creds.txt")?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        /// local function to parses a line of text with an equals sign into a key value pair
        fn get_key_value(line: String) -> Result<(String, String), &'static str> {
            let mut split = line.split("=");
            let key = split.next().ok_or("")?;
            let value = split.next().ok_or("")?;

            Ok((key.to_string(), value.to_string()))

        }

        for line in lines {
            let (key, value) = get_key_value(line?)?; 
            match key.as_str() {
                "username" => self.username = Some(value),
                "password" => self.password = Some(value),
                _ => return Err("invalid key".into()),
            }
        }

        Ok(())
    }

    fn save_creds(&self) -> Result<(), Error> {
        Ok(())
        // todo!();
    }

    pub fn prompt_user(&mut self) -> Result<(), Error> {

        let mut prompt_save = false;

        if self.username == None {
            let username = Text::new("Spotify Username:").prompt()?;
            self.username = Some(username);
            prompt_save = true;
        }

        if self.password == None {
            let password = Password::new("Spotify Password:").prompt()?;
            self.password = Some(password);
            prompt_save = true;
        }

        if prompt_save {
            let save = Confirm::new("Save Credentials?")
                .with_default(true)
                .with_help_message("Save these credentials to a local file so you don't have to type them again")
                .prompt()?;

            if save { self.save_creds(); }
        }

        if self.uri == None {
            let url = Text::new("Playlist:").prompt()?;
            self.uri = Some(format_from_url(&url));
        }

        Ok(())
    }
}
