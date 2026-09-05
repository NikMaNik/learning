/// SM-2 Spaced Repetition Algorithm
///
/// Quality: 0-5
///   0 — complete blackout
///   1 — incorrect, remembered upon seeing answer
///   2 — incorrect, but easy to recall after seeing answer
///   3 — correct with serious difficulty
///   4 — correct with some hesitation
///   5 — perfect response
///
/// Returns: (new_interval_days, new_ease_factor, new_repetitions)
pub fn calculate(
    quality: u8,
    repetitions: i64,
    ease_factor: f64,
    interval: i64,
) -> (i64, f64, i64) {
    let q = quality.min(5) as f64;

    let (new_interval, new_reps) = if q < 3.0 {
        // Failed: reset
        (1, 0)
    } else {
        // Passed
        let new_reps = repetitions + 1;
        let new_interval = match new_reps {
            1 => 1,
            2 => 6,
            _ => ((interval as f64) * ease_factor).round() as i64,
        };
        (new_interval, new_reps)
    };

    // Update ease factor
    let new_ef = (ease_factor + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02))).max(1.3);

    (new_interval, new_ef, new_reps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failed_resets() {
        let (interval, ef, reps) = calculate(2, 5, 2.5, 30);
        assert_eq!(interval, 1);
        assert_eq!(reps, 0);
        assert!(ef >= 1.3);
    }

    #[test]
    fn test_first_success() {
        let (interval, _ef, reps) = calculate(4, 0, 2.5, 0);
        assert_eq!(interval, 1);
        assert_eq!(reps, 1);
    }

    #[test]
    fn test_second_success() {
        let (interval, _ef, reps) = calculate(4, 1, 2.5, 1);
        assert_eq!(interval, 6);
        assert_eq!(reps, 2);
    }

    #[test]
    fn test_subsequent_success() {
        let (interval, _ef, reps) = calculate(4, 2, 2.5, 6);
        assert_eq!(interval, 15);
        assert_eq!(reps, 3);
    }

    #[test]
    fn test_ef_never_below_1_3() {
        let (_, ef, _) = calculate(0, 10, 1.3, 100);
        assert!(ef >= 1.3);
    }

    #[test]
    fn test_quality_5_increases_ef() {
        let (_, ef, _) = calculate(5, 3, 2.5, 15);
        assert!(ef > 2.5);
    }
}
