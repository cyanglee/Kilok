//! Billable hours calculation module
//!
//! Formula: raw_hours × 1.2, rounded up to nearest 0.5h

const MULTIPLIER: f64 = 1.2;
const ROUND_UNIT: f64 = 0.5; // hours

/// Calculate billable seconds from raw seconds
///
/// Applies the formula: raw_hours × 1.2, rounded up to nearest 0.5h
///
/// # Examples
///
/// ```
/// use claude_time_tracker::billable::calculate_billable_seconds;
///
/// // 10 minutes → 0.167h × 1.2 = 0.2h → ceil(0.2/0.5)*0.5 = 0.5h = 1800s
/// assert_eq!(calculate_billable_seconds(600), 1800);
///
/// // 2.7h → 3.24h → ceil(3.24/0.5)*0.5 = 3.5h = 12600s
/// assert_eq!(calculate_billable_seconds(9720), 12600);
/// ```
pub fn calculate_billable_seconds(raw_seconds: i64) -> i64 {
    if raw_seconds <= 0 {
        return 0;
    }
    let raw_hours = raw_seconds as f64 / 3600.0;
    let multiplied = raw_hours * MULTIPLIER;
    let rounded = (multiplied / ROUND_UNIT).ceil() * ROUND_UNIT;
    (rounded * 3600.0) as i64
}

/// Calculate billable hours from raw seconds
///
/// # Returns
/// Billable hours as f64, rounded to nearest 0.5h
pub fn calculate_billable_hours(raw_seconds: i64) -> f64 {
    calculate_billable_seconds(raw_seconds) as f64 / 3600.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_seconds() {
        assert_eq!(calculate_billable_seconds(0), 0);
        assert_eq!(calculate_billable_seconds(-100), 0);
    }

    #[test]
    fn test_small_time() {
        // 10 minutes = 600 seconds
        // 600/3600 = 0.167h × 1.2 = 0.2h → rounds up to 0.5h = 1800s
        assert_eq!(calculate_billable_seconds(600), 1800);
    }

    #[test]
    fn test_one_hour() {
        // 1 hour = 3600 seconds
        // 1h × 1.2 = 1.2h → rounds up to 1.5h = 5400s
        assert_eq!(calculate_billable_seconds(3600), 5400);
    }

    #[test]
    fn test_two_point_seven_hours() {
        // 2.7h = 9720 seconds
        // 2.7h × 1.2 = 3.24h → rounds up to 3.5h = 12600s
        assert_eq!(calculate_billable_seconds(9720), 12600);
    }

    #[test]
    fn test_exact_half_hour() {
        // 0.5h = 1800 seconds
        // 0.5h × 1.2 = 0.6h → rounds up to 1.0h = 3600s
        assert_eq!(calculate_billable_seconds(1800), 3600);
    }

    #[test]
    fn test_billable_hours() {
        assert_eq!(calculate_billable_hours(600), 0.5);
        assert_eq!(calculate_billable_hours(3600), 1.5);
        assert_eq!(calculate_billable_hours(9720), 3.5);
    }
}
