use serde::{Deserialize, Serialize};

/// What HostBlocker currently writes into `/etc/hosts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockingMode {
    /// Baseline and custom entries are both written.
    Active,
    /// Baseline is written, the user's own sites are not.
    CustomPaused,
    /// Nothing is written; the managed section stays empty.
    AllPaused,
}

impl BlockingMode {
    pub fn writes_baseline(self) -> bool {
        !matches!(self, BlockingMode::AllPaused)
    }

    pub fn writes_custom(self) -> bool {
        matches!(self, BlockingMode::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_writes_matrix() {
        assert!(BlockingMode::Active.writes_baseline());
        assert!(BlockingMode::Active.writes_custom());
        assert!(BlockingMode::CustomPaused.writes_baseline());
        assert!(!BlockingMode::CustomPaused.writes_custom());
        assert!(!BlockingMode::AllPaused.writes_baseline());
        assert!(!BlockingMode::AllPaused.writes_custom());
    }

    #[test]
    fn serde_uses_camel_case() {
        assert_eq!(
            serde_json::to_string(&BlockingMode::CustomPaused).unwrap(),
            "\"customPaused\""
        );
        let m: BlockingMode = serde_json::from_str("\"allPaused\"").unwrap();
        assert_eq!(m, BlockingMode::AllPaused);
    }
}
