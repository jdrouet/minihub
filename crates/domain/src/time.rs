//! Time and timestamp helpers.

use chrono::{DateTime, Utc};

/// UTC timestamp used for `last_changed`, `last_updated`, event times, etc.
pub type Timestamp = DateTime<Utc>;

/// Return the current UTC time.
#[must_use]
pub fn now() -> Timestamp {
    Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_current_utc_time() {
        let before = Utc::now();
        let ts = now();
        let after = Utc::now();
        assert!(ts >= before);
        assert!(ts <= after);
    }
}
