//! R97-1c：DIRECTOR（負責人）的 ERP 授權邊界。
//!
//! ## 為什麼要有這支測試
//!
//! 使用者 2026-08-17 訂下的原則：
//!
//! > **負責人是監督與終審，不是操作者。不建單、不改單、不送審。**
//! > 除緊急狀況外不碰第一線。
//!
//! 這條原則若只寫在 `startup/permissions.rs` 的清單與註解裡，日後很容易被
//! 「順手多加一個權限」侵蝕——而侵蝕的當下不會有任何測試變紅。本檔把邊界
//! 兩側都鎖住：該有的必須有、**不該有的必須沒有**。
//!
//! 後者才是重點。多數權限測試只驗「有沒有拿到」，於是「不小心多給」永遠不會被發現。
//!
//! ## 這條原則與職務分離的關係
//!
//! R97-1 規定「建立者不得自核」。只要負責人從不建單，他就永遠有資格核准任何單據，
//! 不會出現「自己審自己而卡住」的死結。**不給建單權不只是潔癖，是讓終審關始終可用。**

mod common;
use common::TestApp;
use serial_test::serial;

/// 負責人**必須**具備的 ERP 權限，以及每一項的存在理由。
///
/// 少了任一項，終審流程會在某處斷掉——理由寫在這裡，日後有人想拿掉時看得到代價。
const REQUIRED: &[(&str, &str)] = &[
    ("erp.document.view", "看不到單據內容就無法審"),
    (
        "erp.document.view_all",
        "負責人不建單，少了它會什麼都看不到——一般 view 只涵蓋自己建立的單據",
    ),
    (
        "erp.stock.view",
        "審「調減 200 個」時要知道現在剩多少，否則終審只是蓋章",
    ),
    ("erp.document.final_approve", "大額調整單的終審核准／駁回"),
    ("erp.document.reverse_approve", "沖銷單核准"),
];

/// 負責人**不得**具備的權限：全部是「第一線操作」。
const FORBIDDEN: &[(&str, &str)] = &[
    ("erp.document.create", "建單是倉管的事"),
    ("erp.document.edit", "改單是倉管的事"),
    ("erp.document.submit", "送審是倉管的事"),
    ("erp.document.delete", "刪單是第一線操作"),
    ("erp.document.cancel", "作廢是第一線操作"),
    (
        "erp.document.approve",
        "倉管階段的核准——負責人若同時有它，等於一人可走完兩關",
    ),
    ("erp.warehouse.create", "倉庫主檔維護屬倉管職責"),
    ("erp.warehouse.edit", "同上"),
    ("erp.product.create", "產品主檔維護屬倉管職責"),
    ("erp.product.edit", "同上"),
    ("erp.storage.create", "儲位維護屬倉管職責"),
    ("erp.stock.adjust", "直接調整庫存是第一線操作"),
    ("erp.stocktake.create", "開盤點單是第一線操作"),
];

async fn director_erp_permissions(app: &TestApp) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT p.code FROM roles r \
         JOIN role_permissions rp ON rp.role_id = r.id \
         JOIN permissions p ON p.id = rp.permission_id \
         WHERE r.code = 'DIRECTOR' AND p.code LIKE 'erp.%' ORDER BY p.code",
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("query DIRECTOR erp permissions")
}

#[tokio::test]
#[serial]
async fn director_has_every_permission_needed_to_finalize() {
    let app = TestApp::spawn().await;
    let granted = director_erp_permissions(&app).await;

    for (code, why) in REQUIRED {
        assert!(
            granted.iter().any(|g| g == code),
            "DIRECTOR 缺少 `{code}`：{why}。\n實際擁有：{granted:?}"
        );
    }
}

#[tokio::test]
#[serial]
async fn director_cannot_operate_the_front_line() {
    let app = TestApp::spawn().await;
    let granted = director_erp_permissions(&app).await;

    for (code, why) in FORBIDDEN {
        assert!(
            !granted.iter().any(|g| g == code),
            "DIRECTOR 不應具備 `{code}`：{why}。\n\
             使用者裁定：負責人是監督與終審，不是操作者。\n\
             若確實要放寬，請一併修改本測試並在 PR 說明理由。"
        );
    }
}

/// 邊界的完整性：DIRECTOR 的 ERP 權限**恰好**是 REQUIRED，沒有多的。
///
/// 前兩項測試各自只看單邊；這一項防的是「加了一個既不在 REQUIRED、也不在
/// FORBIDDEN 名單裡的新權限」——那種漏網之魚不會被前兩項發現。
#[tokio::test]
#[serial]
async fn director_erp_permissions_are_exactly_the_documented_set() {
    let app = TestApp::spawn().await;
    let mut granted = director_erp_permissions(&app).await;
    granted.sort();

    let mut expected: Vec<String> = REQUIRED.iter().map(|(c, _)| c.to_string()).collect();
    expected.sort();

    assert_eq!(
        granted, expected,
        "DIRECTOR 的 ERP 權限與文件化清單不符。\n\
         新增或移除權限時請同步更新本檔的 REQUIRED，讓「負責人能做什麼」\n\
         始終有一份可讀、可驗證的來源（換負責人時據此交接）。"
    );
}
