//! 共用查詢（shared queries）：**≥2 處重複的 SQL 抽取點，不是資料層邊界**。
//!
//! ⚠️ 名字叫 `repositories` 容易讓人以為有一層可依賴的資料層抽象，實際沒有——
//! 本目錄只包了約 156 個查詢，而 `services/` 內另有約 **1,491 處 `sqlx` 呼叫**
//! （2026-08-11 實測），亦即**約九成的 SQL 不經過這裡**。service 直接寫 SQL 是本專案的常態，
//! 不是待修的違規；請勿因為「應該走 repository」而把既有查詢搬過來。
//!
//! 什麼時候該在這裡新增函式：同一段查詢在 **2 處以上**重複出現、且抽出來能讓呼叫端不必
//! 重複知道表名與欄位條件時。只被單一 service 用到的查詢留在該 service 內即可——
//! 搬過來只會讓介面變寬（多一個要維護的公開函式）而沒有換到任何抽象。
//!
//! **採用共用查詢時**的依賴方向：Services → Repositories → Models
//! （不是「所有 service 都必須經過這裡」——見上方定位說明）
//!
//! 相關背景：模組深度盤點 `docs/design/module-depth/PLAN.md`（R94-2 依使用者裁定採
//! 「保留目錄名、改 doc 寫清定位」，未改名為 `queries/`，以免動到 38 支檔的 import——
//! 2026-08-11 第五輪訂正，原記 39 誤含一筆日誌字串，口徑見 PLAN.md）。

pub mod ai;
pub mod equipment;

pub use ai::AiRepository;
pub mod accounting;
pub mod application_notice;
pub mod audit_log;
pub mod data_retention;
pub mod glp_compliance;
pub mod hr;
pub mod notification;
pub mod pen;
pub mod pending_owner;
pub mod product;
pub mod qa_plan;
pub mod role;
pub mod sku;
pub mod user;
pub mod user_preference;
pub mod warehouse;
