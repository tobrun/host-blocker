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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_names_are_stable_strings() {
        // The frontend switches on these exact values; renaming one is a
        // breaking change.
        assert_eq!(AppError::User("u".into()).kind(), "user");
        assert_eq!(AppError::HostsFile("h".into()).kind(), "hosts_file");
        assert_eq!(AppError::Network("n".into()).kind(), "network");
        assert_eq!(AppError::Cancelled.kind(), "cancelled");
        assert_eq!(AppError::Privileged("p".into()).kind(), "privileged");
        assert_eq!(AppError::Io(std::io::Error::other("i")).kind(), "io");
        assert_eq!(AppError::Data("d".into()).kind(), "data");
    }

    #[test]
    fn serializes_as_kind_and_message() {
        let value = serde_json::to_value(AppError::Cancelled).unwrap();
        assert_eq!(value["kind"], "cancelled");
        assert!(value["message"].is_string());

        let value = serde_json::to_value(AppError::HostsFile("bad markers".into())).unwrap();
        assert_eq!(value["kind"], "hosts_file");
        assert_eq!(
            value["message"],
            "could not update the hosts file: bad markers"
        );
    }
}
