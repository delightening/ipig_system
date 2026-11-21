//! IACUC No. 變更事件的「補登實際日期」
//! （`POST /api/v1/animals/{id}/events/{event_id}/actual-date`，migration 027）。
//!
//! 要釘住的契約：
//! 1. 補登成功後，`GET .../events` 該事件帶 `actual_date` 與 `backfill`（補登人、時間、原因）；
//!    沒補登過的事件兩者為 null。
//! 2. 只新增不覆蓋：再補登一次＝新增一筆，讀取取最新一筆；舊筆仍在表內，表本身 UPDATE／DELETE 被擋。
//! 3. 日期範圍：第一次分配＝[進場日, min(登錄日, 第一筆需計畫的試驗紀錄)]；
//!    之後的更換＝[前一事件的有效日期, 自己的登錄日]。體重、疫苗、血液檢查與一般／異常觀察不算試驗紀錄。
//! 4. 權限（`animal.field_correction.review`）、事件歸屬（屬於該動物且類型為 IACUC_CHANGE）、
//!    原因必填、日期格式，都要擋；每次補登寫一筆稽核事件。

mod common;

use serde_json::{json, Value};
use serial_test::serial;
use uuid::Uuid;

use common::TestApp;

/// 進場日。測試動物都用這個日期建立。
const ENTRY_DATE: &str = "2026-07-13";

fn animal_body(ear_tag: &str) -> Value {
    json!({
        "ear_tag": ear_tag,
        "gender": "male",
        "breed": "minipig",
        "entry_date": ENTRY_DATE,
        "entry_weight": 25.5,
        "pen_location": "A-01",
        "force_create": true,
    })
}

async fn create_animal(app: &TestApp, token: &str) -> Uuid {
    let created = common::create_animal_with_free_ear_tag(app, token, animal_body).await;
    Uuid::parse_str(created["id"].as_str().expect("animal id")).expect("uuid")
}

async fn put_animal(app: &TestApp, token: &str, id: Uuid, body: Value) {
    let res = app
        .auth_put(&format!("/api/v1/animals/{id}"), &body, token)
        .await;
    let status = res.status().as_u16();
    let json: Value = res.json().await.unwrap_or(Value::Null);
    assert_eq!(status, 200, "更新動物失敗：{json}");
}

async fn get_events(app: &TestApp, token: &str, id: Uuid) -> Vec<Value> {
    let res = app
        .auth_get(&format!("/api/v1/animals/{id}/events"), token)
        .await;
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.expect("events json");
    body.as_array().expect("events 應為陣列").clone()
}

/// 事件列表是新 → 舊排序；取出事件 id 並轉成由舊到新。
fn event_ids_oldest_first(events: &[Value]) -> Vec<Uuid> {
    events
        .iter()
        .rev()
        .map(|e| Uuid::parse_str(e["id"].as_str().expect("event id")).expect("event uuid"))
        .collect()
}

async fn backfill(
    app: &TestApp,
    token: &str,
    animal: Uuid,
    event: Uuid,
    date: &str,
    reason: &str,
) -> (u16, Value) {
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{animal}/events/{event}/actual-date"),
            &json!({ "actual_date": date, "reason": reason }),
            token,
        )
        .await;
    let status = res.status().as_u16();
    let body: Value = res.json().await.unwrap_or(Value::Null);
    (status, body)
}

/// 建立一隻有一個 IACUC 事件（首次分配）的動物，回傳 (動物 id, 事件 id)。
async fn animal_with_first_assignment(app: &TestApp, token: &str) -> (Uuid, Uuid) {
    let id = create_animal(app, token).await;
    put_animal(app, token, id, json!({ "iacuc_no": "PIG-TEST-A" })).await;
    let events = get_events(app, token, id).await;
    assert_eq!(events.len(), 1, "{events:?}");
    (id, event_ids_oldest_first(&events)[0])
}

/// 事件的登錄日（寫入時間換算成台北時間的日曆日，YYYY-MM-DD）。
///
/// 邊界斷言一律用事件自己的登錄日，不用「現在的台北今天」：兩者在測試跑過台北午夜時會差一天，
/// 而伺服器是以事件的登錄日當上限。
async fn registered_date(app: &TestApp, event: Uuid) -> String {
    sqlx::query_scalar(
        "SELECT to_char((created_at AT TIME ZONE 'Asia/Taipei')::date, 'YYYY-MM-DD') \
         FROM user_activity_logs WHERE id = $1",
    )
    .bind(event)
    .fetch_one(&app.db_pool)
    .await
    .expect("registered date")
}

/// 給定日期的隔天（YYYY-MM-DD）。
async fn day_after(app: &TestApp, date: &str) -> String {
    // $1::text::date：先固定參數型別為 text（sqlx 綁的是字串），再轉成日期
    sqlx::query_scalar("SELECT to_char(($1::text::date + 1), 'YYYY-MM-DD')")
        .bind(date)
        .fetch_one(&app.db_pool)
        .await
        .expect("day after")
}

async fn backfill_row_count(app: &TestApp, animal: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM animal_event_actual_dates WHERE animal_id = $1")
        .bind(animal)
        .fetch_one(&app.db_pool)
        .await
        .expect("count rows")
}

#[tokio::test]
#[serial]
async fn backfill_is_returned_on_the_timeline_and_unbackfilled_events_stay_null() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (id, first) = animal_with_first_assignment(&app, &token).await;
    // 再轉換一次，製造第二個沒補登的事件
    put_animal(&app, &token, id, json!({ "iacuc_no": "PIG-TEST-B" })).await;

    let before = get_events(&app, &token, id).await;
    assert_eq!(before.len(), 2, "{before:?}");
    // 要求「欄位存在且為 null」，不是「欄位缺席」：serde_json 對缺少的鍵索引也回 Null，
    // 用 e["x"].is_null() 會讓整個欄位被拿掉的回歸也通過
    for e in &before {
        assert_eq!(
            e.get("actual_date"),
            Some(&Value::Null),
            "沒補登過應為 null：{e}"
        );
        assert_eq!(
            e.get("backfill"),
            Some(&Value::Null),
            "沒補登過應為 null：{e}"
        );
    }

    let (status, body) = backfill(&app, &token, id, first, "2026-07-20", "依紙本申請單補登").await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["event_id"], first.to_string());
    assert_eq!(body["actual_date"], "2026-07-20");
    assert_eq!(body["backfill"]["reason"], "依紙本申請單補登");
    assert!(body["backfill"]["by_name"].is_string(), "{body}");
    assert!(body["backfill"]["at"].is_string(), "{body}");

    let after = get_events(&app, &token, id).await;
    let first_event = after
        .iter()
        .find(|e| e["id"] == first.to_string())
        .expect("first event");
    assert_eq!(first_event["actual_date"], "2026-07-20", "{first_event}");
    assert_eq!(first_event["backfill"]["reason"], "依紙本申請單補登");
    // 另一個事件不受影響
    let other = after
        .iter()
        .find(|e| e["id"] != first.to_string())
        .expect("other event");
    assert_eq!(other.get("actual_date"), Some(&Value::Null), "{other}");
    assert_eq!(other.get("backfill"), Some(&Value::Null), "{other}");
}

#[tokio::test]
#[serial]
async fn backfilling_again_appends_a_row_and_the_latest_one_wins() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (id, first) = animal_with_first_assignment(&app, &token).await;

    let (s1, b1) = backfill(&app, &token, id, first, "2026-07-20", "第一次補登").await;
    assert_eq!(s1, 200, "{b1}");
    let (s2, b2) = backfill(&app, &token, id, first, "2026-07-25", "日期更正").await;
    assert_eq!(s2, 200, "{b2}");

    let events = get_events(&app, &token, id).await;
    assert_eq!(events[0]["actual_date"], "2026-07-25", "{}", events[0]);
    assert_eq!(events[0]["backfill"]["reason"], "日期更正");
    assert_eq!(backfill_row_count(&app, id).await, 2, "兩次補登都應保留");

    // 稽核：每次補登一筆，且第二次的 before 帶出上一次的日期
    let audit: Vec<(Option<Value>, Option<Value>)> = sqlx::query_as(
        "SELECT before_data, after_data FROM user_activity_logs \
         WHERE entity_type = 'animal' AND entity_id = $1 AND event_type = 'IACUC_EVENT_DATE_BACKFILL' \
         ORDER BY created_at ASC",
    )
    .bind(id)
    .fetch_all(&app.db_pool)
    .await
    .expect("audit rows");
    assert_eq!(audit.len(), 2, "每次補登都要寫一筆稽核事件");
    let (before_0, after_0) = &audit[0];
    assert!(before_0.as_ref().expect("before 0")["actual_date"].is_null());
    assert_eq!(
        after_0.as_ref().expect("after 0")["actual_date"],
        "2026-07-20"
    );
    let (before_1, after_1) = &audit[1];
    assert_eq!(
        before_1.as_ref().expect("before 1")["actual_date"],
        "2026-07-20"
    );
    assert_eq!(
        after_1.as_ref().expect("after 1")["actual_date"],
        "2026-07-25"
    );
    assert_eq!(after_1.as_ref().expect("after 1")["reason"], "日期更正");
}

#[tokio::test]
#[serial]
async fn created_at_records_the_write_time_not_the_transaction_start() {
    // 讀「最新一筆」靠 created_at 排序，而補登是先鎖動物列、之後才 INSERT。
    // 預設若是 now()（交易開始時間），先開始卻較晚拿到鎖的補登會帶著較早的時間寫在後面，
    // 被誤判成較舊；clock_timestamp() 是寫入當下，鎖內的寫入順序就等於時間順序。
    let app = TestApp::spawn().await;
    let default: String = sqlx::query_scalar(
        "SELECT column_default FROM information_schema.columns \
         WHERE table_name = 'animal_event_actual_dates' AND column_name = 'created_at'",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("created_at default");
    assert!(
        default.contains("clock_timestamp"),
        "created_at 預設應為 clock_timestamp()，實際：{default}"
    );
}

#[tokio::test]
#[serial]
async fn the_table_itself_rejects_update_and_delete() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (id, first) = animal_with_first_assignment(&app, &token).await;
    let (status, body) = backfill(&app, &token, id, first, "2026-07-20", "補登").await;
    assert_eq!(status, 200, "{body}");

    let update =
        sqlx::query("UPDATE animal_event_actual_dates SET reason = 'x' WHERE event_id = $1")
            .bind(first)
            .execute(&app.db_pool)
            .await;
    assert!(update.is_err(), "UPDATE 應被觸發器拒絕");
    let delete = sqlx::query("DELETE FROM animal_event_actual_dates WHERE event_id = $1")
        .bind(first)
        .execute(&app.db_pool)
        .await;
    assert!(delete.is_err(), "DELETE 應被觸發器拒絕");
}

#[tokio::test]
#[serial]
async fn first_assignment_range_starts_at_entry_date_and_ends_at_today() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (id, first) = animal_with_first_assignment(&app, &token).await;
    let registered = registered_date(&app, first).await;
    let after_registered = day_after(&app, &registered).await;

    // 早於進場日
    let (s, b) = backfill(&app, &token, id, first, "2026-07-12", "太早").await;
    assert_eq!(s, 400, "{b}");
    assert!(
        b.to_string().contains(ENTRY_DATE),
        "訊息應帶出允許範圍：{b}"
    );

    // 晚於登錄日（沒有試驗紀錄時上限是登錄日）
    let (s, b) = backfill(&app, &token, id, first, &after_registered, "太晚").await;
    assert_eq!(s, 400, "{b}");

    // 兩端含在內
    let (s, b) = backfill(&app, &token, id, first, ENTRY_DATE, "進場當天").await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = backfill(&app, &token, id, first, &registered, "登錄當天").await;
    assert_eq!(s, 200, "{b}");
}

#[tokio::test]
#[serial]
async fn first_assignment_is_capped_by_the_first_plan_required_record_only() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (id, first) = animal_with_first_assignment(&app, &token).await;

    // 不需計畫的紀錄不應拉低上限：異常觀察、體重
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{id}/observations"),
            &json!({ "event_date": "2026-07-16", "record_type": "abnormal", "content": "異常觀察" }),
            &token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "異常觀察建立失敗：{}",
        res.status()
    );
    sqlx::query(
        "INSERT INTO animal_weights (animal_id, measure_date, weight) VALUES ($1, '2026-07-17', 26.0)",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("insert weight");
    // 需計畫的紀錄：試驗性觀察
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{id}/observations"),
            &json!({ "event_date": "2026-08-01", "record_type": "experiment", "content": "試驗性觀察" }),
            &token,
        )
        .await;
    assert!(
        res.status().is_success(),
        "試驗性觀察建立失敗：{}",
        res.status()
    );

    // 上限是第一筆試驗性觀察（08-01），不是更早的異常觀察或體重
    let (s, b) = backfill(&app, &token, id, first, "2026-08-02", "晚於試驗紀錄").await;
    assert_eq!(s, 400, "{b}");
    assert!(b.to_string().contains("2026-08-01"), "訊息應帶出上限：{b}");
    let (s, b) = backfill(&app, &token, id, first, "2026-08-01", "等於試驗紀錄當天").await;
    assert_eq!(s, 200, "{b}");
}

#[tokio::test]
#[serial]
async fn later_change_is_sandwiched_by_the_previous_event_and_its_own_registration() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (id, first) = animal_with_first_assignment(&app, &token).await;
    put_animal(&app, &token, id, json!({ "iacuc_no": "PIG-TEST-B" })).await;
    let events = get_events(&app, &token, id).await;
    let ids = event_ids_oldest_first(&events);
    assert_eq!(ids.len(), 2);
    let second = ids[1];
    let second_registered = registered_date(&app, second).await;

    // 前一事件還沒補登：有效日期＝它的登錄日，所以更換事件不能更早（07-30 遠早於登錄日）；
    // 上限是自己的登錄日，兩端含在內
    let (s, b) = backfill(&app, &token, id, second, "2026-07-30", "早於前一事件").await;
    assert_eq!(s, 400, "{b}");
    let (s, b) = backfill(&app, &token, id, second, &second_registered, "與登錄日同日").await;
    assert_eq!(s, 200, "{b}");

    // 前一事件補登到 07-20 之後，更換事件的下限跟著放寬到 07-20
    let (s, b) = backfill(&app, &token, id, first, "2026-07-20", "補登首次分配").await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = backfill(&app, &token, id, second, "2026-07-25", "補登更換").await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = backfill(&app, &token, id, second, "2026-07-19", "早於前一事件補登日").await;
    assert_eq!(s, 400, "{b}");
    assert!(b.to_string().contains("2026-07-20"), "訊息應帶出下限：{b}");

    // 首次分配不能晚於下一事件的有效日期（07-25）
    let (s, b) = backfill(&app, &token, id, first, "2026-07-26", "晚於下一事件").await;
    assert_eq!(s, 400, "{b}");
}

#[tokio::test]
#[serial]
async fn a_legacy_event_without_before_after_can_be_backfilled() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let id = create_animal(&app, &token).await;

    // 模擬修正前寫入的歷史事件：before/after 為空，只有顯示文字
    let actor: Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("any user");
    sqlx::query(
        "SELECT public.log_activity($1, 'ANIMAL', 'IACUC_CHANGE', 'animal', $2, \
         '[(無) → PIG-LEGACY-1] 243', NULL::jsonb, NULL::jsonb, NULL::inet, NULL::text, \
         NULL::uuid, NULL::text[], false)",
    )
    .bind(actor)
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("insert legacy-style audit row");
    let events = get_events(&app, &token, id).await;
    assert_eq!(events.len(), 1, "{events:?}");
    let event = event_ids_oldest_first(&events)[0];

    let (s, b) = backfill(&app, &token, id, event, "2026-07-20", "歷史事件補登").await;
    assert_eq!(s, 200, "{b}");
    let events = get_events(&app, &token, id).await;
    assert_eq!(events[0]["actual_date"], "2026-07-20", "{}", events[0]);
    // 回補的 before/after 仍在
    assert_eq!(
        events[0]["after_data"]["iacuc_no"], "PIG-LEGACY-1",
        "{}",
        events[0]
    );
}

#[tokio::test]
#[serial]
async fn invalid_input_is_rejected_and_writes_nothing() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (id, first) = animal_with_first_assignment(&app, &token).await;

    let (s, b) = backfill(&app, &token, id, first, "2026-07-20", "   ").await;
    assert_eq!(s, 400, "原因空白應被拒：{b}");
    let (s, b) = backfill(&app, &token, id, first, "2026/07/20", "格式錯誤").await;
    assert_eq!(s, 400, "日期格式錯誤應被拒：{b}");
    let (s, b) = backfill(&app, &token, id, first, "2026-07-20", &"字".repeat(501)).await;
    assert_eq!(s, 400, "原因過長應被拒：{b}");

    assert_eq!(
        backfill_row_count(&app, id).await,
        0,
        "被拒的請求不可寫入任何資料"
    );
}

#[tokio::test]
#[serial]
async fn the_event_must_belong_to_the_animal_and_be_an_iacuc_change() {
    let app = TestApp::spawn().await;
    let token = app.login_as_admin().await;
    let (id, first) = animal_with_first_assignment(&app, &token).await;
    let (other_animal, _other_event) = animal_with_first_assignment(&app, &token).await;

    // 事件屬於別隻動物
    let (s, b) = backfill(&app, &token, other_animal, first, "2026-07-20", "跨動物").await;
    assert_eq!(s, 404, "{b}");
    // 不存在的事件
    let (s, b) = backfill(&app, &token, id, Uuid::new_v4(), "2026-07-20", "不存在").await;
    assert_eq!(s, 404, "{b}");
    // 同一隻動物、但不是 IACUC_CHANGE 的事件（首次分配會同時寫 ANIMAL_UPDATE）
    let non_iacuc: Uuid = sqlx::query_scalar(
        "SELECT id FROM user_activity_logs \
         WHERE entity_type = 'animal' AND entity_id = $1 AND event_type = 'ANIMAL_UPDATE' LIMIT 1",
    )
    .bind(id)
    .fetch_one(&app.db_pool)
    .await
    .expect("an ANIMAL_UPDATE event");
    let (s, b) = backfill(&app, &token, id, non_iacuc, "2026-07-20", "類型不符").await;
    assert_eq!(s, 404, "{b}");
    // 不存在的動物
    let (s, b) = backfill(
        &app,
        &token,
        Uuid::new_v4(),
        first,
        "2026-07-20",
        "沒有動物",
    )
    .await;
    assert_eq!(s, 404, "{b}");

    assert_eq!(
        backfill_row_count(&app, id).await,
        0,
        "被拒的請求不可寫入任何資料"
    );
}

#[tokio::test]
#[serial]
async fn permission_is_required() {
    let app = TestApp::spawn().await;
    let admin = app.login_as_admin().await;
    let (id, first) = animal_with_first_assignment(&app, &admin).await;

    // 沒有任何角色的使用者：沒有 animal.field_correction.review
    let nobody = app
        .login_new_user_with_role("no-perm", "NO_SUCH_ROLE")
        .await;
    let (s, b) = backfill(&app, &nobody, id, first, "2026-07-20", "沒有權限").await;
    assert_eq!(s, 403, "{b}");

    // 未登入：app.client 的 cookie 罐在前面登入時已存了無權限使用者的 cookie，
    // 要用全新的 client 才是真的沒帶任何憑證
    let res = reqwest::Client::new()
        .post(app.url(&format!("/api/v1/animals/{id}/events/{first}/actual-date")))
        .json(&json!({ "actual_date": "2026-07-20", "reason": "未登入" }))
        .send()
        .await
        .expect("request");
    assert_eq!(res.status().as_u16(), 401);

    assert_eq!(
        backfill_row_count(&app, id).await,
        0,
        "被拒的請求不可寫入任何資料"
    );
}
