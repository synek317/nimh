use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::civil_date::CivilDate;

const SECONDS_PER_DAY: u64 = 86_400;

/// A moment in time, displayed in UTC with millisecond precision, as in RFC 3339:
/// `2026-11-11T12:41:03.123Z`.
///
/// UTC needs no time zone database, means the same on every machine and sorts as text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Timestamp {
    since_epoch: Duration,
}

impl Timestamp {
    /// A clock set to before 1970 is beyond what a log line can fix. It reads as 1970.
    pub(crate) fn now() -> Self {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self::from_duration_since_epoch(since_epoch)
    }

    pub(crate) const fn from_duration_since_epoch(since_epoch: Duration) -> Self {
        Self { since_epoch }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.since_epoch.as_secs();
        let date = CivilDate::from_days_since_epoch(seconds / SECONDS_PER_DAY);
        let second_of_day = seconds % SECONDS_PER_DAY;
        write!(
            f,
            "{date}T{:02}:{:02}:{:02}.{:03}Z",
            second_of_day / 3600,
            second_of_day / 60 % 60,
            second_of_day % 60,
            self.since_epoch.subsec_millis(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(0, "1970-01-01T00:00:00.000Z"; "the epoch")]
    #[test_case(999, "1970-01-01T00:00:00.999Z"; "milliseconds")]
    #[test_case(86_399_999, "1970-01-01T23:59:59.999Z"; "last millisecond of a day")]
    #[test_case(86_400_000, "1970-01-02T00:00:00.000Z"; "first millisecond of a day")]
    #[test_case(951_782_400_000, "2000-02-29T00:00:00.000Z"; "leap day")]
    #[test_case(1_794_400_863_123, "2026-11-11T12:41:03.123Z"; "an ordinary moment")]
    fn is_displayed_as_rfc_3339_in_utc(milliseconds_since_epoch: u64, expected: &str) {
        let timestamp =
            Timestamp::from_duration_since_epoch(Duration::from_millis(milliseconds_since_epoch));

        assert_eq!(timestamp.to_string(), expected);
    }

    #[test]
    fn digits_below_a_millisecond_are_cut_off() {
        let timestamp = Timestamp::from_duration_since_epoch(Duration::from_nanos(1_999_999));

        assert_eq!(timestamp.to_string(), "1970-01-01T00:00:00.001Z");
    }

    #[test]
    fn now_is_after_this_test_was_written() {
        let written = Timestamp::from_duration_since_epoch(Duration::from_secs(1_794_400_863));

        assert!(Timestamp::now().since_epoch > written.since_epoch);
    }
}
