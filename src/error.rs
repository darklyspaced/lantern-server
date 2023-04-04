use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum LanternError {
    InvalidSessionID, // cannot auth due to invalid session ID
    SchoolCode,
    FireflyAPI,           // something went wrong interacting with Firefly
    Misc(Box<dyn Error>), // anything from a database to a dotenvy error (third party errors essentially)
}

impl Error for LanternError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Misc(e) => Some(&**e),
            _ => None,
        }
    }
}

impl fmt::Display for LanternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?})", {
            match self {
                Self::InvalidSessionID => {
                    String::from("could not auth with firefly; invalid session id")
                }
                Self::FireflyAPI => String::from("something went wrong interacting with firefly"),
                Self::SchoolCode => String::from("incorrect school code provided"),
                Self::Misc(e) => e.to_string(),
            }
        })
    }
}

impl From<url::ParseError> for LanternError {
    fn from(error: url::ParseError) -> LanternError {
        LanternError::Misc(Box::new(error))
    }
}
impl From<reqwest::Error> for LanternError {
    fn from(error: reqwest::Error) -> LanternError {
        LanternError::Misc(Box::new(error))
    }
}
