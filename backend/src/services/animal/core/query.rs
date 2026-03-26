use sqlx::PgPool;
use uuid::Uuid;

use super::super::utils::AnimalUtils;
use super::super::AnimalService;
use crate::{
    models::{
        Animal, AnimalListItem, AnimalQuery, AnimalsByPen, AnimalStatsResponse, PaginatedResponse,
    },
    AppError, Result,
};

impl AnimalService {
    pub(super) fn push_animal_filters(
        qb: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
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

    /// 取得動物狀態統計（單一查詢，合併 status count + pen count）
    pub async fn stats(pool: &PgPool) -> Result<AnimalStatsResponse> {
        let rows: Vec<(String, i64, i64)> = sqlx::query_as(
            r#"
            SELECT
                status::text,
                COUNT(*) as count,
                COUNT(*) FILTER (
                    WHERE pen_location IS NOT NULL AND TRIM(pen_location) != ''
                ) as pen_count
            FROM animals
            WHERE deleted_at IS NULL
            GROUP BY status
            "#,
        )
        .fetch_all(pool)
        .await?;

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
            total,
        })
    }

    /// 取得動物列表（支援分頁）
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
                vrd.max_created_at as vet_recommendation_date,
                lw.weight as latest_weight,
                lw.measure_date as latest_weight_date
            FROM animals p
            LEFT JOIN animal_sources s ON p.source_id = s.id
            LEFT JOIN species sp ON p.species_id = sp.id
            LEFT JOIN LATERAL (
                SELECT MAX(vr.created_at) as max_created_at
                FROM vet_recommendations vr
                WHERE (vr.record_type = 'observation'::vet_record_type AND vr.record_id IN (
                    SELECT id FROM animal_observations WHERE animal_id = p.id
                ))
                OR (vr.record_type = 'surgery'::vet_record_type AND vr.record_id IN (
                    SELECT id FROM animal_surgeries WHERE animal_id = p.id
                ))
            ) vrd ON true
            LEFT JOIN LATERAL (
                SELECT pw.weight, pw.measure_date
                FROM animal_weights pw
                WHERE pw.animal_id = p.id
                ORDER BY pw.measure_date DESC
                LIMIT 1
            ) lw ON true
            WHERE p.deleted_at IS NULL
            "#,
        );

        Self::push_animal_filters(&mut query_builder, query);
        query_builder.push(" ORDER BY p.id DESC");

        if paginated {
            let offset = (page - 1) * per_page;
            query_builder.push(" LIMIT ");
            query_builder.push_bind(per_page);
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset);
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
        Ok(PaginatedResponse::new(animals, actual_total, page, per_page))
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
                (SELECT u.display_name FROM users u WHERE u.id = p.experiment_assigned_by) AS experiment_assigned_by_name
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

    /// 標記動物為獸醫已讀
    pub async fn mark_vet_read(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE animals SET vet_read_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
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
                u.name as changed_by_name
            FROM record_versions r
            LEFT JOIN users u ON r.changed_by = u.id
            WHERE r.record_type = $1 AND r.record_id = $2
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
}
