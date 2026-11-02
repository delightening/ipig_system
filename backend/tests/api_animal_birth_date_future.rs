//! 動物出生日期不可為未來日期（建立動物 API）。
//!
//! 規則只作用於新寫入；以台灣時間的今日為界，當天允許、明天起擋下（400）。

mod common;

use serial_test::serial;

fn animal_body(ear_tag: &str, birth_date: &str) -> serde_json::Value {
    serde_json::json!({
        "ear_tag": ear_tag,
        "gender": "male",
        "breed": "minipig",
        "birth_date": birth_date,
        "entry_date": "2026-07-13",
        "entry_weight": 59.0,
        "pen_location": "A-01",
        "force_create": true,
    })
}

fn rand_ear_tag() -> String {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time should be valid");
    format!("{:03}", (d.as_nanos() % 900) as u32 + 100)
}

#[tokio::test]
#[serial]
async fn create_animal_with_future_birth_date_is_rejected() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let tomorrow = erp_backend::time::today_taiwan_naive() + chrono::Days::new(1);
    let body = animal_body(&rand_ear_tag(), &tomorrow.to_string());
    let res = app.auth_post("/api/v1/animals", &body, &token).await;

    assert_eq!(
        res.status(),
        400,
        "未來的出生日期應回 400，實際 {}",
        res.status()
    );
    let text = res.text().await.unwrap_or_default();
    assert!(
        text.contains("出生日期不可晚於今日"),
        "錯誤訊息應說明原因，實際 {text}"
    );
}

#[tokio::test]
#[serial]
async fn create_animal_with_today_birth_date_is_allowed() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let today = erp_backend::time::today_taiwan_naive();
    let body = animal_body(&rand_ear_tag(), &today.to_string());
    let res = app.auth_post("/api/v1/animals", &body, &token).await;

    assert!(
        res.status() == 200 || res.status() == 201,
        "當天出生日期應可建立，實際 {}",
        res.status()
    );
}
