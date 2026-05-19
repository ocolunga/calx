use chrono::{Datelike, Duration, NaiveDate};

// ── Week configuration ────────────────────────────────────────────────────────

/// Everything that controls how weeks are numbered.
/// `first_day`: 1 = Monday … 7 = Sunday.
/// `min_days`:  1 = simple (Jan 1 is always week 1), 4 = ISO-style boundary.
#[derive(Clone, Copy, Debug)]
pub struct WeekConfig {
    pub first_day: u8,
    pub min_days: u8,
}

impl Default for WeekConfig {
    fn default() -> Self {
        Self { first_day: 1, min_days: 4 } // standard ISO Monday
    }
}

// ── Simple calculations ───────────────────────────────────────────────────────

/// Day 1 = January 1, day 366 = December 31 of a leap year.
pub fn day_of_year(date: NaiveDate) -> u32 {
    date.ordinal() // chrono provides this directly
}

/// Pair weeks into fortnights: weeks 1-2 → biweek 1, weeks 3-4 → biweek 2, …
pub fn compute_biweek_number(week: u32) -> u32 {
    (week + 1) / 2
}

// ── Week number ───────────────────────────────────────────────────────────────

/// Compute the week number for `date` according to `cfg`.
///
/// Key subtlety: Python's `%` is always non-negative (e.g. `-1 % 7 == 6`),
/// but Rust's `%` can be negative (e.g. `-1 % 7 == -1`).  Anywhere we need
/// a non-negative modulo we use `.rem_euclid(7)` instead.
pub fn compute_week_number(date: NaiveDate, cfg: WeekConfig) -> u32 {
    // Fast path: standard ISO Monday-based weeks via chrono's built-in.
    if cfg.first_day == 1 && cfg.min_days == 4 {
        return date.iso_week().week();
    }

    let year = date.year();
    let jan1 = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();

    // Convert first_day (1=Mon…7=Sun) to 0-based Monday-anchored index.
    // chrono's num_days_from_monday() returns Mon=0 … Sun=6, same convention.
    let py_first = ((cfg.first_day as i32) - 1) % 7;
    let jan1_wd = jan1.weekday().num_days_from_monday() as i32;

    // How many days into the first display-week does Jan 1 fall?
    let jan1_offset = (jan1_wd - py_first).rem_euclid(7);

    // 0-based day-of-year (Jan 1 = 0).
    let doy = (date - jan1).num_days() as i32;

    // ── Simple mode: no year-boundary spillover ───────────────────────────
    if cfg.min_days == 1 {
        return ((doy + jan1_offset) / 7 + 1) as u32;
    }

    // ── ISO-style mode ────────────────────────────────────────────────────
    // How many days of the first partial week fall within this year?
    let days_in_first_partial = if jan1_offset > 0 { 7 - jan1_offset } else { 7 };

    let week = if days_in_first_partial >= cfg.min_days as i32 {
        // The first partial week has enough days to count as week 1.
        (doy + jan1_offset) / 7 + 1
    } else {
        // The first partial week is too short — it belongs to last year.
        if doy < days_in_first_partial {
            // This date itself falls in that short leading fragment: recurse
            // into the previous year to find its actual week number.
            let dec31_prev = NaiveDate::from_ymd_opt(year - 1, 12, 31).unwrap();
            return compute_week_number(dec31_prev, cfg);
        }
        // Dates after the leading fragment: renumber from the first full week.
        (doy - days_in_first_partial) / 7 + 1
    };

    // Check whether this date has already crossed into next year's week 1.
    let next_jan1 = NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap();
    let next_wd = next_jan1.weekday().num_days_from_monday() as i32;
    let next_offset = (next_wd - py_first).rem_euclid(7);
    let next_days_in_first = if next_offset > 0 { 7 - next_offset } else { 7 };

    if next_days_in_first >= cfg.min_days as i32 {
        // Next year's week 1 starts at next_jan1 - next_offset days.
        let week1_start_doy = (next_jan1 - jan1).num_days() as i32 - next_offset;
        if doy >= week1_start_doy {
            return 1;
        }
    }

    week as u32
}

// ── Calendar grid ─────────────────────────────────────────────────────────────

/// Build the calendar grid for a month: each element is one display-week,
/// each week is exactly 7 consecutive NaiveDates.
/// Dates that belong to adjacent months are included so week numbers can be
/// computed correctly — callers must check `date.month() == month` before
/// rendering a cell.
pub fn month_weeks(year: i32, month: u32, cfg: WeekConfig) -> Vec<[NaiveDate; 7]> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let last = last_day_of_month(year, month);

    // 0-based Monday-anchored weekday of the first of the month.
    let py_first = ((cfg.first_day as i64) - 1) % 7;
    let first_wd = first.weekday().num_days_from_monday() as i64;
    let back = (first_wd - py_first).rem_euclid(7);

    let grid_start = first - Duration::days(back);
    let mut weeks = Vec::new();
    let mut cursor = grid_start;

    loop {
        let week: [NaiveDate; 7] = std::array::from_fn(|i| cursor + Duration::days(i as i64));
        weeks.push(week);
        cursor += Duration::days(7);
        // Keep going as long as any day in the upcoming week is still in this month.
        if cursor > last {
            break;
        }
    }

    weeks
}

fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    let (y, m) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    NaiveDate::from_ymd_opt(y, m, 1).unwrap() - Duration::days(1)
}

// ── Labels ────────────────────────────────────────────────────────────────────

const ALL_DAY_NAMES: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

/// Two-letter day abbreviations starting from `first_day`.
pub fn day_names(first_day: u8) -> [&'static str; 7] {
    let idx = ((first_day - 1) % 7) as usize;
    std::array::from_fn(|i| ALL_DAY_NAMES[(idx + i) % 7])
}

/// Human-readable description of the week-numbering scheme.
pub fn view_label(cfg: WeekConfig) -> String {
    if cfg.first_day == 1 && cfg.min_days == 4 {
        return "ISO Week".to_string();
    }
    let day_name = match cfg.first_day {
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        7 => "Sunday",
        _ => "Unknown",
    };
    let style = if cfg.min_days == 1 { "Simple" } else { "ISO-like" };
    format!("{day_name} {style}")
}

// ── Month navigation ──────────────────────────────────────────────────────────

pub fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 { (year - 1, 12) } else { (year, month - 1) }
}

pub fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 { (year + 1, 1) } else { (year, month + 1) }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn iso_week_fast_path() {
        let cfg = WeekConfig::default();
        assert_eq!(compute_week_number(d(2024, 1, 1), cfg), 1);
        assert_eq!(compute_week_number(d(2023, 1, 1), cfg), 52); // prev-year spillover
        assert_eq!(compute_week_number(d(2023, 12, 31), cfg), 52);
        assert_eq!(compute_week_number(d(2026, 5, 18), cfg), 21);
        assert_eq!(compute_week_number(d(2025, 12, 29), cfg), 1); // next-year spillover
    }

    #[test]
    fn simple_monday() {
        let cfg = WeekConfig { first_day: 1, min_days: 1 };
        assert_eq!(compute_week_number(d(2024, 1, 1), cfg), 1);
        assert_eq!(compute_week_number(d(2024, 1, 7), cfg), 1);
        assert_eq!(compute_week_number(d(2024, 1, 8), cfg), 2);
    }

    #[test]
    fn simple_sunday() {
        let cfg = WeekConfig { first_day: 7, min_days: 1 };
        assert_eq!(compute_week_number(d(2024, 1, 1), cfg), 1); // Jan 1 is Monday → week 1
        assert_eq!(compute_week_number(d(2024, 1, 7), cfg), 2); // first Sunday → week 2 starts
    }

    #[test]
    fn biweek_pairs() {
        assert_eq!(compute_biweek_number(1), 1);
        assert_eq!(compute_biweek_number(2), 1);
        assert_eq!(compute_biweek_number(3), 2);
        assert_eq!(compute_biweek_number(4), 2);
        assert_eq!(compute_biweek_number(52), 26);
    }

    #[test]
    fn day_of_year_values() {
        assert_eq!(day_of_year(d(2024, 1, 1)), 1);
        assert_eq!(day_of_year(d(2024, 12, 31)), 366); // 2024 is a leap year
        assert_eq!(day_of_year(d(2025, 12, 31)), 365);
    }

    #[test]
    fn month_weeks_row_count() {
        let cfg = WeekConfig::default();
        // May 2026: Mon 4 May → Sun 31 May fits in 5 display weeks
        let weeks = month_weeks(2026, 5, cfg);
        assert!(weeks.len() >= 4 && weeks.len() <= 6);
        // Every week has 7 days
        for w in &weeks {
            assert_eq!(w.len(), 7);
        }
        // At least one day per week is in the target month
        for w in &weeks {
            assert!(w.iter().any(|d| d.month() == 5));
        }
    }
}
