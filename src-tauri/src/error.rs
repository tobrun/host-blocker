use serde::Serialize;

/// Application error, serialised for the UI as `{ kind, message }`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// User-facing validation or usage error; safe to show verbatim.
    #[error("{0}")]
    User(String),
    /// `/etc/hosts` could not be parsed or merged.
    #[error("could not update the hosts file: {0}")]
    HostsFile(String),
    /// Network problem while fetching a baseline list.
    #[error("network error: {0}")]
    Network(String),
    /// The user dismissed the privilege prompt.
    #[error("the administrator prompt was cancelled")]
    Cancelled,
    /// A privileged write failed for a reason other than cancellation.
    #[error("could not write /etc/hosts: {0}")]
    Privileged(String),
    #[error("input/output error: {0}")]
    Io(#[from] std::io::Error),
    #[error("data file error: {0}")]
    Data(String),
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Data(e.to_string())
    }
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::User(_) => "user",
            AppError::HostsFile(_) => "hosts_file",
            AppError::Network(_) => "network",
            AppError::Cancelled => "cancelled",
            AppError::Privileged(_) => "privileged",
            AppError::Io(_) => "io",
            AppError::Data(_) => "data",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
