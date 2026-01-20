use chrono::{DateTime, Duration, Utc};

use crate::models::ReviewRating;

/// SM-2 Algorithm Implementation
///
/// The SM-2 algorithm calculates the optimal review interval based on:
/// - Current interval (days since last review)
/// - Ease factor (difficulty multiplier, starts at 2.5)
/// - Number of consecutive correct reviews (repetitions)
/// - User's rating of recall quality (0-3)
///
/// Returns: (new_interval, new_ease_factor, new_repetitions, next_review_date)
pub fn calculate_sm2(
    current_interval: i32,
    current_ease_factor: f64,
    current_repetitions: i32,
    rating: ReviewRating,
) -> (i32, f64, i32, DateTime<Utc>) {
    let rating_value = rating as i32;

    // Calculate new ease factor
    // EF' = EF + (0.1 - (3 - q) * (0.08 + (3 - q) * 0.02))
    // where q is the rating (0-3)
    let new_ease_factor = if rating_value >= 2 {
        let q = rating_value as f64;
        let ef_change = 0.1 - (3.0 - q) * (0.08 + (3.0 - q) * 0.02);
        (current_ease_factor + ef_change).max(1.3)
    } else {
        // For ratings < 2, decrease ease factor but keep above 1.3
        (current_ease_factor - 0.2).max(1.3)
    };

    let (new_interval, new_repetitions) = match rating {
        ReviewRating::Again => {
            // Complete failure: reset to beginning
            (1, 0)
        }
        ReviewRating::Hard => {
            // Struggled but recalled: increase interval slightly
            if current_repetitions == 0 {
                (1, 1)
            } else if current_repetitions == 1 {
                (3, current_repetitions + 1)
            } else {
                let interval = ((current_interval as f64) * new_ease_factor * 0.8) as i32;
                (interval.max(1), current_repetitions + 1)
            }
        }
        ReviewRating::Good => {
            // Normal correct response
            if current_repetitions == 0 {
                (1, 1)
            } else if current_repetitions == 1 {
                (6, 2)
            } else {
                let interval = ((current_interval as f64) * new_ease_factor) as i32;
                (interval, current_repetitions + 1)
            }
        }
        ReviewRating::Easy => {
            // Perfect response: longer interval
            if current_repetitions == 0 {
                (4, 1)
            } else if current_repetitions == 1 {
                (10, 2)
            } else {
                let interval = ((current_interval as f64) * new_ease_factor * 1.3) as i32;
                (interval, current_repetitions + 1)
            }
        }
    };

    let next_review = Utc::now() + Duration::days(new_interval as i64);

    (new_interval, new_ease_factor, new_repetitions, next_review)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_review_good() {
        let (interval, ease, reps, _) = calculate_sm2(0, 2.5, 0, ReviewRating::Good);
        assert_eq!(interval, 1);
        assert_eq!(reps, 1);
        assert!((ease - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_second_review_good() {
        let (interval, ease, reps, _) = calculate_sm2(1, 2.5, 1, ReviewRating::Good);
        assert_eq!(interval, 6);
        assert_eq!(reps, 2);
    }

    #[test]
    fn test_third_review_good() {
        let (interval, _ease, reps, _) = calculate_sm2(6, 2.5, 2, ReviewRating::Good);
        assert_eq!(interval, 15); // 6 * 2.5 = 15
        assert_eq!(reps, 3);
    }

    #[test]
    fn test_review_again_resets() {
        let (interval, ease, reps, _) = calculate_sm2(15, 2.5, 3, ReviewRating::Again);
        assert_eq!(interval, 1);
        assert_eq!(reps, 0);
        assert!((ease - 2.3).abs() < 0.01); // Ease decreased
    }

    #[test]
    fn test_ease_factor_minimum() {
        // After many "again" ratings, ease factor should not go below 1.3
        let mut ease = 2.5;
        for _ in 0..10 {
            let (_, new_ease, _, _) = calculate_sm2(1, ease, 1, ReviewRating::Again);
            ease = new_ease;
        }
        assert!(ease >= 1.3);
    }

    #[test]
    fn test_easy_longer_intervals() {
        let (easy_interval, _, _, _) = calculate_sm2(6, 2.5, 2, ReviewRating::Easy);
        let (good_interval, _, _, _) = calculate_sm2(6, 2.5, 2, ReviewRating::Good);
        assert!(easy_interval > good_interval);
    }
}
