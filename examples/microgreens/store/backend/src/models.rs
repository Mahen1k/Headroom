use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// (min_days, max_days, min_height_cm, max_height_cm) harvest window per variety.
pub fn variety_window(variety: &str) -> Option<(i64, i64, f64, f64)> {
    match variety {
        "sunflower" => Some((8, 12, 6.0, 10.0)),
        "pea" => Some((8, 14, 5.0, 12.0)),
        "radish" => Some((6, 10, 4.0, 8.0)),
        "broccoli" => Some((7, 12, 3.0, 7.0)),
        "arugula" => Some((6, 10, 3.0, 6.0)),
        "kale" => Some((8, 12, 3.0, 6.0)),
        "cilantro" => Some((14, 21, 4.0, 8.0)),
        "basil" => Some((14, 21, 3.0, 6.0)),
        _ => None,
    }
}

pub fn known_varieties() -> Vec<&'static str> {
    vec![
        "sunflower", "pea", "radish", "broccoli", "arugula", "kale", "cilantro", "basil",
    ]
}

#[derive(Debug, sqlx::FromRow)]
pub struct BatchRow {
    pub id: i64,
    pub variety: String,
    pub sow_date: String,
    pub quantity_trays: i64,
    pub price_per_tray: f64,
    pub height_cm: Option<f64>,
    pub mold: Option<i64>,
    pub color_uniform: Option<i64>,
    pub stem_ok: Option<i64>,
    pub root_mat_ok: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct Product {
    pub id: i64,
    pub variety: String,
    pub sow_date: String,
    pub days_since_sowing: i64,
    pub quantity_trays: i64,
    pub price_per_tray: f64,
    pub grade: String,
}

/// Grade a batch's quality check: mold -> reject, one soft issue -> B, none -> A.
fn quality_grade(row: &BatchRow) -> (String, Vec<String>) {
    let mut reasons = Vec::new();

    if row.mold.unwrap_or(0) != 0 {
        return ("reject".into(), vec!["mold detected".into()]);
    }
    if row.stem_ok == Some(0) {
        reasons.push("stems too short/leggy".into());
    }
    if row.root_mat_ok == Some(0) {
        reasons.push("root mat not fully established".into());
    }
    if row.color_uniform == Some(0) {
        reasons.push("uneven coloring".into());
    }

    if reasons.is_empty() {
        ("A".into(), reasons)
    } else if reasons.len() == 1 {
        ("B".into(), reasons)
    } else {
        ("reject".into(), reasons)
    }
}

/// Returns (salable, grade, reasons) combining harvest readiness and quality grading.
pub fn evaluate_salability(row: &BatchRow, today: NaiveDate) -> (bool, String, Vec<String>) {
    let Some((min_days, max_days, min_h, max_h)) = variety_window(&row.variety) else {
        return (false, "unrated".into(), vec!["unknown variety".into()]);
    };

    let sow_date = match NaiveDate::parse_from_str(&row.sow_date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return (false, "unrated".into(), vec!["invalid sow date".into()]),
    };
    let days = (today - sow_date).num_days();

    let mut reasons = Vec::new();
    let mut days_ok = true;
    if days < min_days {
        reasons.push(format!(
            "only {days} days since sowing, needs at least {min_days}"
        ));
        days_ok = false;
    } else if days > max_days {
        reasons.push(format!(
            "{days} days since sowing exceeds max {max_days} (overgrown)"
        ));
        days_ok = false;
    }

    let Some(height) = row.height_cm else {
        reasons.push("no quality check recorded".into());
        return (false, "unrated".into(), reasons);
    };

    let mut height_ok = true;
    if height < min_h {
        reasons.push(format!("height {height}cm below minimum {min_h}cm"));
        height_ok = false;
    } else if height > max_h {
        reasons.push(format!("height {height}cm above maximum {max_h}cm"));
        height_ok = false;
    }

    let (grade, grade_reasons) = quality_grade(row);
    reasons.extend(grade_reasons);

    let salable = days_ok && height_ok && (grade == "A" || grade == "B");

    (salable, grade, reasons)
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct AddBatchRequest {
    pub variety: String,
    pub sow_date: String,
    pub quantity_trays: i64,
    pub price_per_tray: f64,
}

#[derive(Debug, Deserialize)]
pub struct InspectRequest {
    pub height_cm: f64,
    pub mold: bool,
    pub color_uniform: bool,
    pub stem_ok: bool,
    pub root_mat_ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddToCartRequest {
    pub batch_id: i64,
    pub quantity: i64,
}

#[derive(Debug, Serialize)]
pub struct CartItemView {
    pub id: i64,
    pub batch_id: i64,
    pub variety: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub line_total: f64,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    pub discount_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderView {
    pub id: i64,
    pub subtotal: f64,
    pub total: f64,
    pub discount_code: Option<String>,
    pub items: Vec<CartItemView>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_sunflower(sow_date: &str) -> BatchRow {
        BatchRow {
            id: 1,
            variety: "sunflower".into(),
            sow_date: sow_date.into(),
            quantity_trays: 10,
            price_per_tray: 4.5,
            height_cm: Some(7.5),
            mold: Some(0),
            color_uniform: Some(1),
            stem_ok: Some(1),
            root_mat_ok: Some(1),
        }
    }

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn healthy_batch_within_window_is_salable_grade_a() {
        let row = healthy_sunflower("2026-08-04");
        let (salable, grade, reasons) = evaluate_salability(&row, date("2026-08-14"));
        assert!(salable);
        assert_eq!(grade, "A");
        assert!(reasons.is_empty());
    }

    #[test]
    fn too_young_batch_is_not_salable() {
        let row = healthy_sunflower("2026-08-13");
        let (salable, _grade, reasons) = evaluate_salability(&row, date("2026-08-14"));
        assert!(!salable);
        assert!(reasons.iter().any(|r| r.contains("only 1 days")));
    }

    #[test]
    fn overgrown_batch_is_not_salable() {
        let row = healthy_sunflower("2026-07-01");
        let (salable, _grade, reasons) = evaluate_salability(&row, date("2026-08-14"));
        assert!(!salable);
        assert!(reasons.iter().any(|r| r.contains("overgrown")));
    }

    #[test]
    fn mold_forces_reject_regardless_of_other_checks() {
        let mut row = healthy_sunflower("2026-08-04");
        row.mold = Some(1);
        let (salable, grade, reasons) = evaluate_salability(&row, date("2026-08-14"));
        assert!(!salable);
        assert_eq!(grade, "reject");
        assert_eq!(reasons, vec!["mold detected".to_string()]);
    }

    #[test]
    fn single_soft_issue_grades_b_but_still_salable() {
        let mut row = healthy_sunflower("2026-08-04");
        row.stem_ok = Some(0);
        let (salable, grade, reasons) = evaluate_salability(&row, date("2026-08-14"));
        assert!(salable);
        assert_eq!(grade, "B");
        assert_eq!(reasons, vec!["stems too short/leggy".to_string()]);
    }

    #[test]
    fn two_soft_issues_reject() {
        let mut row = healthy_sunflower("2026-08-04");
        row.stem_ok = Some(0);
        row.root_mat_ok = Some(0);
        let (salable, grade, _reasons) = evaluate_salability(&row, date("2026-08-14"));
        assert!(!salable);
        assert_eq!(grade, "reject");
    }

    #[test]
    fn height_outside_window_is_not_salable() {
        let mut row = healthy_sunflower("2026-08-04");
        row.height_cm = Some(2.0);
        let (salable, _grade, reasons) = evaluate_salability(&row, date("2026-08-14"));
        assert!(!salable);
        assert!(reasons.iter().any(|r| r.contains("below minimum")));
    }

    #[test]
    fn no_quality_check_yet_is_unrated() {
        let mut row = healthy_sunflower("2026-08-04");
        row.height_cm = None;
        let (salable, grade, reasons) = evaluate_salability(&row, date("2026-08-14"));
        assert!(!salable);
        assert_eq!(grade, "unrated");
        assert!(reasons.iter().any(|r| r.contains("no quality check recorded")));
    }

    #[test]
    fn unknown_variety_is_unrated() {
        let mut row = healthy_sunflower("2026-08-04");
        row.variety = "durian".into();
        let (salable, grade, _reasons) = evaluate_salability(&row, date("2026-08-14"));
        assert!(!salable);
        assert_eq!(grade, "unrated");
    }

    #[test]
    fn known_varieties_matches_variety_window_entries() {
        for variety in known_varieties() {
            assert!(variety_window(variety).is_some());
        }
    }
}
