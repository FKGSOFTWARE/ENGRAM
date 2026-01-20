use chrono::{DateTime, Duration, Utc};

use crate::models::ReviewRating;

/// FSRS (Free Spaced Repetition Scheduling) Algorithm Implementation
///
/// FSRS is a modern spaced repetition algorithm that uses:
/// - Stability (S): Days until recall probability drops to 90%
/// - Difficulty (D): Card difficulty from 1.0 to 10.0
/// - Retrievability (R): Current probability of successful recall
///
/// This implementation is based on FSRS-4.5 with optimized parameters.

/// FSRS model parameters (optimized defaults)
#[derive(Debug, Clone)]
pub struct FSRSParameters {
    /// Initial stability values for each rating [Again, Hard, Good, Easy]
    pub w: [f64; 17],
    /// Decay rate for retrievability
    pub decay: f64,
    /// Factor for retrievability formula
    pub factor: f64,
    /// Target retention rate (default 0.9 = 90%)
    pub request_retention: f64,
    /// Maximum interval in days
    pub maximum_interval: i32,
}

impl Default for FSRSParameters {
    fn default() -> Self {
        Self {
            // Optimized FSRS-4.5 weights
            w: [
                0.4,    // w0: initial stability for Again
                0.6,    // w1: initial stability for Hard
                2.4,    // w2: initial stability for Good
                5.8,    // w3: initial stability for Easy
                4.93,   // w4: difficulty weight
                0.94,   // w5: difficulty weight
                0.86,   // w6: difficulty weight
                0.01,   // w7: difficulty weight
                1.49,   // w8: stability after success weight
                0.14,   // w9: stability after success weight
                0.94,   // w10: stability after success weight
                2.18,   // w11: stability after failure weight
                0.05,   // w12: stability after failure weight
                0.34,   // w13: stability after failure weight
                1.26,   // w14: stability after failure weight
                0.29,   // w15: hard penalty
                2.61,   // w16: easy bonus
            ],
            decay: -0.5,
            factor: 19.0 / 81.0,
            request_retention: 0.9,
            maximum_interval: 36500, // 100 years
        }
    }
}

/// FSRS state for a card
#[derive(Debug, Clone)]
pub struct FSRSState {
    /// Stability: Days until 90% retention
    pub stability: f64,
    /// Difficulty: 1.0 (easiest) to 10.0 (hardest)
    pub difficulty: f64,
    /// Number of reviews (0 = new card)
    pub reps: i32,
    /// Number of lapses (times rated Again after learning)
    pub lapses: i32,
    /// Last review timestamp
    pub last_review: Option<DateTime<Utc>>,
}

impl Default for FSRSState {
    fn default() -> Self {
        Self {
            stability: 0.0,
            difficulty: 5.0,
            reps: 0,
            lapses: 0,
            last_review: None,
        }
    }
}

/// Calculate retrievability (probability of recall) given elapsed time
fn retrievability(elapsed_days: f64, stability: f64, params: &FSRSParameters) -> f64 {
    if stability <= 0.0 {
        return 0.0;
    }
    (1.0 + params.factor * elapsed_days / stability).powf(params.decay)
}

/// Calculate initial difficulty from first rating
fn init_difficulty(rating: ReviewRating, params: &FSRSParameters) -> f64 {
    let g = rating as i32 as f64;
    let d0 = params.w[4] - (g - 3.0) * params.w[5];
    d0.clamp(1.0, 10.0)
}

/// Calculate initial stability from first rating
fn init_stability(rating: ReviewRating, params: &FSRSParameters) -> f64 {
    params.w[rating as usize]
}

/// Calculate next difficulty after review
fn next_difficulty(d: f64, rating: ReviewRating, params: &FSRSParameters) -> f64 {
    let g = rating as i32 as f64;
    let d_new = d - params.w[6] * (g - 3.0);
    // Mean reversion towards initial difficulty
    let d_final = params.w[7] * init_difficulty(rating, params) + (1.0 - params.w[7]) * d_new;
    d_final.clamp(1.0, 10.0)
}

/// Calculate next stability after successful recall
fn next_stability_success(
    d: f64,
    s: f64,
    r: f64,
    rating: ReviewRating,
    params: &FSRSParameters,
) -> f64 {
    let hard_penalty = if rating == ReviewRating::Hard {
        params.w[15]
    } else {
        1.0
    };
    let easy_bonus = if rating == ReviewRating::Easy {
        params.w[16]
    } else {
        1.0
    };

    let s_new = s * (1.0
        + f64::exp(params.w[8])
            * (11.0 - d)
            * s.powf(-params.w[9])
            * (f64::exp((1.0 - r) * params.w[10]) - 1.0)
            * hard_penalty
            * easy_bonus);

    s_new.max(0.1)
}

/// Calculate next stability after failure (lapse)
fn next_stability_failure(d: f64, s: f64, r: f64, params: &FSRSParameters) -> f64 {
    let s_new = params.w[11]
        * d.powf(-params.w[12])
        * ((s + 1.0).powf(params.w[13]) - 1.0)
        * f64::exp((1.0 - r) * params.w[14]);

    s_new.clamp(0.1, s) // New stability should not exceed old
}

/// Calculate interval from stability to achieve target retention
fn stability_to_interval(stability: f64, params: &FSRSParameters) -> i32 {
    let r = params.request_retention;
    let interval = (stability / params.factor) * (r.powf(1.0 / params.decay) - 1.0);
    (interval.round() as i32).clamp(1, params.maximum_interval)
}

/// FSRS Algorithm Implementation
///
/// Calculate the next review state based on current state and rating.
///
/// Returns: (new_interval, new_stability, new_difficulty, new_reps, new_lapses, next_review_date)
pub fn calculate_fsrs(
    state: &FSRSState,
    rating: ReviewRating,
    params: Option<&FSRSParameters>,
) -> (i32, f64, f64, i32, i32, DateTime<Utc>) {
    let params = params.cloned().unwrap_or_default();

    // Calculate elapsed time since last review
    let elapsed_days = state
        .last_review
        .map(|lr| (Utc::now() - lr).num_seconds() as f64 / 86400.0)
        .unwrap_or(0.0)
        .max(0.0);

    let (new_stability, new_difficulty, new_reps, new_lapses) = if state.reps == 0 {
        // First review: initialize state
        let s = init_stability(rating, &params);
        let d = init_difficulty(rating, &params);
        let lapses = if rating == ReviewRating::Again { 1 } else { 0 };
        (s, d, 1, lapses)
    } else {
        // Subsequent review: update state
        let r = retrievability(elapsed_days, state.stability, &params);
        let new_d = next_difficulty(state.difficulty, rating, &params);

        let (new_s, new_lapses) = if rating == ReviewRating::Again {
            // Failed recall
            let s = next_stability_failure(state.difficulty, state.stability, r, &params);
            (s, state.lapses + 1)
        } else {
            // Successful recall
            let s = next_stability_success(state.difficulty, state.stability, r, rating, &params);
            (s, state.lapses)
        };

        (new_s, new_d, state.reps + 1, new_lapses)
    };

    let new_interval = stability_to_interval(new_stability, &params);
    let next_review = Utc::now() + Duration::days(new_interval as i64);

    (
        new_interval,
        new_stability,
        new_difficulty,
        new_reps,
        new_lapses,
        next_review,
    )
}

/// Simplified interface matching the old SM-2 signature for backward compatibility
///
/// Maps SM-2 parameters to FSRS state:
/// - ease_factor → difficulty (inverted scale)
/// - interval → used to estimate stability
/// - repetitions → reps
///
/// Returns: (new_interval, new_ease_factor_equivalent, new_repetitions, next_review_date)
#[allow(dead_code)] // Kept for backward compatibility with external integrations
pub fn calculate_sm2(
    current_interval: i32,
    current_ease_factor: f64,
    current_repetitions: i32,
    rating: ReviewRating,
) -> (i32, f64, i32, DateTime<Utc>) {
    // Map ease factor (1.3-2.5+) to difficulty (1-10)
    // Higher ease = lower difficulty
    let difficulty = ((2.5 - current_ease_factor.clamp(1.3, 3.0)) / 0.17 + 1.0).clamp(1.0, 10.0);

    // Estimate stability from interval (rough approximation)
    let stability = if current_repetitions == 0 {
        0.0
    } else {
        // For established cards, stability ≈ interval / -ln(0.9)
        (current_interval as f64) / 0.105
    };

    let state = FSRSState {
        stability,
        difficulty,
        reps: current_repetitions,
        lapses: 0,
        last_review: if current_interval > 0 {
            Some(Utc::now() - Duration::days(current_interval as i64))
        } else {
            None
        },
    };

    let (new_interval, _new_stability, new_difficulty, new_reps, _, next_review) =
        calculate_fsrs(&state, rating, None);

    // Map difficulty back to ease factor for compatibility
    let new_ease_factor = (2.5 - (new_difficulty - 1.0) * 0.17).clamp(1.3, 3.0);

    (new_interval, new_ease_factor, new_reps, next_review)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_review_good() {
        let state = FSRSState::default();
        let (interval, stability, difficulty, reps, lapses, _) =
            calculate_fsrs(&state, ReviewRating::Good, None);

        assert!(interval >= 1);
        assert!(stability > 0.0);
        assert!(difficulty >= 1.0 && difficulty <= 10.0);
        assert_eq!(reps, 1);
        assert_eq!(lapses, 0);
    }

    #[test]
    fn test_first_review_again_creates_lapse() {
        let state = FSRSState::default();
        let (_, _, _, reps, lapses, _) = calculate_fsrs(&state, ReviewRating::Again, None);

        assert_eq!(reps, 1);
        assert_eq!(lapses, 1);
    }

    #[test]
    fn test_retrievability_decreases_over_time() {
        let params = FSRSParameters::default();
        let stability = 10.0; // 10 days stability

        let r_day1 = retrievability(1.0, stability, &params);
        let r_day5 = retrievability(5.0, stability, &params);
        let r_day10 = retrievability(10.0, stability, &params);

        assert!(r_day1 > r_day5);
        assert!(r_day5 > r_day10);
        // At stability days, retrievability should be ~90%
        assert!((r_day10 - 0.9).abs() < 0.05);
    }

    #[test]
    fn test_easy_rating_increases_stability_more() {
        let state = FSRSState {
            stability: 5.0,
            difficulty: 5.0,
            reps: 2,
            lapses: 0,
            last_review: Some(Utc::now() - Duration::days(5)),
        };

        let (_, easy_stability, _, _, _, _) = calculate_fsrs(&state, ReviewRating::Easy, None);
        let (_, good_stability, _, _, _, _) = calculate_fsrs(&state, ReviewRating::Good, None);

        assert!(easy_stability > good_stability);
    }

    #[test]
    fn test_again_rating_resets_stability() {
        let state = FSRSState {
            stability: 30.0,
            difficulty: 5.0,
            reps: 5,
            lapses: 0,
            last_review: Some(Utc::now() - Duration::days(30)),
        };

        let (interval, stability, _, _, lapses, _) =
            calculate_fsrs(&state, ReviewRating::Again, None);

        assert!(stability < state.stability);
        assert!(interval < 30);
        assert_eq!(lapses, 1);
    }

    #[test]
    fn test_difficulty_bounded() {
        let params = FSRSParameters::default();

        // Many easy ratings should not push difficulty below 1
        let mut d = 5.0;
        for _ in 0..20 {
            d = next_difficulty(d, ReviewRating::Easy, &params);
        }
        assert!(d >= 1.0);

        // Many again ratings should not push difficulty above 10
        d = 5.0;
        for _ in 0..20 {
            d = next_difficulty(d, ReviewRating::Again, &params);
        }
        assert!(d <= 10.0);
    }

    #[test]
    fn test_backward_compatible_sm2_interface() {
        // Test that the backward-compatible interface works
        let (interval, ease, reps, _) = calculate_sm2(0, 2.5, 0, ReviewRating::Good);

        assert!(interval >= 1);
        assert!(ease >= 1.3 && ease <= 3.0);
        assert_eq!(reps, 1);
    }

    #[test]
    fn test_easy_longer_intervals_compat() {
        let (easy_interval, _, _, _) = calculate_sm2(6, 2.5, 2, ReviewRating::Easy);
        let (good_interval, _, _, _) = calculate_sm2(6, 2.5, 2, ReviewRating::Good);

        assert!(easy_interval >= good_interval);
    }
}
