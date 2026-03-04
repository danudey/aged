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
}
