pub type Result<T> = core::result::Result<T, Error>;

use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    //
    // 1st party
    //
    NotImplemented,
    QuitSignalFailure,
    ConnectionFailure,
    Empty,
    NotConnected,
    MissingSender,
    EventSendFailure,
    EventDataMissing,
    EventNameMissing,
    EventTypeNotImplemented {
        name: String,
    },
    UrlNotInitialized,
    InvalidEndpoint,
    HttpFailure,
    ReadFailure,
    EndpointMissing,
    Eof,
    ConnectionStateFailure,
    NotFound,
    FunctionCallFailure {
        error: String,
    },

    //
    // 2nd party
    //
    #[from]
    Utf8(std::str::Utf8Error),
    #[from]
    Utf8Error(std::string::FromUtf8Error),
    #[from]
    Io(std::io::Error),

    //
    // 3rd party
    //
    #[from]
    JoinError(tokio::task::JoinError),
    #[from]
    Rstaples(rstaples::error::Error),
    #[from]
    Reqwest(reqwest::Error),
    #[from]
    HeaderError(reqwest::header::InvalidHeaderName),
    #[from]
    HeaderValue(reqwest::header::InvalidHeaderValue),
    #[from]
    Serialization(serde_json::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
