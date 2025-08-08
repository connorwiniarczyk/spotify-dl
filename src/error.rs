use crate::spotify::SpotifyId;

use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Empty,
    Exists(SpotifyId),
    EarlyPause,
    Unavailable(SpotifyId),
    InvalidLink(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Exists(id) => {
                write!(f, "{} already exists", id.to_base62().unwrap())?;
            },
            Self::Unavailable(id) => {
				write!(f, "the requested resource is unavailable {}", id.to_base62().unwrap())?;
            },
            Self::InvalidLink(ref link) => {
                f.write_str("tried to parse an invalid spotify link: ")?;
                f.write_str(link)?;
                f.write_str("\n")?;
            },
            _ => f.write_str("unspecified error")?,
        };

        Ok(())
    }
}


impl Error {
    pub fn invalid_link(input: &str) -> Self {
        Self::InvalidLink(input.to_owned())
    }
}

impl From<()> for Error {
	fn from(_: ()) -> Self {
    	Self::Empty
	}
}
