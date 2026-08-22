//! 整合測試的資料庫隔離護欄（fail-closed）。
//!
//! # 為什麼「要求 `TEST_DATABASE_URL` 存在」還不夠
//!
//! 本分支先前的修正把 fallback 到 `DATABASE_URL` 拿掉了，解決的是「忘記指定」——
//! 開發機（這台同時是 prod）的 `DATABASE_URL` 指向正式庫，沒有 fallback 就不會誤入。
//!
//! 但它只檢查「有沒有這個變數」，**不檢查變數指向哪裡**。而在這台機器上最順手的
//! 寫法恰好是把 `DATABASE_URL` 的值抄過去（`TEST_DATABASE_URL=$DATABASE_URL`），
//! 抄了就繞過整道防線：整合測試會對正式庫跑 `sqlx::migrate!`、寫入 fixture，
//! 而且**沒有 teardown**，污染正式表與稽核鏈（CodeRabbit #37 兩則 Major）。
//!
//! # 判斷依據以資料庫本身為主，名稱只在內容無從判斷時當第二層
//!
//! 主要判斷**不靠名字**（不看 DSN 是否含 `test`、也不比對資料庫名）：名稱是
//! 猜測，正式庫改名或多出一個正式庫，靠名字的防線就靜默失效。這裡改看實際狀態：
//!
//! 1. 資料庫有[`MARKER_TABLE`]標記表，**且其中記的資料庫身分就是這一顆**
//!    → 曾被本護欄認定為丟棄用，放行。身分對不上（標記被 dump 還原到別顆庫、
//!    或有人手動建了一張同名空表）一律視同沒有標記，往下走完整檢查。
//! 2. 沒有有效標記，且[`PROBE_TABLES`]任一張已有資料 → **中止**，不連不寫。
//! 3. 沒有標記且探測表全空 → 名稱檢查（見下）過關後蓋上標記放行。
//! 4. 沒有標記且探測表不齊 → 只有在 public schema **完全沒有使用者資料表**、
//!    **且**名稱檢查過關時，才算「全新空庫」而放行；否則一律**中止**。
//!
//! 第 3 步是 bootstrap：CI 的 `backend-test` job 在 `cargo test` 之前就先跑過
//! `sqlx migrate run`，所以測試開始時資料庫已經有全部結構、卻還沒有標記。用
//! 「業務資料是否為空」而非「有沒有表」來判斷，CI 才不需要額外的蓋章步驟
//! （2026-08-07 與使用者確認採此變體，理由是少一個「CI 與護欄必須同步、忘了加
//! 就整支紅」的耦合點）。
//!
//! 第 4 步的收緊（2026-08-08）：本檔原本把「探測表不齊」直接當成全新空庫並**立刻
//! 蓋章**。那等於在還沒認出這顆庫是什麼之前就先對它寫入——DSN 若指到本機其他系統
//! 的資料庫（這台同時跑 prod 與 observability，postgres 上不只一顆庫），它會被蓋上
//! 「可丟棄」標記；而標記一旦蓋下去，**下一次執行就直接走第 1 步放行**，護欄自此對
//! 那顆庫永久失效。改成「有表但不是我們的表 → 中止」後，只有真正空無一物的庫會被
//! bootstrap。代價是「migration 只跑了一半」的測試庫會被擋下，需重建或手動蓋章——
//! 這個方向是刻意的：這支護欄的存在目的就是寧可誤擋，不可誤放。
//!
//! 第 3、4 步的名稱檢查（2026-08-09 補，見 [`assert_name_looks_like_test_db`]）：
//! 一顆「完全空無一物」的資料庫沒有任何業務資料可供內容比對——內容判斷在這裡
//! 本質上就是死角（PostgreSQL 叢集自帶的 `postgres` 維護庫即為一例，2026-08-09
//! 實測會直接落進這兩步被放行）。這個死角**只在準備蓋新章時**補一層名稱檢查，
//! 不對已通過身分比對的既有標記重查。
//!
//! # 殘留風險（明說，不假裝沒有）
//!
//! 名稱檢查本身是啟發式，會被刻意或意外取的名字繞過——例如一顆內容全空的正式庫
//! 若被命名為含 `test` 的名字（codeant-ai 於 2026-08-09 指出），仍會通過檢查並
//! 被永久授信。要完全堵死這個死角，需要改成「明確要求呼叫端額外設一個不可能被
//! 名字意外滿足的旗標」（例如 `I_CONFIRM_THIS_IS_A_DISPOSABLE_TEST_DB=1`），
//! 但那是使用者 2026-08-08 已經考慮過、選擇不採用的更重方案（討論中的 B2；
//! 目前採用的是較輕量的 B1 + 名稱檢查）。此殘留風險是那個決定的已知代價，
//! 不是本次疏漏。

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// 蓋在「可丟棄的測試資料庫」上的標記表。
///
/// 命名帶雙下底線前綴，明確表示非業務結構；`sqlx migrate` 只追蹤
/// `_sqlx_migrations`，多這張表不影響 migration。
///
/// # 為什麼帶 `_v2`，以及為什麼標記裡要記身分
///
/// v1 只看「這張表存不存在」就放行。那讓標記變成一張**可攜帶、可偽造、且永不
/// 重驗的通行證**（codeant-ai 於 PR #37 指出，Critical）：
///
/// - 把測試庫 `pg_dump` 還原到正式庫，標記表跟著過去，正式庫從此被當成丟棄庫；
/// - 任何人在某顆庫手動建一張同名空表，就能讓護欄對它永久放行。
///
/// v2 在標記裡寫入**蓋章當下那顆資料庫的身分**（名稱 + OID + 叢集識別碼），
/// 讀取時逐項比對；對不上就當作沒有標記，回到完整的探測表檢查。上述兩種情境
/// 都會在比對這一關被擋下（還原過去的標記記的是原本那顆庫的名字/OID，手建的
/// 空表則根本沒有那一列）。
///
/// 換表名而不是改欄位：v1 的標記本來就不該再有效力，換名字讓舊標記自動失效，
/// 也免掉「同一張表要同時支援兩種結構」的升級邏輯。舊表留在測試庫裡無害。
const MARKER_TABLE: &str = "__ipig_disposable_test_db_v2";

/// 探測表：核心業務表，**任何 migration 都不會 seed**。
///
/// 2026-08-07 實測依據：跑完全部 142 支 migration 的全新庫三者皆為 0；
/// prod 為 `animals=163` / `audit_logs=244` / `protocols=38`。
///
/// 刻意**不含 `users`**——它被 migration seed 了 1 筆，會讓每個新建的測試庫
/// 都被誤判成正式庫。同理排除 `roles` / `permissions` / `pens` 等 27 張被 seed
/// 的參考表（改用反向白名單要維護那 27 筆，只要有新 migration seed 新參考表
/// 就誤擋，維護成本高得多）。
///
/// 將來若真有 migration seed 到這三張其中之一，CI 會**當場紅掉**而不是靜默
/// 放行——這是刻意選的失敗方向。
const PROBE_TABLES: [&str; 3] = ["animals", "audit_logs", "protocols"];

/// 讀 `TEST_DATABASE_URL`；未設定或為空即中止。
///
/// **不 fallback 到 `DATABASE_URL`**：開發機那條指向 prod，見 CLAUDE.md
/// 「禁止在 prod 跑 backend 整合測試」。
///
/// 空字串要一起擋：`TEST_DATABASE_URL=`（等號後留空）會讓 `std::env::var` 回
/// `Ok("")` 而不是 `Err(NotPresent)`，原本的 `expect` 放它過去，最後才在 sqlx
/// 解析 URL 時失敗——錯誤訊息退化成「連不上」，看的人不會知道真正該做什麼。
fn require_test_database_url() -> String {
    dotenvy::dotenv().ok();
    let url = std::env::var("TEST_DATABASE_URL").unwrap_or_default();
    assert!(
        !url.trim().is_empty(),
        "需設定 TEST_DATABASE_URL 指向獨立的丟棄用測試 DB（目前未設定或為空字串）；\
         禁止 fallback 到 DATABASE_URL（開發機那條指向 prod，見 CLAUDE.md）"
    );
    url
}

/// 建立連線池，並在回傳前確認目標資料庫可以安全地被測試破壞。
///
/// `max_connections` 由呼叫端指定以保留各測試原本的池大小
/// （`PgPool::connect` 的 sqlx 預設是 10）。
pub async fn connect_disposable(max_connections: u32) -> PgPool {
    let url = require_test_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&url)
        .await
        .expect("connect test db");
    assert_disposable(&pool).await;
    pool
}

/// R97-1：seed 一位「不是建單者」的使用者，回傳其 id。
///
/// ADJ / GRN 的核准自 R97-1 起禁止自核
/// （`services/document/workflow.rs::assert_document_sod`），因此測試在
/// **建立庫存前置狀態**時不能再沿用建單者核准，否則會拿到
/// `Forbidden("職務分離：...")`。各 ERP 測試共用本 helper 取得第二位核准人，
/// 免得每個檔案各自複製一份 users insert。
///
/// `label` 只影響 email/顯示名稱，方便測試失敗時辨認來源檔。
///
/// `#[allow(dead_code)]`：本檔以 `#[path]` 被多支測試 include，
/// 沒用到本函式的那些會報 dead_code。
#[allow(dead_code)]
pub async fn seed_other_user(pool: &PgPool, label: &str) -> uuid::Uuid {
    let id = uuid::Uuid::new_v4();
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, is_active) \
         VALUES ($1, $2, $3, 'x', true)",
    )
    .bind(id)
    .bind(format!("{label}-{}@example.com", &suffix[..8]))
    .bind(format!("{label} 核准人"))
    .execute(pool)
    .await
    .expect("seed second user for SoD-compliant approval");
    id
}

/// 確認 `pool` 指向的資料庫是丟棄用測試庫；不是就 panic。
///
/// 必須在 `sqlx::migrate!` 與任何寫入**之前**呼叫。**在確認之前不對目標資料庫
/// 做任何寫入**——包含蓋標記表本身。
async fn assert_disposable(pool: &PgPool) {
    if marker_matches_this_database(pool).await {
        return;
    }

    if !probe_tables_all_exist(pool).await {
        // 探測表不齊 → 沒辦法用「業務資料是否為空」判斷這是什麼庫。
        // 只有 public schema 完全沒有使用者資料表時，才確定它是全新空庫。
        let existing_tables = user_table_count(pool).await;
        assert!(
            existing_tables == 0,
            "拒絕在資料庫 `{}` 執行整合測試：它沒有測試庫標記（{MARKER_TABLE}），\
             核心業務表（{}）不齊，卻已經有 {existing_tables} 張使用者資料表——\
             這不是全新的空庫，也不是本系統的測試庫，很可能是別的系統的資料庫。\
             請把 TEST_DATABASE_URL 指向獨立的丟棄用 DB（或先 dropdb 重建）。",
            current_database(pool).await,
            PROBE_TABLES.join(", "),
        );
        // 探測表不齊 + 使用者資料表為 0，內容判斷到此為止——一顆真正空無一物的
        // 資料庫沒有任何業務資料可供比對。這正是內容判斷本身的死角：prod cluster
        // 自帶的 `postgres` 維護庫、或任何從未被使用過的資料庫，都會落在這裡，
        // 且沒有任何內容訊號能把它跟真正的丟棄用測試庫區分開來。名稱是這裡
        // 唯一剩下的訊號（2026-08-08 使用者裁定：B1 內容判斷 + 名稱檢查當第二層）。
        assert_name_looks_like_test_db(&current_database(pool).await);
        stamp_marker(pool).await;
        return;
    }

    let populated = populated_probe_tables(pool).await;
    if !populated.is_empty() {
        // 只印資料庫名與筆數，不印完整 DSN——後者含密碼，會進 CI log。
        panic!(
            "拒絕在資料庫 `{}` 執行整合測試：它沒有測試庫標記（{MARKER_TABLE}），\
             且核心業務表已有資料（{}）。整合測試會跑 migration 並寫入 fixture 且不清理，\
             這看起來是正式資料庫。請把 TEST_DATABASE_URL 指向獨立的丟棄用 DB。",
            current_database(pool).await,
            populated.join(", ")
        );
    }

    // 與上面同理：探測表齊全但全空，同樣是「內容判斷說不出個所以然」的情況
    // （這正是 CI 全新 migrate 完的庫會落入的分支——見 PROBE_TABLES 上的說明），
    // 一樣要求名稱過關才准蓋章。
    assert_name_looks_like_test_db(&current_database(pool).await);
    stamp_marker(pool).await;
}

/// 名稱檢查：內容判斷之外的**第二層**防線（使用者 2026-08-08 裁定 B1 + 此層）。
///
/// # 為什麼身分綁定的標記解決不了這個
///
/// [`marker_matches_this_database`] 防的是「標記被偽造或搬運」——一旦某顆庫
/// 被合法蓋過章，它解決不了「這是一顆從未被本護欄看過、但根本不是測試庫」的
/// 情境。2026-08-09 實測：prod cluster 自帶的 `postgres` 維護庫在真實環境下
/// 通常完全空無一物（0 張使用者資料表），會直接落進上面兩個「內容全空即蓋章」
/// 的分支——這種庫沒有任何業務資料可供比對，內容判斷在這裡完全派不上用場。
///
/// # 為什麼刻意保守
///
/// 名稱判準本身是啟發式（改名或多一顆同名庫就可能失效），所以只做兩件事、
/// 不做更多：
/// - 明確等於已知 prod 資料庫名 → 直接拒絕（就算它意外也含 "test" 字樣）
/// - 資料庫名不含 "test" → 拒絕
///
/// 這一層只在**準備蓋新章**時查——已通過身分比對而確認過的既有標記不會重查，
/// 那是舊授信的延續，不是新授信的來源，不该因為之後改名字就失效。
fn assert_name_looks_like_test_db(db_name: &str) {
    // 已知的正式資料庫名稱。目前只有一顆——`docker-compose.yml` 的
    // `POSTGRES_DB=ipig_db`。放常數陣列而非單一 &str，方便日後有多顆 prod
    // 資料庫時擴充，不必改呼叫端。
    const KNOWN_PROD_DB_NAMES: [&str; 1] = ["ipig_db"];

    assert!(
        !KNOWN_PROD_DB_NAMES.contains(&db_name),
        "拒絕在資料庫 `{db_name}` 執行整合測試：這是已知的正式資料庫名稱，\
         即使它目前看起來空無一物也不可信任——名稱本身就是紅線。"
    );
    assert!(
        db_name.contains("test"),
        "拒絕在資料庫 `{db_name}` 執行整合測試：它沒有任何業務資料可供內容判斷\
         （核心業務表為空或不存在），資料庫名稱又看不出是測試庫（需含 test）。\
         這是內容判斷本身的死角——一顆真正空無一物的資料庫（例如 PostgreSQL \
         叢集自帶的 `postgres` 維護庫）沒有任何內容訊號能把它跟丟棄用測試庫\
         區分開來，名稱是唯一剩下的防線。若這確實是丟棄用資料庫，請改名後重試。"
    );
}

/// 目前連線的資料庫名（只用於錯誤訊息；DSN 含密碼，一律不印）。
async fn current_database(pool: &PgPool) -> String {
    sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await
        .expect("query current_database")
}

/// public 以外的系統 schema 不算；回傳這顆庫裡「屬於某個系統」的資料表張數。
///
/// 用 `pg_tables` 而非 `information_schema.tables`：後者只列出目前角色有權限看到的
/// 表，權限不足時會回 0 而讓「有表」被誤判成「空庫」——那正好是要擋的方向。
async fn user_table_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM pg_catalog.pg_tables \
         WHERE schemaname NOT IN ('pg_catalog', 'information_schema')",
    )
    .fetch_one(pool)
    .await
    .expect("count user tables")
}

/// 蓋章當下那顆資料庫的身分。三項都是「跟著資料庫走」而不是「跟著檔案走」的值，
/// 所以標記表被 dump / 還原到別顆庫時對不上。
struct DatabaseIdentity {
    name: String,
    oid: i64,
    /// 叢集識別碼（`initdb` 時產生，跨叢集必不同）。
    ///
    /// `pg_control_system()` 在部分部署會被 REVOKE，取不到就是 `None`——此時降級
    /// 成只比對名稱與 OID，不讓護欄因為權限差異整支跑不起來。2026-08-08 在本專案
    /// 的 postgres:16 測試容器實測：一般角色可讀。
    cluster: Option<String>,
}

async fn database_identity(pool: &PgPool) -> DatabaseIdentity {
    let (name, oid): (String, i64) = sqlx::query_as(
        "SELECT current_database()::text, \
                (SELECT oid::bigint FROM pg_catalog.pg_database WHERE datname = current_database())",
    )
    .fetch_one(pool)
    .await
    .expect("query database identity");

    let cluster = sqlx::query_scalar("SELECT system_identifier::text FROM pg_control_system()")
        .fetch_one(pool)
        .await
        .ok();

    DatabaseIdentity { name, oid, cluster }
}

/// 標記是否存在**且屬於目前這顆資料庫**。
///
/// 對不上就回 `false`——呼叫端會因此走完整的探測表檢查，是 fail-closed 的方向。
async fn marker_matches_this_database(pool: &PgPool) -> bool {
    let stamped = match sqlx::query_as::<_, (String, i64, Option<String>)>(
        "SELECT db_name, db_oid, cluster_id FROM public.__ipig_disposable_test_db_v2 LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    {
        Ok(row) => row,
        // 42P01 = relation 不存在＝還沒蓋過章，正常情形。
        // 其他讀取錯誤不吞：靜默降級會讓護欄的失效無人察覺。
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("42P01") => None,
        Err(e) => panic!("讀取測試庫標記表 {MARKER_TABLE} 失敗：{e}"),
    };

    // 表在但沒有那一列（例如有人手動建了一張同名空表）＝不是本護欄蓋的章。
    let Some((stamped_name, stamped_oid, stamped_cluster)) = stamped else {
        return false;
    };

    let here = database_identity(pool).await;
    if stamped_name != here.name || stamped_oid != here.oid {
        return false;
    }
    match (stamped_cluster.as_deref(), here.cluster.as_deref()) {
        (Some(stamped), Some(current)) => stamped == current,
        // 任一端取不到叢集識別碼就無從比對，此時以名稱 + OID 為準。
        _ => true,
    }
}

async fn probe_tables_all_exist(pool: &PgPool) -> bool {
    for table in PROBE_TABLES {
        let oid: Option<String> = sqlx::query_scalar("SELECT to_regclass($1)::text")
            .bind(format!("public.{table}"))
            .fetch_one(pool)
            .await
            .expect("probe business table existence");
        if oid.is_none() {
            return false;
        }
    }
    true
}

/// 回傳「有資料」的探測表描述（例：`animals=163`）；全空時為空 vec。
///
/// 三條查詢刻意寫死、不用 `format!` 組 SQL：表名雖來自本檔常數（非外部輸入），
/// 但靜態 SQL 更好稽核，也不會誤觸 SQL 注入的靜態掃描。
async fn populated_probe_tables(pool: &PgPool) -> Vec<String> {
    let counts: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM public.animals), \
                (SELECT count(*) FROM public.audit_logs), \
                (SELECT count(*) FROM public.protocols)",
    )
    .fetch_one(pool)
    .await
    .expect("count probe tables");

    let mut populated = Vec::new();
    for (name, n) in PROBE_TABLES.iter().zip([counts.0, counts.1, counts.2]) {
        if n > 0 {
            populated.push(format!("{name}={n}"));
        }
    }
    populated
}

/// 蓋上「這是可丟棄的測試資料庫」標記。
///
/// 那一列同時是**核准憑證**與除錯資訊：`db_name` / `db_oid` / `cluster_id` 由
/// [`marker_matches_this_database`] 逐項比對，`stamped_at` 給人看「這顆庫何時被
/// 認定為丟棄庫」。**必須真的寫進去**——只建表不插列，下次讀不到那一列就等於沒
/// 蓋章（2026-08-07 實測抓到「只建表」的版本）。
///
/// # `IF NOT EXISTS` 不等於沒有競態
///
/// 本檔原註解宣稱靠 `IF NOT EXISTS` / `WHERE NOT EXISTS` 就能避免競態，**那是錯的**。
/// Postgres 的 `CREATE TABLE IF NOT EXISTS` 不是原子操作：兩個連線同時執行時，落後的
/// 那個可能在建 relation 的型別時撞上 `pg_type_typname_nsp_index` 的 unique violation
/// （23505），而不是靜默略過；也可能回 42P07（duplicate table）或 **42710
/// （duplicate_object，訊息是 `type "…" already exists`）**。
///
/// 42710 是 2026-08-09 本機平行跑 `notification_kind_split` 時實際撞到的那個
/// ——原本只容忍 23505 / 42P07，這個漏掉的 SQLSTATE 讓測試間歇性紅。多個測試 binary 是
/// 循序跑的，但**同一個 binary 內的平行測試**會同時走到這裡，CI 目前靠
/// `--test-threads=1` 掩蓋，本機平行跑就會間歇紅。
///
/// 這兩個 SQLSTATE 的語意都是「別人剛好先建好了」，結果與我們想要的一致，故視為成功。
async fn stamp_marker(pool: &PgPool) {
    let here = database_identity(pool).await;

    if let Err(e) = sqlx::query(
        "CREATE TABLE IF NOT EXISTS public.__ipig_disposable_test_db_v2 (\
             db_name    text        NOT NULL,\
             db_oid     bigint      NOT NULL,\
             cluster_id text,\
             stamped_at timestamptz NOT NULL DEFAULT now()\
         )",
    )
    .execute(pool)
    .await
    {
        let created_concurrently = e
            .as_database_error()
            .and_then(|db| db.code())
            .is_some_and(|code| matches!(code.as_ref(), "23505" | "42P07" | "42710"));
        assert!(
            created_concurrently,
            "無法在測試資料庫 `{}` 建立標記表 {MARKER_TABLE}：{e}\n\
             若這是權限錯誤（42501）：PostgreSQL 15 起 public schema 預設不開放非 owner \
             CREATE，請改用該資料庫的 owner 帳號連線，或先手動建好這張表。",
            current_database(pool).await,
        );
    }

    sqlx::query(
        "INSERT INTO public.__ipig_disposable_test_db_v2 (db_name, db_oid, cluster_id) \
         SELECT $1, $2, $3 \
         WHERE NOT EXISTS (SELECT 1 FROM public.__ipig_disposable_test_db_v2)",
    )
    .bind(&here.name)
    .bind(here.oid)
    .bind(here.cluster.as_deref())
    .execute(pool)
    .await
    .expect("record disposable test db identity");
}
