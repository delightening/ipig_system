use sqlx::PgPool;
use uuid::Uuid;

use super::super::utils::AnimalUtils;
use super::super::AnimalService;
use crate::{
    models::{
        Animal, AnimalBreed, AnimalClientInfoResponse, AnimalListItem, AnimalQuery,
        AnimalStatsResponse, AnimalsByPen, AvailablePigListResponse, AvailablePigQuery,
        AvailablePigRow, AvailablePigSummary, PaginatedResponse,
        AVAILABLE_PIG_WEIGHT_FRESHNESS_DAYS, BREEDING_PROTOCOL_NO,
    },
    services::access,
    AppError, Result,
};

/// R35-5: server-side sort 欄位白名單 — (`AnimalQuery::sort_by` 字串, SQL 表達式)。
/// 用白名單避免 SQL injection；不在表內的請求 fallback 到預設 `p.id DESC`。
/// 對應 frontend `AnimalListTable.tsx` 的 `SortableHeader` columns。
const ANIMAL_SORT_COLUMNS: &[(&str, &str)] = &[
    ("ear_tag", "p.ear_tag"),
    ("animal_no", "p.animal_no"),
    ("pen_location", "p.pen_location"),
    ("iacuc_no", "p.iacuc_no"),
    ("entry_date", "p.entry_date"),
    ("status", "p.status"),
    ("breed", "p.breed"),
    ("created_at", "p.created_at"),
    ("latest_weight", "lw.weight"),
];

/// 解析 sort param → 安全的 ORDER BY clause（不含 `ORDER BY` 前綴）。
/// `sort_by` 不在白名單 / 為 None → return None（caller 用預設）。
fn resolve_animal_order_by(query: &AnimalQuery) -> Option<String> {
    let key = query.sort_by.as_deref()?.trim();
    let column = ANIMAL_SORT_COLUMNS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, sql)| *sql)?;
    let direction = match query.sort_order.as_deref().map(str::to_ascii_lowercase) {
        Some(ref s) if s == "asc" => "ASC",
        _ => "DESC",
    };
    // NULLS LAST：對 iacuc_no / latest_weight 等可空欄位避免空值排在前
    Some(format!("{column} {direction} NULLS LAST, p.id {direction}"))
}

impl AnimalService {
    pub(super) fn push_animal_filters(
        qb: &mut sqlx::QueryBuilder<sqlx::Postgres>,
        query: &AnimalQuery,
    ) {
        if let Some(status) = &query.status {
            qb.push(" AND p.status = ");
            qb.push_bind(*status);
        }
        if let Some(breed) = &query.breed {
            let breed_str = AnimalUtils::breed_to_db_value(breed);
            qb.push(" AND p.breed = ");
            qb.push_bind(breed_str.to_string());
            qb.push("::animal_breed");
        }
        if let Some(iacuc_no) = &query.iacuc_no {
            qb.push(" AND p.iacuc_no = ");
            qb.push_bind(iacuc_no.clone());
        }
        if let Some(keyword) = &query.keyword {
            let keyword_pattern = format!("%{}%", keyword);
            qb.push(" AND (p.ear_tag ILIKE ");
            qb.push_bind(keyword_pattern.clone());
            qb.push(" OR p.pen_location ILIKE ");
            qb.push_bind(keyword_pattern);
            qb.push(")");
        }
        if let Some(true) = query.is_on_medication {
            qb.push(
                r#"
                AND (
                    EXISTS(
                        SELECT 1 FROM animal_observations po
                        WHERE po.animal_id = p.id
                        AND po.no_medication_needed = false
                    )
                    OR
                    EXISTS(
                        SELECT 1 FROM animal_surgeries ps
                        WHERE ps.animal_id = p.id
                        AND ps.no_medication_needed = false
                    )
                )
            "#,
            );
        }
    }

    /// 取得動物狀態統計（status count + pen count + 分棟 pen count）
    ///
    /// 兩段統計必須看到**同一份資料快照**：否則動物在兩次查詢之間移欄／刪除時，
    /// 各棟加總會不等於 `pen_animals_count`。故整包放進 REPEATABLE READ transaction。
    pub async fn stats(pool: &PgPool) -> Result<AnimalStatsResponse> {
        let mut tx = pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .execute(&mut *tx)
            .await?;

        let rows: Vec<(String, i64, i64)> = sqlx::query_as(
            r#"
            SELECT
                status::text,
                COUNT(*) as count,
                COUNT(*) FILTER (
                    WHERE pen_location IS NOT NULL AND TRIM(pen_location) != ''
                    AND status NOT IN ('euthanized', 'sudden_death', 'transferred')
                ) as pen_count
            FROM animals
            WHERE deleted_at IS NULL
            GROUP BY status
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;

        let pen_counts_by_building = Self::pen_counts_by_building(&mut *tx).await?;
        tx.commit().await?;

        let mut status_counts = std::collections::HashMap::new();
        let mut total: i64 = 0;
        let mut pen_animals_count: i64 = 0;
        for (status, count, pen_count) in &rows {
            status_counts.insert(status.clone(), *count);
            total += count;
            pen_animals_count += pen_count;
        }

        Ok(AnimalStatsResponse {
            status_counts,
            pen_animals_count,
            pen_counts_by_building,
            total,
        })
    }

    /// 各棟舍的在欄動物數（key = `buildings.code`）。
    ///
    /// 動物集合與 `stats()` 的 `pen_animals_count` **完全相同**（同樣的 deleted_at /
    /// pen_location / 終止狀態條件），本查詢只多做「這隻歸屬哪一棟」的映射，
    /// 因此各棟加總 == `pen_animals_count`（除了 `pen_location` 對不到任何欄位的孤兒資料，
    /// 那種動物本來也不會出現在任何一棟的欄位視圖裡）。
    ///
    /// 為什麼從 animals 出發、而不是從 buildings LEFT JOIN 下來：
    /// - `pens.code` **沒有全域唯一性**：現有的只是部分唯一索引
    ///   `uq_pens_zone_code_active`（`(zone_id, code) WHERE is_active`），
    ///   跨 zone 以及停用的列都不受它約束（實測 `S01` 就有 active / inactive 兩筆）。
    ///   從 buildings 展開時，同一隻動物會 match 到多筆同名欄位；若那些欄位分屬不同棟，
    ///   `COUNT(DISTINCT a.id)` 只能在單一棟內去重，跨棟仍會重複計數。
    ///   （現行資料兩筆 `S01` 同屬羊舍、全庫無跨棟重複 code，故此為防禦性設計。）
    /// - `LATERAL ... LIMIT 1` 讓每隻動物只歸屬一棟，並優先採用 active 的欄位／區域，
    ///   同時避免對停用設施採用與 `stats()` 不同的過濾範圍（會使加總對不上）。
    ///
    /// 註：不能改用 `a.pen_id = p.id`——`pen_id` 尚未完全回填（實測 122 隻在欄動物中
    /// 有 13 隻為 NULL），改用 FK 會讓這些動物從分棟統計中消失。
    async fn pen_counts_by_building<'e, E>(
        executor: E,
    ) -> Result<std::collections::HashMap<String, i64>>
    where
        E: sqlx::PgExecutor<'e>,
    {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT b.code, COUNT(*) as pen_count
            FROM animals a
            JOIN LATERAL (
                SELECT z2.building_id
                FROM pens p2
                JOIN zones z2 ON z2.id = p2.zone_id
                WHERE p2.code = a.pen_location
                ORDER BY (p2.is_active AND z2.is_active) DESC, p2.id
                LIMIT 1
            ) pick ON TRUE
            JOIN buildings b ON b.id = pick.building_id
            WHERE a.deleted_at IS NULL
                AND a.pen_location IS NOT NULL AND TRIM(a.pen_location) != ''
                AND a.status NOT IN ('euthanized', 'sudden_death', 'transferred')
            GROUP BY b.code
            "#,
        )
        .fetch_all(executor)
        .await?;

        Ok(rows.into_iter().collect())
    }

    /// 取得動物列表（支援分頁）
    ///
    /// 效能（W1）：分頁採**兩段式** —— 第一段只用瘦索引取本頁 id（OFFSET 不對昂貴 enrich
    /// 空跑），第二段才對本頁少量 id 做 LATERAL / EXISTS / JOIN enrich。深分頁（第 200 頁）
    /// 由 867ms 降至 ~12ms（50k 資料實測）。詳見 docs/design/db_performance_refactor_plan.md W1。
    pub async fn list(
        pool: &PgPool,
        query: &AnimalQuery,
    ) -> Result<PaginatedResponse<AnimalListItem>> {
        let paginated = query.page.is_some() || query.per_page.is_some();
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(50).clamp(1, 200);

        let total: i64 = if paginated {
            let mut count_qb = sqlx::QueryBuilder::new(
                "SELECT COUNT(*) FROM animals p WHERE p.deleted_at IS NULL",
            );
            Self::push_animal_filters(&mut count_qb, query);
            let (cnt,): (i64,) = count_qb.build_query_as().fetch_one(pool).await?;
            cnt
        } else {
            0 // will be set from vec len below
        };

        // R35-5: server-side sort（白名單）；無/不合法時 fallback 預設 `p.id DESC`。
        // 兩段式需在兩段用同一排序 → 先解析一次共用。
        let order_by = resolve_animal_order_by(query);
        // 排序若用 latest_weight（enrich 欄位），第一段瘦查詢需帶體重 LATERAL 才能正確選頁。
        let sort_by_weight = query.sort_by.as_deref().map(str::trim) == Some("latest_weight");

        // ── 第一段：取本頁 id（僅分頁時）。filter 於此套用以決定哪些動物入頁；
        //    OFFSET 走瘦索引，不對被丟棄的列空跑 enrich。
        let page_ids: Option<Vec<Uuid>> = if paginated {
            let mut id_qb = sqlx::QueryBuilder::new("SELECT p.id FROM animals p ");
            if sort_by_weight {
                id_qb.push(
                    "LEFT JOIN LATERAL (SELECT pw.weight, pw.measure_date FROM animal_weights pw \
                     WHERE pw.animal_id = p.id ORDER BY pw.measure_date DESC LIMIT 1) lw ON true ",
                );
            }
            id_qb.push("WHERE p.deleted_at IS NULL");
            Self::push_animal_filters(&mut id_qb, query);
            match &order_by {
                Some(clause) => {
                    id_qb.push(" ORDER BY ");
                    id_qb.push(clause.as_str());
                }
                None => {
                    id_qb.push(" ORDER BY p.id DESC");
                }
            }
            let offset = (page - 1) * per_page;
            id_qb.push(" LIMIT ");
            id_qb.push_bind(per_page);
            id_qb.push(" OFFSET ");
            id_qb.push_bind(offset);
            Some(id_qb.build_query_scalar::<Uuid>().fetch_all(pool).await?)
        } else {
            None
        };

        // 本頁無資料 → 早返回，免跑 enrich。
        if let Some(ids) = &page_ids {
            if ids.is_empty() {
                return Ok(PaginatedResponse::new(Vec::new(), total, page, per_page));
            }
        }

        // ── 第二段：對本頁 id（或非分頁的全集）做完整 enrich。
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT
                p.id, p.animal_no, p.ear_tag, p.status, p.breed, p.breed_other, p.gender, p.pen_location,
                p.pen_id, p.species_id, sp.name as species_name,
                p.iacuc_no, p.entry_date, s.name as source_name,
                p.vet_last_viewed_at, p.created_at,
                EXISTS(
                    SELECT 1 FROM animal_observations po
                    WHERE po.animal_id = p.id
                    AND po.record_type = 'abnormal'::record_type
                ) as has_abnormal_record,
                (
                    EXISTS(
                        SELECT 1 FROM animal_observations po
                        WHERE po.animal_id = p.id
                        AND po.no_medication_needed = false
                    )
                    OR
                    EXISTS(
                        SELECT 1 FROM animal_surgeries ps
                        WHERE ps.animal_id = p.id
                        AND ps.no_medication_needed = false
                    )
                ) as is_on_medication,
                lw.weight as latest_weight,
                lw.measure_date as latest_weight_date,
                pt.status::text as pending_transfer_status
            FROM animals p
            LEFT JOIN animal_sources s ON p.source_id = s.id
            LEFT JOIN species sp ON p.species_id = sp.id
            LEFT JOIN LATERAL (
                SELECT pw.weight, pw.measure_date
                FROM animal_weights pw
                WHERE pw.animal_id = p.id
                ORDER BY pw.measure_date DESC
                LIMIT 1
            ) lw ON true
            -- issue #180：待轉讓不再靠 animals.status 中間態，改由未結案的 animal_transfers 列表示。
            -- `initiate_transfer` 已保證同一動物至多一筆未結案轉讓，故 LIMIT 1 不會漏顯示。
            LEFT JOIN LATERAL (
                SELECT t.status
                FROM animal_transfers t
                WHERE t.animal_id = p.id
                  AND t.status NOT IN ('completed', 'rejected')
                ORDER BY t.created_at DESC
                LIMIT 1
            ) pt ON true
            WHERE p.deleted_at IS NULL
            "#,
        );

        match &page_ids {
            Some(ids) => {
                // filter 已於第一段套用，第二段只取本頁 id（不重套 filter）。
                query_builder.push(" AND p.id = ANY(");
                query_builder.push_bind(ids.clone());
                query_builder.push("::uuid[])");
            }
            None => {
                // 非分頁：維持原行為（全集 + filter）。
                Self::push_animal_filters(&mut query_builder, query);
            }
        }
        match &order_by {
            Some(clause) => {
                query_builder.push(" ORDER BY ");
                query_builder.push(clause.as_str());
            }
            None => {
                query_builder.push(" ORDER BY p.id DESC");
            }
        }

        let mut animals: Vec<AnimalListItem> = query_builder
            .build_query_as::<AnimalListItem>()
            .fetch_all(pool)
            .await?;

        for animal in &mut animals {
            animal.ear_tag = AnimalUtils::format_ear_tag(&animal.ear_tag);
            if let Some(pen) = &animal.pen_location {
                animal.pen_location = Some(AnimalUtils::format_pen_location(pen));
            }
        }

        let actual_total = if paginated {
            total
        } else {
            animals.len() as i64
        };
        Ok(PaginatedResponse::new(
            animals,
            actual_total,
            page,
            per_page,
        ))
    }

    /// 取得使用者關聯計畫的 IACUC 編號清單
    /// 透過 user_protocols 表查詢使用者作為 PI/CO_EDITOR/CLIENT 的計畫
    pub async fn get_user_iacuc_nos(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>> {
        let iacuc_nos: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT p.iacuc_no
            FROM protocols p
            INNER JOIN user_protocols up ON up.protocol_id = p.id
            WHERE up.user_id = $1 AND p.iacuc_no IS NOT NULL
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(iacuc_nos)
    }

    /// 依欄位分組取得動物
    pub async fn list_by_pen(pool: &PgPool) -> Result<Vec<AnimalsByPen>> {
        let mut animals = sqlx::query_as::<_, AnimalListItem>(
            r#"
            SELECT
                p.id, p.animal_no, p.ear_tag, p.status, p.breed, p.breed_other, p.gender, p.pen_location,
                p.pen_id, p.species_id, sp.name as species_name,
                p.iacuc_no, p.entry_date, s.name as source_name,
                p.vet_last_viewed_at, p.created_at
            FROM animals p
            LEFT JOIN animal_sources s ON p.source_id = s.id
            LEFT JOIN species sp ON p.species_id = sp.id
            WHERE p.pen_location IS NOT NULL
            AND p.deleted_at IS NULL
            -- 只列在欄活躍動物；終態（安樂死/猝死/已轉讓）雖可能殘留 pen_location
            -- （歷史/匯入資料）但已不在欄，須排除，與 pen.current_count 重算定義一致。
            AND p.status NOT IN ('euthanized', 'sudden_death', 'transferred')
            ORDER BY p.pen_location, p.id
            "#,
        )
        .fetch_all(pool)
        .await?;

        for animal in &mut animals {
            animal.ear_tag = AnimalUtils::format_ear_tag(&animal.ear_tag);
            if let Some(pen) = &animal.pen_location {
                animal.pen_location = Some(AnimalUtils::format_pen_location(pen));
            }
        }

        let mut grouped: std::collections::HashMap<String, Vec<AnimalListItem>> =
            std::collections::HashMap::new();
        for animal in animals {
            if let Some(pen) = &animal.pen_location {
                grouped.entry(pen.clone()).or_default().push(animal);
            }
        }

        let result: Vec<AnimalsByPen> = grouped
            .into_iter()
            .map(|(pen, animals)| AnimalsByPen {
                pen_location: pen,
                animals,
            })
            .collect();

        Ok(result)
    }

    /// 取得單一動物
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Animal> {
        let mut animal = sqlx::query_as::<_, Animal>(
            r#"
            SELECT p.*,
                (SELECT u.display_name FROM users u WHERE u.id = p.experiment_assigned_by) AS experiment_assigned_by_name,
                (SELECT s.name FROM species s WHERE s.id = p.species_id) AS species_name
            FROM animals p
            WHERE p.id = $1 AND p.deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Animal not found".to_string()))?;

        animal.ear_tag = AnimalUtils::format_ear_tag(&animal.ear_tag);
        if let Some(pen) = &animal.pen_location {
            animal.pen_location = Some(AnimalUtils::format_pen_location(pen));
        }

        Ok(animal)
    }

    /// 標記動物為獸醫已讀（更新列表所讀的 `vet_last_viewed_at`；
    /// 舊版誤寫不存在的 `vet_read_at` 欄位導致 endpoint SQL 失敗）。
    pub async fn mark_vet_read(
        pool: &PgPool,
        scope: access::Scoped<access::AnimalRead>,
    ) -> Result<()> {
        let id = scope.id();
        // 僅更新「最後檢視」狀態欄位，不 bump updated_at（檢視非內容修改，避免快取失效 /
        // 冗餘 audit）；加 deleted_at IS NULL 防禦性排除軟刪除動物（成員分支授權未過濾之）。
        sqlx::query(
            "UPDATE animals SET vet_last_viewed_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// R47 — 可用豬隻快速查詢（庫存盤點）
    ///
    /// 「可用」= `Unassigned` + `Completed` 永遠列入；若 `include_breeding=true`，
    /// 額外加上 `InExperiment` 且該豬所屬 protocol 的 `protocol_no = '000'`（飼養計畫）。
    /// 排除 Euthanized / SuddenDeath / Transferred / deleted。
    ///
    /// 體重必須有最近 ≤ [`AVAILABLE_PIG_WEIGHT_FRESHNESS_DAYS`] 天的量測紀錄，
    /// 否則排除出主結果並計入 `summary.excluded_weight_expired`。
    ///
    /// 詳見 `docs/plans/r47-available-pigs-query.md`。
    /// `restrict_to_assigned`：true 時只回已指派計畫（`iacuc_no IS NOT NULL`）的豬，
    /// 隱藏未指派庫存豬。供「只有 view_project、無 view_all」的呼叫者收斂可見範圍，
    /// 與 `list_animals` 的 `retain(iacuc_no.is_some())` 一致（#386）。
    pub async fn list_available_pigs(
        pool: &PgPool,
        query: &AvailablePigQuery,
        restrict_to_assigned: bool,
    ) -> Result<AvailablePigListResponse> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).clamp(1, 200);
        let offset = (page - 1) * per_page;

        let fresh_days = AVAILABLE_PIG_WEIGHT_FRESHNESS_DAYS;
        let breeding_no = BREEDING_PROTOCOL_NO;
        // restrict bool 為後端依權限計算（非使用者輸入），可安全內插進 SQL。
        let scope_clause = if restrict_to_assigned {
            "AND a.iacuc_no IS NOT NULL"
        } else {
            ""
        };

        // 共用 CTE：latest_weight（最近 ≤ N 天）+ eligible（status / breeding / 過濾條件套用）
        // 為避免重複組裝 + Option 過濾參數，採 COALESCE 寫法（NULL 時短路為「不過濾」）。
        let base_cte = format!(
            r#"
            WITH latest_weight AS (
                SELECT DISTINCT ON (animal_id)
                    animal_id, weight, measure_date
                FROM animal_weights
                WHERE deleted_at IS NULL
                  AND measure_date >= (CURRENT_DATE - INTERVAL '{fresh_days} days')
                ORDER BY animal_id, measure_date DESC
            ),
            eligible AS (
                SELECT
                    a.id, a.ear_tag, a.animal_no, a.breed, a.gender,
                    a.birth_date, a.pen_location, a.remark,
                    (EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                     + EXTRACT(MONTH FROM AGE(a.birth_date))::int) AS age_months,
                    lw.weight AS latest_weight_kg,
                    lw.measure_date AS weight_measured_at
                FROM animals a
                LEFT JOIN protocols pr ON a.iacuc_no = pr.iacuc_no
                INNER JOIN latest_weight lw ON lw.animal_id = a.id
                WHERE a.deleted_at IS NULL
                  AND a.birth_date IS NOT NULL
                  {scope_clause}
                  AND (
                      a.status IN ('unassigned'::animal_status, 'completed'::animal_status)
                      OR (
                          a.status = 'in_experiment'::animal_status
                          AND $1::boolean
                          AND pr.protocol_no = '{breeding_no}'
                      )
                  )
                  AND a.gender = COALESCE($2::animal_gender, a.gender)
                  AND (EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                       + EXTRACT(MONTH FROM AGE(a.birth_date))::int)
                      >= COALESCE($3::int,
                                  EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                                  + EXTRACT(MONTH FROM AGE(a.birth_date))::int)
                  AND (EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                       + EXTRACT(MONTH FROM AGE(a.birth_date))::int)
                      <= COALESCE($4::int,
                                  EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                                  + EXTRACT(MONTH FROM AGE(a.birth_date))::int)
                  AND lw.weight >= COALESCE($5::numeric, lw.weight)
                  AND lw.weight <= COALESCE($6::numeric, lw.weight)
            )
            "#
        );

        // ── Query 1: 列表（含 pagination）──
        let list_sql = format!(
            "{base_cte} SELECT * FROM eligible ORDER BY age_months ASC, ear_tag ASC LIMIT $7 OFFSET $8"
        );

        let mut rows: Vec<AvailablePigRow> = sqlx::query_as(sqlx::AssertSqlSafe(list_sql))
            .bind(query.include_breeding)
            .bind(query.sex)
            .bind(query.age_months_min)
            .bind(query.age_months_max)
            .bind(query.weight_min)
            .bind(query.weight_max)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        for row in &mut rows {
            row.ear_tag = AnimalUtils::format_ear_tag(&row.ear_tag);
            if let Some(pen) = &row.pen_location {
                row.pen_location = Some(AnimalUtils::format_pen_location(pen));
            }
        }

        // ── Query 2: summary aggregate（同 filter，無 pagination）──
        let summary_sql = format!(
            r#"
            {base_cte}
            SELECT
                COUNT(*)::bigint                                                           AS total,
                COUNT(*) FILTER (WHERE gender = 'male'::animal_gender)::bigint             AS male,
                COUNT(*) FILTER (WHERE gender = 'female'::animal_gender)::bigint           AS female,
                COUNT(*) FILTER (WHERE breed = 'miniature'::animal_breed)::bigint          AS minipig,
                COUNT(*) FILTER (WHERE breed = 'white'::animal_breed)::bigint              AS white,
                COUNT(*) FILTER (WHERE breed = 'LYD'::animal_breed)::bigint                AS lyd,
                COUNT(*) FILTER (WHERE breed = 'other'::animal_breed)::bigint              AS other
            FROM eligible
            "#
        );

        let (total, male, female, minipig, white, lyd, other): (i64, i64, i64, i64, i64, i64, i64) =
            sqlx::query_as(sqlx::AssertSqlSafe(summary_sql))
                .bind(query.include_breeding)
                .bind(query.sex)
                .bind(query.age_months_min)
                .bind(query.age_months_max)
                .bind(query.weight_min)
                .bind(query.weight_max)
                .fetch_one(pool)
                .await?;

        // ── Query 3: excluded_weight_expired（同 status / sex / age filter，但無新鮮體重）──
        let excluded_sql = format!(
            r#"
            SELECT COUNT(*)::bigint
            FROM animals a
            LEFT JOIN protocols pr ON a.iacuc_no = pr.iacuc_no
            WHERE a.deleted_at IS NULL
              AND a.birth_date IS NOT NULL
              {scope_clause}
              AND (
                  a.status IN ('unassigned'::animal_status, 'completed'::animal_status)
                  OR (
                      a.status = 'in_experiment'::animal_status
                      AND $1::boolean
                      AND pr.protocol_no = '{breeding_no}'
                  )
              )
              AND a.gender = COALESCE($2::animal_gender, a.gender)
              AND (EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                   + EXTRACT(MONTH FROM AGE(a.birth_date))::int)
                  >= COALESCE($3::int,
                              EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                              + EXTRACT(MONTH FROM AGE(a.birth_date))::int)
              AND (EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                   + EXTRACT(MONTH FROM AGE(a.birth_date))::int)
                  <= COALESCE($4::int,
                              EXTRACT(YEAR FROM AGE(a.birth_date))::int * 12
                              + EXTRACT(MONTH FROM AGE(a.birth_date))::int)
              AND NOT EXISTS (
                  SELECT 1 FROM animal_weights aw
                  WHERE aw.animal_id = a.id
                    AND aw.deleted_at IS NULL
                    AND aw.measure_date >= (CURRENT_DATE - INTERVAL '{fresh_days} days')
              )
            "#
        );

        let (excluded_weight_expired,): (i64,) = sqlx::query_as(sqlx::AssertSqlSafe(excluded_sql))
            .bind(query.include_breeding)
            .bind(query.sex)
            .bind(query.age_months_min)
            .bind(query.age_months_max)
            .fetch_one(pool)
            .await?;

        let mut by_breed = std::collections::HashMap::new();
        if minipig > 0 {
            by_breed.insert(AnimalBreed::Minipig.display_name().to_string(), minipig);
        }
        if white > 0 {
            by_breed.insert(AnimalBreed::White.display_name().to_string(), white);
        }
        if lyd > 0 {
            by_breed.insert(AnimalBreed::LYD.display_name().to_string(), lyd);
        }
        if other > 0 {
            by_breed.insert(AnimalBreed::Other.display_name().to_string(), other);
        }

        let summary = AvailablePigSummary {
            total,
            male,
            female,
            by_breed,
            excluded_weight_expired,
        };

        Ok(AvailablePigListResponse {
            animals: rows,
            summary,
            page,
            per_page,
        })
    }

    /// 取得紀錄版本歷史
    pub async fn get_record_versions(
        pool: &PgPool,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<crate::models::VersionHistoryResponse> {
        let versions = sqlx::query_as::<_, crate::models::VersionDiff>(
            r#"
            SELECT
                r.id,
                r.version_no,
                r.changed_at,
                r.changed_by,
                r.snapshot,
                r.diff_summary,
                u.display_name as changed_by_name
            FROM record_versions r
            LEFT JOIN users u ON r.changed_by = u.id
            WHERE r.record_type::text = $1 AND r.record_id = $2
            ORDER BY r.version_no DESC
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(pool)
        .await?;

        let current_version = versions.first().map(|v| v.version_no).unwrap_or(1);

        Ok(crate::models::VersionHistoryResponse {
            record_type: entity_type.to_string(),
            record_id: entity_id,
            current_version,
            versions,
        })
    }

    /// 取得動物客戶/委託資訊（狀態相依，供匯入體重驗證資訊行）。
    /// 實驗中/已完成 → 動物 `iacuc_no`；已轉讓 → 最新 completed 轉讓的 `to_iacuc_no`；
    /// 再 join `protocols` 取 `working_content.basic.sponsor.name` / `pi.name`。
    pub async fn get_client_info(
        pool: &PgPool,
        animal_id: Uuid,
    ) -> Result<AnimalClientInfoResponse> {
        sqlx::query_as::<_, AnimalClientInfoResponse>(
            r#"
            SELECT
                a.status::text AS status,
                eff.iacuc_no AS iacuc_no,
                p.working_content->'basic'->'sponsor'->>'name' AS sponsor_name,
                p.working_content->'basic'->'pi'->>'name' AS pi_name
            FROM animals a
            LEFT JOIN LATERAL (
                SELECT CASE
                    WHEN a.status = 'transferred' THEN (
                        SELECT t.to_iacuc_no FROM animal_transfers t
                        WHERE t.animal_id = a.id AND t.status = 'completed'
                        ORDER BY t.completed_at DESC NULLS LAST
                        LIMIT 1
                    )
                    ELSE a.iacuc_no
                END AS iacuc_no
            ) eff ON true
            LEFT JOIN protocols p ON p.iacuc_no = eff.iacuc_no
            WHERE a.id = $1 AND a.deleted_at IS NULL
            "#,
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("動物不存在".into()))
    }
}
