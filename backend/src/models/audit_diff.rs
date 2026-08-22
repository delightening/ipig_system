//! Audit log 的 before/after diff 計算工具。
//!
//! Service-driven audit 重構的核心工具之一：entity 變更前後比對產出
//! - `before`：舊資料 JSON（供 user_activity_logs.before_data 欄位）
//! - `after`：新資料 JSON（user_activity_logs.after_data 欄位）
//! - `changed_fields`：變動欄位清單（方便稽核員快速掃視；包含敏感欄位名，
//!   但值會被 redact 為 "[REDACTED]"）
//!
//! 搭配 [`AuditRedact`] trait 讓敏感欄位（密碼雜湊、token 等）的**值**被
//! 替換為 `"[REDACTED]"`，**欄位名仍保留**於 before/after/changed_fields，
//! 讓 GLP 稽核員可查詢「哪些 user 改過密碼」這類問題。

use serde::Serialize;
use serde_json::{Map, Value};

/// 計算 entity 變更前後的差異，產出可寫入 `user_activity_logs` 的 payload。
///
/// # 封裝設計
///
/// 欄位為 `pub(crate)` — 同 crate 內的 [`AuditService`] 可直接存取以寫入
/// DB；外部呼叫者只能透過 [`before`](Self::before) / [`after`](Self::after) /
/// [`changed_fields`](Self::changed_fields) 讀取，或 [`into_parts`](Self::into_parts)
/// 消費所有權。
///
/// 建構只能透過 [`compute`](Self::compute) / [`create_only`](Self::create_only) /
/// [`delete_only`](Self::delete_only) / [`empty`](Self::empty)，**無法繞過 redact**。
#[derive(Debug, Clone, Default)]
pub struct DataDiff {
    pub(crate) before: Option<Value>,
    pub(crate) after: Option<Value>,
    pub(crate) changed_fields: Vec<String>,
}

/// Entity 宣告哪些欄位應從 audit log 中脫敏（避免密碼/token 等洩漏）。
///
/// 欄位路徑支援三種寫法：
/// - `"password_hash"` — top-level 欄位
/// - `"/password_hash"` — JSON Pointer（語意同上）
/// - `"/profile/api_key"` — 巢狀路徑（`profile` 內的 `api_key`）
/// - `"/sessions/0/token"` — 陣列元素的欄位
///
/// 脫敏後該欄位的值變為字串 `"[REDACTED]"`，**欄位名保留在 JSON 與
/// changed_fields 中**，方便稽核查詢「誰改過密碼」而不洩漏密碼值。
///
/// # Example
/// ```ignore
/// impl AuditRedact for User {
///     fn redacted_fields() -> &'static [&'static str] {
///         &["password_hash", "totp_secret", "mfa_backup_codes"]
///     }
/// }
/// ```
pub trait AuditRedact {
    fn redacted_fields() -> &'static [&'static str] {
        &[]
    }
}

/// 被脫敏欄位在 JSON 裡替換成的 sentinel 值。
const REDACTED: &str = "[REDACTED]";

impl DataDiff {
    /// 讀取 before JSON（已 redact，不含敏感值）。
    pub fn before(&self) -> Option<&Value> {
        self.before.as_ref()
    }

    /// 讀取 after JSON（已 redact，不含敏感值）。
    pub fn after(&self) -> Option<&Value> {
        self.after.as_ref()
    }

    /// 讀取變動欄位清單（含被 redact 的欄位名）。
    pub fn changed_fields(&self) -> &[String] {
        &self.changed_fields
    }

    /// 消費 self 取得三元組，用於 [`AuditService`] 寫入 DB。
    pub fn into_parts(self) -> (Option<Value>, Option<Value>, Vec<String>) {
        (self.before, self.after, self.changed_fields)
    }

    /// 對兩個可序列化 + 可 redact 的 entity 計算差異。
    ///
    /// 流程：
    /// 1. 序列化 before / after 為 JSON（失敗則 tracing::error 並 fallback None）
    /// 2. 在**原始 JSON**（未 redact）上計算 changed_fields
    /// 3. 對 before / after 套 redact：敏感欄位的**值**替換為 `"[REDACTED]"`
    ///
    /// 語意：
    /// - `(None, None)` → 空 diff
    /// - `(None, Some)` → CREATE：`before=None`, `after=<redacted>`, `changed_fields=["*"]`
    /// - `(Some, None)` → DELETE：`before=<redacted>`, `after=None`, `changed_fields=["*"]`
    /// - `(Some, Some)` → UPDATE：兩者都有 + 完整 changed_fields（含 redacted 欄位名）
    pub fn compute<T: Serialize + AuditRedact>(before: Option<&T>, after: Option<&T>) -> Self {
        let before_raw = before.and_then(|v| serialize_or_log::<T>(v, "before"));
        let after_raw = after.and_then(|v| serialize_or_log::<T>(v, "after"));

        // changed_fields 在 **未 redact** 的原始 JSON 上計算
        // → 密碼變更也會出現在清單中，稽核員可查詢
        let changed_fields = match (&before_raw, &after_raw) {
            (Some(Value::Object(b)), Some(Value::Object(a))) => diff_object_keys(b, a),
            (None, Some(_)) | (Some(_), None) => vec!["*".into()],
            _ => Vec::new(),
        };

        // 套 redact：敏感欄位的值變成 "[REDACTED]"，欄位名保留
        let redacted = T::redacted_fields();
        let before_json = before_raw.map(|j| apply_redact(j, redacted));
        let after_json = after_raw.map(|j| apply_redact(j, redacted));

        Self {
            before: before_json,
            after: after_json,
            changed_fields,
        }
    }

    /// 無 diff 內容（給沒有 before/after 概念的事件使用，如 PROTOCOL_SUBMIT）。
    pub fn empty() -> Self {
        Self::default()
    }

    /// 只記 after（用於 CREATE 情境）。
    pub fn create_only<T: Serialize + AuditRedact>(after: &T) -> Self {
        Self::compute::<T>(None, Some(after))
    }

    /// 只記 before（用於 DELETE 情境）。
    pub fn delete_only<T: Serialize + AuditRedact>(before: &T) -> Self {
        Self::compute::<T>(Some(before), None)
    }
}

/// 序列化 entity 為 Value；失敗則 tracing::error!（不 panic、不 bubble up）。
///
/// GLP 考量：失敗極罕見（derive Serialize 的純資料 struct 幾乎不會失敗），
/// 但若發生需可見。Prometheus 可對 `[audit] serialize failed` log pattern
/// 設 alert。
fn serialize_or_log<T: Serialize>(value: &T, which: &'static str) -> Option<Value> {
    match serde_json::to_value(value) {
        Ok(j) => Some(j),
        Err(e) => {
            tracing::error!(
                type_name = std::any::type_name::<T>(),
                error = %e,
                which = which,
                "[audit] DataDiff::compute serialize failed — entity absent from audit log"
            );
            None
        }
    }
}

/// 對 JSON 套用多個 redact 路徑。每個路徑的目標值會被替換為 `"[REDACTED]"`；
/// 欄位名與 JSON 結構保持不變。
fn apply_redact(value: Value, fields: &[&str]) -> Value {
    if fields.is_empty() {
        return value;
    }
    let mut result = value;
    for f in fields {
        // 同時接受 "password_hash" 和 "/password_hash"
        let path = f.strip_prefix('/').unwrap_or(f);
        let parts: Vec<&str> = path.split('/').collect();
        result = redact_at_path(result, &parts);
    }
    result
}

/// 遞迴將指定路徑的 leaf 值替換為 `"[REDACTED]"`。路徑不存在則不變動。
/// 陣列 index 支援數字字串（例如 "0" 取第 0 個元素）。
fn redact_at_path(value: Value, parts: &[&str]) -> Value {
    if parts.is_empty() {
        return Value::String(REDACTED.into());
    }
    let head = parts[0];
    let rest = &parts[1..];

    match value {
        Value::Object(mut map) => {
            if rest.is_empty() {
                if map.contains_key(head) {
                    map.insert(head.to_string(), Value::String(REDACTED.into()));
                }
            } else if let Some(child) = map.remove(head) {
                let redacted = redact_at_path(child, rest);
                map.insert(head.to_string(), redacted);
            }
            Value::Object(map)
        }
        Value::Array(mut arr) => {
            if let Ok(idx) = head.parse::<usize>() {
                if idx < arr.len() {
                    let child = std::mem::take(&mut arr[idx]);
                    arr[idx] = if rest.is_empty() {
                        Value::String(REDACTED.into())
                    } else {
                        redact_at_path(child, rest)
                    };
                }
            }
            Value::Array(arr)
        }
        // leaf 不匹配（例如路徑指向物件但值是 String）→ 不變
        other => other,
    }
}

/// 計算兩個 JSON 物件的 changed keys（不論值是否敏感）。
fn diff_object_keys(before: &Map<String, Value>, after: &Map<String, Value>) -> Vec<String> {
    let mut changed = Vec::new();
    for (key, after_v) in after {
        match before.get(key) {
            Some(before_v) if before_v != after_v => changed.push(key.clone()),
            None => changed.push(key.clone()),
            _ => {}
        }
    }
    for key in before.keys() {
        if !after.contains_key(key) {
            changed.push(key.clone());
        }
    }
    changed.sort();
    changed.dedup();
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json::json;

    #[derive(Serialize)]
    struct Animal {
        id: u32,
        ear_tag: String,
        weight: f64,
        pen: String,
    }

    impl AuditRedact for Animal {}

    #[derive(Serialize)]
    struct UserLike {
        id: u32,
        email: String,
        password_hash: String,
        totp_secret: Option<String>,
    }

    impl AuditRedact for UserLike {
        fn redacted_fields() -> &'static [&'static str] {
            &["password_hash", "totp_secret"]
        }
    }

    #[derive(Serialize)]
    struct Profile {
        name: String,
        credentials: Credentials,
    }

    #[derive(Serialize)]
    struct Credentials {
        api_key: String,
        public_id: String,
    }

    impl AuditRedact for Profile {
        fn redacted_fields() -> &'static [&'static str] {
            &["/credentials/api_key"]
        }
    }

    #[derive(Serialize)]
    struct SessionList {
        user_id: u32,
        sessions: Vec<Session>,
    }

    #[derive(Serialize)]
    struct Session {
        id: String,
        token: String,
    }

    impl AuditRedact for SessionList {
        fn redacted_fields() -> &'static [&'static str] {
            &["/sessions/0/token"]
        }
    }

    #[test]
    fn diff_update_detects_single_field_change() {
        let before = Animal {
            id: 1,
            ear_tag: "pig1".into(),
            weight: 25.0,
            pen: "A1".into(),
        };
        let after = Animal {
            id: 1,
            ear_tag: "pig1".into(),
            weight: 28.0,
            pen: "A1".into(),
        };
        let diff = DataDiff::compute(Some(&before), Some(&after));
        assert_eq!(diff.changed_fields(), &["weight".to_string()]);
    }

    #[test]
    fn diff_update_detects_multiple_field_changes() {
        let before = Animal {
            id: 1,
            ear_tag: "pig1".into(),
            weight: 25.0,
            pen: "A1".into(),
        };
        let after = Animal {
            id: 1,
            ear_tag: "pig1".into(),
            weight: 28.0,
            pen: "B2".into(),
        };
        let diff = DataDiff::compute(Some(&before), Some(&after));
        assert_eq!(
            diff.changed_fields(),
            &["pen".to_string(), "weight".to_string()]
        );
    }

    #[test]
    fn create_only_marks_full_changeset() {
        let a = Animal {
            id: 1,
            ear_tag: "pig1".into(),
            weight: 25.0,
            pen: "A1".into(),
        };
        let diff = DataDiff::create_only(&a);
        assert_eq!(diff.changed_fields(), &["*".to_string()]);
        assert!(diff.before().is_none());
        assert!(diff.after().is_some());
    }

    #[test]
    fn delete_only_marks_full_changeset() {
        let a = Animal {
            id: 1,
            ear_tag: "pig1".into(),
            weight: 25.0,
            pen: "A1".into(),
        };
        let diff = DataDiff::delete_only(&a);
        assert_eq!(diff.changed_fields(), &["*".to_string()]);
        assert!(diff.before().is_some());
        assert!(diff.after().is_none());
    }

    #[test]
    fn redact_replaces_value_but_keeps_field_name() {
        let u = UserLike {
            id: 1,
            email: "a@b.c".into(),
            password_hash: "$argon2id$...".into(),
            totp_secret: Some("TOTP_SECRET_XYZ".into()),
        };
        let diff = DataDiff::create_only(&u);
        let after = diff.after().expect("after should be present").clone();
        let obj = after.as_object().expect("should be object");

        // 非敏感欄位保留原值
        assert_eq!(obj.get("email"), Some(&json!("a@b.c")));
        assert_eq!(obj.get("id"), Some(&json!(1)));

        // 敏感欄位：**名稱保留**，但**值被脫敏為 "[REDACTED]"**
        assert_eq!(obj.get("password_hash"), Some(&json!(REDACTED)));
        assert_eq!(obj.get("totp_secret"), Some(&json!(REDACTED)));
    }

    #[test]
    fn redact_preserves_field_in_changed_fields_gdpr_compliant() {
        let before = UserLike {
            id: 1,
            email: "old@example.com".into(),
            password_hash: "old_hash".into(),
            totp_secret: None,
        };
        let after = UserLike {
            id: 1,
            email: "new@example.com".into(),
            password_hash: "new_hash".into(),
            totp_secret: None,
        };
        let diff = DataDiff::compute(Some(&before), Some(&after));

        // 兩邊都看不到具體密碼值
        let before_obj = diff.before().expect("before").clone();
        let after_obj = diff.after().expect("after").clone();
        assert_eq!(before_obj["password_hash"], json!(REDACTED));
        assert_eq!(after_obj["password_hash"], json!(REDACTED));

        // GLP 稽核關鍵：password_hash 仍出現在 changed_fields
        // 稽核員可用 `'password_hash' = ANY(changed_fields)` SQL 查詢
        let cf = diff.changed_fields();
        assert!(cf.contains(&"email".to_string()));
        assert!(cf.contains(&"password_hash".to_string()));
    }

    #[test]
    fn redact_with_nested_path_via_json_pointer() {
        let p = Profile {
            name: "alice".into(),
            credentials: Credentials {
                api_key: "SECRET_KEY_123".into(),
                public_id: "pub-001".into(),
            },
        };
        let diff = DataDiff::create_only(&p);
        let after = diff.after().expect("after").clone();

        // 巢狀路徑 /credentials/api_key 被 redact
        assert_eq!(after["credentials"]["api_key"], json!(REDACTED));
        // 同層的非敏感欄位不動
        assert_eq!(after["credentials"]["public_id"], json!("pub-001"));
        // top-level 欄位不動
        assert_eq!(after["name"], json!("alice"));
    }

    #[test]
    fn redact_array_element_by_index() {
        let s = SessionList {
            user_id: 1,
            sessions: vec![
                Session {
                    id: "s1".into(),
                    token: "TOKEN_AAA".into(),
                },
                Session {
                    id: "s2".into(),
                    token: "TOKEN_BBB".into(),
                },
            ],
        };
        let diff = DataDiff::create_only(&s);
        let after = diff.after().expect("after").clone();

        // 只 redact index 0 的 token
        assert_eq!(after["sessions"][0]["token"], json!(REDACTED));
        assert_eq!(after["sessions"][0]["id"], json!("s1"));
        // index 1 的 token 不受影響
        assert_eq!(after["sessions"][1]["token"], json!("TOKEN_BBB"));
    }

    #[derive(Serialize)]
    struct TypoRedact {
        name: String,
    }

    impl AuditRedact for TypoRedact {
        fn redacted_fields() -> &'static [&'static str] {
            &["password_hash_typo"] // 刻意打錯的欄位
        }
    }

    #[test]
    fn redact_path_not_found_is_noop() {
        // 設計：路徑不存在時不報錯、不變動 value
        // 保守設計 — 避免因 refactor 打錯欄位名讓 audit 爆掉
        let t = TypoRedact {
            name: "alice".into(),
        };
        let diff = DataDiff::create_only(&t);
        let after = diff.after().expect("after").clone();
        assert_eq!(after["name"], json!("alice"));
        assert!(after
            .as_object()
            .expect("object")
            .get("password_hash_typo")
            .is_none());
    }

    #[test]
    fn empty_both_sides_returns_empty_diff() {
        let diff = DataDiff::compute::<Animal>(None, None);
        assert!(diff.before().is_none());
        assert!(diff.after().is_none());
        assert!(diff.changed_fields().is_empty());
    }

    #[test]
    fn into_parts_consumes_and_returns_triple() {
        let a = Animal {
            id: 1,
            ear_tag: "pig1".into(),
            weight: 25.0,
            pen: "A1".into(),
        };
        let diff = DataDiff::create_only(&a);
        let (before, after, changed) = diff.into_parts();
        assert!(before.is_none());
        assert!(after.is_some());
        assert_eq!(changed, vec!["*".to_string()]);
    }

    // NOTE: 封裝保證由 `pub(crate)` fields 實現。
    // 模組外（例如 handlers/）不能用 struct literal 建構 DataDiff — 編譯會擋。
    // 這個規則無法寫成 runtime test（同模組 tests 當然可以存取），靠 code review 與 pub(crate) 關鍵字保護。
}
