use std::fmt;

/// A date of the Gregorian calendar, displayed as `2026-11-11`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CivilDate {
    year: u64,
    month: u64,
    day: u64,
}

impl CivilDate {
    /// Finds the date that lies `days` after 1970-01-01.
    ///
    /// This is the `civil_from_days` algorithm by Howard Hinnant, described at
    /// <https://howardhinnant.github.io/date_algorithms.html>. Its trick is to count years
    /// from the 1st of March. The leap day then comes last in the year, every 400 years
    /// (an "era") have the same number of days, and what is left is plain division.
    pub(crate) fn from_days_since_epoch(days: u64) -> Self {
        const DAYS_PER_ERA: u64 = 146_097;
        const DAYS_FROM_0000_03_01_TO_EPOCH: u64 = 719_468;

        let days = days + DAYS_FROM_0000_03_01_TO_EPOCH;
        let era = days / DAYS_PER_ERA;
        let day_of_era = days % DAYS_PER_ERA;
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);

        // Months are counted from March here: 0 is March and 11 is February.
        let month_from_march = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
        let month = if month_from_march < 10 {
            month_from_march + 3
        } else {
            month_from_march - 9
        };

        // January and February belong to the calendar year that follows.
        let year = era * 400 + year_of_era + u64::from(month <= 2);
        Self { year, month, day }
    }
}

impl fmt::Display for CivilDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(0, "1970-01-01"; "the epoch")]
    #[test_case(30, "1970-01-31"; "last day of a month")]
    #[test_case(31, "1970-02-01"; "first day of a month")]
    #[test_case(58, "1970-02-28"; "end of February in a common year")]
    #[test_case(59, "1970-03-01"; "the day after it")]
    #[test_case(364, "1970-12-31"; "last day of a year")]
    #[test_case(365, "1971-01-01"; "first day of a year")]
    #[test_case(11_016, "2000-02-29"; "leap day in a year divisible by 400")]
    #[test_case(19_782, "2024-02-29"; "leap day")]
    #[test_case(19_783, "2024-03-01"; "the day after a leap day")]
    #[test_case(20_768, "2026-11-11"; "an ordinary day")]
    #[test_case(47_540, "2100-02-28"; "end of February in a year divisible by 100")]
    #[test_case(47_541, "2100-03-01"; "no leap day in a year divisible by 100")]
    fn finds_the_date_of_a_day_since_the_epoch(days: u64, expected: &str) {
        assert_eq!(CivilDate::from_days_since_epoch(days).to_string(), expected);
    }

    #[test]
    fn every_day_follows_the_previous_one() {
        let mut previous = CivilDate::from_days_since_epoch(0);

        // A little over 400 years, which is one full cycle of the calendar.
        for days in 1..150_000 {
            let date = CivilDate::from_days_since_epoch(days);
            let is_next_day = date.day == previous.day + 1 && date.month == previous.month;
            let is_next_month =
                date.day == 1 && date.month == previous.month + 1 && date.year == previous.year;
            let is_next_year = date.day == 1
                && date.month == 1
                && previous.month == 12
                && date.year == previous.year + 1;

            assert!(
                is_next_day || is_next_month || is_next_year,
                "{previous} -> {date}"
            );
            previous = date;
        }
    }
}
