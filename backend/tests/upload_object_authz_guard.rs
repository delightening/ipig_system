// R110（2026-08-24）：`upload.rs` 的每個上傳 handler 都必須做物件層授權，
// 不能只靠 `require_permission!` 的全域功能權限——那只回答「有沒有上傳這類檔案的資格」，
// 不回答「這個 observation_id / animal_id 是不是你能碰的」。
//
// 沿用 `admin_authz_guard.rs` 的結構掃描慣例：這不是跑真實 HTTP 請求，是靜態掃原始碼，
// 鎖定「物件層檢查存在，且在任何寫入動作（handle_upload / FileService::upload）之前執行」。
// 好處是新增 handler 或有人手滑刪掉那行檢查時，這支測試會直接紅燈，不必等到滲透測試或事故才發現。
//
// 反向驗證見 `guard_detects_missing_check_and_wrong_order`：確認這個掃描邏輯本身
// 在「有事」的情況下真的會叫——不是只在乾淨檔案上跑過一次就交差。

use std::path::Path;

/// 一個 handler 的驗收規格：函式名、必須出現的物件層檢查（regex 片段）、
/// 必須早於哪個「寫入動作」標記。
struct Spec {
    handler: &'static str,
    object_check: &'static str,
    write_marker: &'static str,
}

const SPECS: &[Spec] = &[
    Spec {
        handler: "upload_animal_photo",
        object_check: "require_animal_access",
        write_marker: "handle_upload(",
    },
    Spec {
        handler: "upload_pathology_report",
        object_check: "require_animal_access",
        write_marker: "handle_upload(",
    },
    Spec {
        handler: "upload_sacrifice_photo",
        object_check: "require_animal_access",
        write_marker: "FileService::upload(",
    },
    Spec {
        handler: "upload_observation_attachment",
        object_check: "Scoped::<access::AnimalWrite>::from_observation",
        write_marker: "handle_upload(",
    },
    Spec {
        handler: "upload_protocol_attachment",
        object_check: "require_protocol_edit",
        write_marker: "handle_upload(",
    },
];

/// 全域資源或無跨對象風險的 handler：不要求物件層檢查，但要求原始碼裡有結論註解，
/// 避免下次稽核又把它列成可疑項（R110-3 的教訓——沒有結案理由的「看起來像漏檢」
/// 會一輪一輪被重新懷疑）。
const EXEMPT_WITH_RATIONALE: &[(&str, &str)] = &[
    ("upload_sop_document", "R110-3"),
    // ⚠️ 不要挑會被原始碼裡的 markdown backtick 截斷的片段——原文是
    // 「`entity_id` 寫死 `current_user.id.to_string()`」，backtick 緊接在
    // entity_id 後面，"entity_id 寫死 current_user.id"（不含 backtick）不是
    // 它的子字串，會讓這個豁免永遠通不過（2026-08-24 實際踩到，靠反向驗證
    // 抓到）。改挑 backtick 內部、不會被截斷的片段。
    ("upload_leave_attachment", "current_user.id.to_string()"),
];

#[test]
fn upload_handlers_have_object_level_authz_before_write() {
    let src = read_upload_rs();
    let mut violations = Vec::new();

    for spec in SPECS {
        let body = extract_fn_body(&src, spec.handler).unwrap_or_else(|| {
            panic!(
                "guard: 找不到 handler {}（可能被改名，測試需要同步更新）",
                spec.handler
            )
        });

        let check_pos = body.find(spec.object_check);
        let write_pos = body.find(spec.write_marker);

        match (check_pos, write_pos) {
            (None, _) => violations.push(format!(
                "  {} — 缺少物件層授權（找不到 `{}`）",
                spec.handler, spec.object_check
            )),
            (Some(_), None) => violations.push(format!(
                "  {} — 找不到寫入標記 `{}`，測試規格需要更新（handler 實作可能已改變）",
                spec.handler, spec.write_marker
            )),
            (Some(c), Some(w)) if c > w => violations.push(format!(
                "  {} — 物件層授權出現在寫入動作 `{}` 之後，等於沒擋（先寫了才檢查）",
                spec.handler, spec.write_marker
            )),
            _ => {}
        }
    }

    for (handler, rationale_marker) in EXEMPT_WITH_RATIONALE {
        let body = extract_fn_doc_and_body(&src, handler)
            .unwrap_or_else(|| panic!("guard: 找不到 handler {}", handler));
        if !body.contains(rationale_marker) {
            violations.push(format!(
                "  {handler} — 屬於免物件層授權白名單，但找不到結論理由 `{rationale_marker}` 的註解，\
                 無法確認豁免仍然成立（可能被改動過卻沒同步更新註解）"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n{} 個上傳 handler 未通過物件層授權掃描：\n{}\n\n\
         修法：在寫入動作之前加上對應的物件層檢查（`access::require_animal_access` /\n\
         `access::Scoped::<access::AnimalWrite>::from_observation` / `access::require_protocol_edit`），\n\
         或若確認該端點不需要物件層授權，加入 EXEMPT_WITH_RATIONALE 並在原始碼寫明理由。\n",
        violations.len(),
        violations.join("\n")
    );
}

/// 反向驗證：確認上面的掃描邏輯在「有事」時真的會叫，不是走個形式。
/// 對著人為構造的「有漏洞」原始碼跑同一套抽取/比對邏輯，三種壞情況都必須被抓到。
#[test]
fn guard_detects_missing_check_and_wrong_order() {
    let missing = "pub async fn upload_animal_photo(x: i32) -> i32 {\n    require_permission!(x);\n    handle_upload(x)\n}\n";
    let body = extract_fn_body(missing, "upload_animal_photo").expect("測試 fixture 應能被抽取");
    assert!(
        body.find("require_animal_access").is_none(),
        "guard 邏輯壞了：缺少檢查的 fixture 卻被判定為有檢查"
    );

    let wrong_order = "pub async fn upload_animal_photo(x: i32) -> i32 {\n    require_permission!(x);\n    handle_upload(x);\n    access::require_animal_access(x)\n}\n";
    let body =
        extract_fn_body(wrong_order, "upload_animal_photo").expect("測試 fixture 應能被抽取");
    let c = body
        .find("require_animal_access")
        .expect("fixture 裡確實有這段文字");
    let w = body
        .find("handle_upload(")
        .expect("fixture 裡確實有這段文字");
    assert!(
        c > w,
        "guard 邏輯壞了：檢查在寫入動作之後（順序錯誤）的 fixture，卻沒被判定為順序錯誤"
    );

    let correct = "pub async fn upload_animal_photo(x: i32) -> i32 {\n    require_permission!(x);\n    access::require_animal_access(x);\n    handle_upload(x)\n}\n";
    let body = extract_fn_body(correct, "upload_animal_photo").expect("測試 fixture 應能被抽取");
    let c = body
        .find("require_animal_access")
        .expect("fixture 裡確實有這段文字");
    let w = body
        .find("handle_upload(")
        .expect("fixture 裡確實有這段文字");
    assert!(
        c < w,
        "guard 邏輯壞了：順序正確的 fixture 卻被判定為順序錯誤"
    );
}

fn read_upload_rs() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/handlers/upload.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("guard: 無法讀取 {}: {e}", path.display()))
}

/// 抓「pub async fn <name>(」到下一個「pub async fn 」（或檔尾）之間的原始碼片段。
/// 不含前面的 doc comment，只含函式簽章與函式本體——避免 doc comment 裡的說明文字
/// 被誤判成「真的有呼叫這個檢查」（這正是 PR #7 那次踩過的坑：註解裡出現同樣的字串）。
fn extract_fn_body(src: &str, fn_name: &str) -> Option<String> {
    let marker = format!("pub async fn {fn_name}(");
    let start = src.find(&marker)?;
    let rest = &src[start..];
    let end = rest[marker.len()..]
        .find("\npub async fn ")
        .map(|i| i + marker.len())
        .unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

/// 抓函式的 doc comment 加函式本體（用於檢查豁免理由是否寫在 doc comment 裡）。
fn extract_fn_doc_and_body(src: &str, fn_name: &str) -> Option<String> {
    let marker = format!("pub async fn {fn_name}(");
    let fn_start = src.find(&marker)?;
    // 往前找 doc comment 的起點：往回找到不是 `///` 開頭的那一行為止
    // ⚠️ 只在連續的 `///` 行往回找，遇到空行就停——本檔案裡 doc comment 與
    // `pub async fn` 之間沒有空行，若把空行也當延續，會一路吃到上一個函式的
    // 本體，把上一個 handler 的內容誤判成這個 handler 的豁免理由。
    let before = &src[..fn_start];
    let lines: Vec<&str> = before.lines().collect();
    let mut doc_start_line = lines.len();
    for (i, l) in lines.iter().enumerate().rev() {
        if l.trim_start().starts_with("///") {
            doc_start_line = i;
        } else {
            break;
        }
    }
    let doc = lines[doc_start_line..].join("\n");
    let body = extract_fn_body(src, fn_name)?;
    Some(format!("{doc}\n{body}"))
}
