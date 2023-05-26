use thiserror::Error;

#[derive(Error, Debug)]
pub enum FireflyError {
    #[error("firefly secret is invalid")]
    InvalidSecret,

    #[error("http request failed with {0}")]
    HTTP(#[from] reqwest::Error),

    #[error("failed with: {0}")]
    Misc(String),
}
