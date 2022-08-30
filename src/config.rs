use inquire::{Text, Password, Confirm};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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

    pub audio_type: String,
    pub id: String,
}

impl Config {
    pub fn generate() -> Self {
        let mut output = ConfigBuilder::new();

        let mut args = std::env::args();
        args.next();
        let url = args.next();

        if let Some(url) = url {
            output.parse_url(&url);
        }

        output.read_creds_file("./spotify-dl.conf");
        output.prompt_user();

        output.try_into().unwrap()
    }


    pub fn uri(&self) -> String {
        format!("spotify:{}:{}", self.audio_type, self.id)
    }
}

impl TryFrom<ConfigBuilder> for Config {
    type Error = &'static str;
    fn try_from(value: ConfigBuilder) -> Result<Self, Self::Error> {
        let out = Self {
            username: value.username.ok_or("no username")?,
            password: value.password.ok_or("no password")?,
            audio_type: value.audio_type.ok_or("no audio type")?,
            id: value.id.ok_or("no id")?,
        };

        Ok(out)
    }
}

#[derive(Debug)]
pub struct ConfigBuilder {
    pub username: Option<String>,
    pub password: Option<String>,
    pub audio_type: Option<String>,
    pub id: Option<String>,
}


impl ConfigBuilder { 
    pub fn new() -> Self {
        Self {username: None, password: None, audio_type: None, id: None}
    }

    pub fn parse_url(&mut self, url: &str) -> Result<(), Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"open\.spotify\.com/(?P<type>.+?)/(?P<id>.+?)$"#).unwrap();
        }

        let captures = RE.captures(url)
            .ok_or("could not parse url")?;

        let audio_type = captures.name("type")
            .ok_or("could not parse url")?
            .as_str();

        let id = captures.name("id")
            .ok_or("could not parse url")?
            .as_str();

        self.audio_type = Some(audio_type.into());
        self.id = Some(id.into());

        Ok(())
    }

    pub fn read_creds_file(&mut self, path: &str) -> Result<(), Error> {
        let file = File::open("spotify-dl.conf")?;
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
        let mut file = File::create("spotify-dl.conf")?;

        match (self.username.as_ref(), self.password.as_ref()) {
            (Some(u), Some(p)) => {
                let username_line = format!("username={}\n", u);
                let password_line = format!("password={}\n", p);
                file.write_all(&username_line.as_bytes())?;
                file.write_all(&password_line.as_bytes())?;
            },
            _ => (),
        }

        Ok(())
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

        if self.id == None {
            let url = Text::new("Playlist:").prompt()?;
            self.parse_url(&url)?;
        }

        Ok(())
    }
}
