use chrono::{Datelike, NaiveDate, Utc};

/// Calculate age in whole years from a birthdate to today.
pub fn calculate_age(birthdate: NaiveDate) -> u32 {
    let today = Utc::now().date_naive();
    calculate_age_on(birthdate, today)
}

/// Calculate age in whole years from a birthdate to a specific date.
pub fn calculate_age_on(birthdate: NaiveDate, on: NaiveDate) -> u32 {
    let mut age = on.year() - birthdate.year();

    // If we haven't reached the birthday this year, subtract one
    if (on.month(), on.day()) < (birthdate.month(), birthdate.day()) {
        age -= 1;
    }

    age.max(0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_exact_birthday() {
        let birth = NaiveDate::from_ymd_opt(1990, 3, 15).unwrap();
        let on = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        assert_eq!(calculate_age_on(birth, on), 35);
    }

    #[test]
    fn age_before_birthday() {
        let birth = NaiveDate::from_ymd_opt(1990, 6, 15).unwrap();
        let on = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
        assert_eq!(calculate_age_on(birth, on), 34);
    }

    #[test]
    fn age_after_birthday() {
        let birth = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        let on = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        assert_eq!(calculate_age_on(birth, on), 35);
    }

    #[test]
    fn age_child() {
        let birth = NaiveDate::from_ymd_opt(2015, 7, 20).unwrap();
        let on = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
        assert_eq!(calculate_age_on(birth, on), 9);
    }

    #[test]
    fn age_zero() {
        let birth = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let on = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        assert_eq!(calculate_age_on(birth, on), 0);
    }

    #[test]
    fn calculate_age_returns_non_negative() {
        // A birthdate far in the past should produce a reasonable age
        let birth = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let age = calculate_age(birth);
        assert!(age >= 125);
    }

    #[test]
    fn calculate_age_recent_birth() {
        // A very recent birthdate should be 0 or 1
        let birth = Utc::now().date_naive();
        let age = calculate_age(birth);
        assert_eq!(age, 0);
    }

    #[test]
    fn age_same_month_earlier_day() {
        let birth = NaiveDate::from_ymd_opt(2000, 6, 20).unwrap();
        let on = NaiveDate::from_ymd_opt(2025, 6, 10).unwrap();
        assert_eq!(calculate_age_on(birth, on), 24);
    }

    #[test]
    fn age_same_month_later_day() {
        let birth = NaiveDate::from_ymd_opt(2000, 6, 10).unwrap();
        let on = NaiveDate::from_ymd_opt(2025, 6, 20).unwrap();
        assert_eq!(calculate_age_on(birth, on), 25);
    }

    #[test]
    fn age_leap_day_birthday() {
        let birth = NaiveDate::from_ymd_opt(2000, 2, 29).unwrap();
        // On March 1 of a non-leap year
        let on = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
        assert_eq!(calculate_age_on(birth, on), 25);
        // On Feb 28 of a non-leap year (hasn't reached birthday yet)
        let on = NaiveDate::from_ymd_opt(2025, 2, 28).unwrap();
        assert_eq!(calculate_age_on(birth, on), 24);
    }
}
