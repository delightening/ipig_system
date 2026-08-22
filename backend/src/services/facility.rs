// Facility Service
// 包含：Species, Facility, Building, Zone, Pen, Department

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::{AuditRedact, DataDiff},
        Building, BuildingWithFacility, CreateBuildingRequest, CreateDepartmentRequest,
        CreateFacilityRequest, CreatePenRequest, CreateSpeciesRequest, CreateZoneRequest,
        Department, DepartmentWithManager, Facility, Pen, PenDetails, PenQuery, Species,
        SpeciesListItem, UpdateBuildingRequest, UpdateDepartmentRequest, UpdateFacilityRequest,
        UpdatePenRequest, UpdateSpeciesRequest, UpdateZoneRequest, Zone, ZoneWithBuilding,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

// 設施結構資料（物種 / 設施 / 棟舍 / 區域 / 欄位 / 部門）皆非 PII，
// audit log 保留完整變更軌跡（GLP 21 CFR §11.10），無需脫敏欄位。
impl AuditRedact for Species {}
impl AuditRedact for Facility {}
impl AuditRedact for Building {}
impl AuditRedact for Zone {}
impl AuditRedact for Pen {}
impl AuditRedact for Department {}

pub struct FacilityService;

impl FacilityService {
    // ============================================
    // Species
    // ============================================

    /// 物種被幾隻動物引用。
    ///
    /// 刻意**不過濾 deleted_at**：軟刪除的動物在明細與稽核紀錄裡仍要顯示物種名稱，
    /// 物種列一旦消失就會變成孤兒。與 facility/building/zone/pen 的「活體動物」
    /// 守衛語意不同——那些擋的是「還住在裡面的動物」，這裡擋的是「還指著這一列的動物」。
    async fn count_animals_referencing_species(
        conn: &mut sqlx::PgConnection,
        species_id: Uuid,
    ) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM animals WHERE species_id = $1")
            .bind(species_id)
            .fetch_one(conn)
            .await?;
        Ok(count.0)
    }

    /// 有幾個子物種以此物種為 parent。
    async fn count_child_species(conn: &mut sqlx::PgConnection, species_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM species WHERE parent_id = $1")
            .bind(species_id)
            .fetch_one(conn)
            .await?;
        Ok(count.0)
    }

    /// 停用或刪除物種前的共用守衛：還被動物引用、或底下還有子物種，都不放行。
    ///
    /// `action` 只用來組錯誤訊息（「停用」/「刪除」）。
    async fn ensure_species_not_in_use(
        conn: &mut sqlx::PgConnection,
        species_id: Uuid,
        action: &str,
    ) -> Result<()> {
        let animals = Self::count_animals_referencing_species(conn, species_id).await?;
        if animals > 0 {
            return Err(AppError::BusinessRule(format!(
                "此物種仍有 {} 隻動物使用中，請先調整這些動物的物種後再{}",
                animals, action
            )));
        }
        // 子物種也要擋：否則子物種的 parent_id 會指向一筆已消失的紀錄，
        // 與上面的「動物孤兒」防護同一個道理。
        let children = Self::count_child_species(conn, species_id).await?;
        if children > 0 {
            return Err(AppError::BusinessRule(format!(
                "此物種底下仍有 {} 個子物種，請先處理子物種後再{}",
                children, action
            )));
        }
        Ok(())
    }

    /// 驗證 parent_id：必須存在、啟用中，且父物種本身必須是頂層。
    ///
    /// 物種階層固定兩層（物種 → 品種），前端父物種下拉本來就只列頂層。
    /// 「父必須是頂層」這條同時也是迴圈防護：有 parent 的列不能當別人的 parent，
    /// 因此 A→B→A 這種環在結構上就不可能成立，只需另外擋「指向自己」。
    /// 停用中的父物種不會出現在前端下拉，但直接打 API 仍可掛上去，故一併擋。
    async fn validate_species_parent(
        conn: &mut sqlx::PgConnection,
        parent_id: Uuid,
        self_id: Option<Uuid>,
    ) -> Result<()> {
        if self_id == Some(parent_id) {
            return Err(AppError::Validation("父物種不可指向自己".to_string()));
        }
        let parent: Option<(Option<Uuid>,)> =
            sqlx::query_as("SELECT parent_id FROM species WHERE id = $1 AND is_active = true")
                .bind(parent_id)
                .fetch_optional(conn)
                .await?;
        let Some((grandparent,)) = parent else {
            return Err(AppError::Validation(
                "指定的父物種不存在或已停用".to_string(),
            ));
        };
        if grandparent.is_some() {
            return Err(AppError::Validation(
                "父物種必須是頂層物種（物種階層僅支援兩層）".to_string(),
            ));
        }
        Ok(())
    }

    /// code 唯一預檢（不分大小寫）。
    ///
    /// DB 端另有 `uq_species_code_lower` 這個 functional unique index 作為併發下的真正防線
    /// （migration 141）——原本只有區分大小寫的 UNIQUE(code)，擋不住 `lyd` 與 `LYD` 併存。
    /// 這裡的預檢是為了回可讀訊息，不是唯一性的保證。
    /// 物種的 code 唯一性是**全域**的（`species_code_key` 與 `uq_species_code_lower` 都沒有
    /// `WHERE is_active` 條件），而刪除是軟刪除——代碼一旦用過就永久保留，即使該物種已停用。
    /// 這與 facilities / zones / pens 的 partial unique 不同，是刻意保留的：物種 code 是匯入
    /// 比對與品種推導的查表鍵，讓同一個代碼在歷史上指向兩種不同動物會破壞既有紀錄的可讀性。
    ///
    /// 因此撞到停用物種時要明確講出來，並指向「重新啟用」這條路，而不是丟一句
    /// 「已存在，請換一個」讓使用者以為系統壞了。
    async fn ensure_species_code_available(
        conn: &mut sqlx::PgConnection,
        code: &str,
    ) -> Result<()> {
        let duplicated: Option<(String, String, bool)> = sqlx::query_as(
            "SELECT code, name, is_active FROM species WHERE lower(code) = lower($1)",
        )
        .bind(code)
        .fetch_optional(conn)
        .await?;
        let Some((existing_code, existing_name, is_active)) = duplicated else {
            return Ok(());
        };
        if is_active {
            return Err(AppError::Conflict(format!(
                "物種代碼「{}」已被使用中的物種「{}」佔用，請換一個代碼",
                existing_code, existing_name
            )));
        }
        Err(AppError::Conflict(format!(
            "物種代碼「{}」屬於已停用的物種「{}」。物種代碼不重複使用，請於清單勾選「顯示停用物種」把它重新啟用，或改用其他代碼",
            existing_code, existing_name
        )))
    }

    /// 改掛父物種時的驗證：父物種本身要合法，且自己底下不能已經有子物種（否則變三層）。
    async fn validate_species_parent_change(
        conn: &mut sqlx::PgConnection,
        id: Uuid,
        parent_id: Uuid,
    ) -> Result<()> {
        Self::validate_species_parent(conn, parent_id, Some(id)).await?;
        let children = Self::count_child_species(conn, id).await?;
        if children > 0 {
            return Err(AppError::Validation(format!(
                "此物種底下已有 {} 個子物種，不可再指定父物種（物種階層僅支援兩層）",
                children
            )));
        }
        Ok(())
    }

    async fn insert_species(
        conn: &mut sqlx::PgConnection,
        payload: &CreateSpeciesRequest,
        code: &str,
        name: &str,
    ) -> Result<Species> {
        let species = sqlx::query_as::<_, Species>(
            r#"
            INSERT INTO species (id, code, name, name_en, icon, parent_id, config, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, 0))
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(code)
        .bind(name)
        .bind(&payload.name_en)
        .bind(&payload.icon)
        .bind(payload.parent_id)
        .bind(&payload.config)
        .bind(payload.sort_order)
        .fetch_one(conn)
        .await?;
        Ok(species)
    }

    /// 全欄位 COALESCE：未提供的欄位維持原值（傳 null 無法清空，為既有語意）。
    async fn apply_species_update(
        conn: &mut sqlx::PgConnection,
        id: Uuid,
        payload: &UpdateSpeciesRequest,
    ) -> Result<Species> {
        let species = sqlx::query_as::<_, Species>(
            r#"
            UPDATE species
            SET name = COALESCE($2, name),
                name_en = COALESCE($3, name_en),
                icon = COALESCE($4, icon),
                is_active = COALESCE($5, is_active),
                parent_id = COALESCE($6, parent_id),
                config = COALESCE($7, config),
                sort_order = COALESCE($8, sort_order),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&payload.name)
        .bind(&payload.name_en)
        .bind(&payload.icon)
        .bind(payload.is_active)
        .bind(payload.parent_id)
        .bind(&payload.config)
        .bind(payload.sort_order)
        .fetch_one(conn)
        .await?;
        Ok(species)
    }

    /// `include_inactive` 預設 false，維持既有行為（動物表單的品種下拉吃同一支 API，
    /// 不能讓停用物種混進去）。物種管理頁需要看見停用物種才能把它重新啟用——
    /// 代碼是全域唯一且不重用的，停用物種若無法現身就等於代碼被永久鎖死。
    pub async fn list_species(
        pool: &PgPool,
        include_inactive: bool,
    ) -> Result<Vec<SpeciesListItem>> {
        // sort_order 有大量重複值，補 name 作次要排序鍵讓輸出順序穩定。
        let species = sqlx::query_as::<_, SpeciesListItem>(
            r#"
            SELECT s.*,
                   (SELECT COUNT(*) FROM animals a WHERE a.species_id = s.id) AS animal_count
            FROM species s
            WHERE (s.is_active = true OR $1)
            ORDER BY s.is_active DESC, s.sort_order, s.name
            "#,
        )
        .bind(include_inactive)
        .fetch_all(pool)
        .await?;
        Ok(species)
    }

    pub async fn get_species(pool: &PgPool, id: Uuid) -> Result<Species> {
        let species = sqlx::query_as::<_, Species>("SELECT * FROM species WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(species)
    }

    pub async fn create_species(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &CreateSpeciesRequest,
    ) -> Result<Species> {
        let code = payload.code.trim();
        let name = payload.name.trim();
        if code.is_empty() {
            return Err(AppError::Validation("物種代碼為必填".to_string()));
        }
        if name.is_empty() {
            return Err(AppError::Validation("物種名稱為必填".to_string()));
        }

        let mut tx = pool.begin().await?;

        Self::ensure_species_code_available(&mut tx, code).await?;

        if let Some(parent_id) = payload.parent_id {
            Self::validate_species_parent(&mut tx, parent_id, None).await?;
        }

        let species = Self::insert_species(&mut tx, payload, code, name).await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_SPECIES_CREATE",
                entity: Some(AuditEntity::new("species", species.id, &species.name)),
                data_diff: Some(DataDiff::create_only(&species)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(species)
    }

    pub async fn update_species(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateSpeciesRequest,
    ) -> Result<Species> {
        let mut tx = pool.begin().await?;
        let before = sqlx::query_as::<_, Species>("SELECT * FROM species WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

        if let Some(parent_id) = payload.parent_id {
            Self::validate_species_parent_change(&mut tx, id, parent_id).await?;
        }

        // 停用使用中的物種會讓那些動物的品種顯示、與子物種的 parent 變成孤兒。
        if payload.is_active == Some(false) && before.is_active {
            Self::ensure_species_not_in_use(&mut tx, id, "停用").await?;
        }

        let species = Self::apply_species_update(&mut tx, id, payload).await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_SPECIES_UPDATE",
                entity: Some(AuditEntity::new("species", species.id, &species.name)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&species))),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(species)
    }

    pub async fn delete_species(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let mut tx = pool.begin().await?;
        let before = sqlx::query_as::<_, Species>("SELECT * FROM species WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        let Some(before) = before else {
            return Err(AppError::NotFound(format!("Species {} not found", id)));
        };
        Self::ensure_species_not_in_use(&mut tx, id, "刪除").await?;
        sqlx::query("UPDATE species SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_SPECIES_DELETE",
                entity: Some(AuditEntity::new("species", before.id, &before.name)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    // ============================================
    // Facility
    // ============================================

    pub async fn list_facilities(pool: &PgPool) -> Result<Vec<Facility>> {
        let facilities = sqlx::query_as::<_, Facility>(
            "SELECT * FROM facilities WHERE is_active = true ORDER BY code",
        )
        .fetch_all(pool)
        .await?;
        Ok(facilities)
    }

    pub async fn get_facility(pool: &PgPool, id: Uuid) -> Result<Facility> {
        let facility = sqlx::query_as::<_, Facility>("SELECT * FROM facilities WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(facility)
    }

    pub async fn create_facility(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &CreateFacilityRequest,
    ) -> Result<Facility> {
        let mut tx = pool.begin().await?;
        let facility = sqlx::query_as::<_, Facility>(
            r#"
            INSERT INTO facilities (id, code, name, address, phone, contact_person, config)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(&payload.address)
        .bind(&payload.phone)
        .bind(&payload.contact_person)
        .bind(&payload.config)
        .fetch_one(&mut *tx)
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_FACILITY_CREATE",
                entity: Some(AuditEntity::new("facility", facility.id, &facility.name)),
                data_diff: Some(DataDiff::create_only(&facility)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(facility)
    }

    pub async fn update_facility(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateFacilityRequest,
    ) -> Result<Facility> {
        let mut tx = pool.begin().await?;
        let before =
            sqlx::query_as::<_, Facility>("SELECT * FROM facilities WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
        let facility = sqlx::query_as::<_, Facility>(
            r#"
            UPDATE facilities
            SET name = COALESCE($2, name),
                address = COALESCE($3, address),
                phone = COALESCE($4, phone),
                contact_person = COALESCE($5, contact_person),
                is_active = COALESCE($6, is_active),
                config = COALESCE($7, config),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&payload.name)
        .bind(&payload.address)
        .bind(&payload.phone)
        .bind(&payload.contact_person)
        .bind(payload.is_active)
        .bind(&payload.config)
        .fetch_one(&mut *tx)
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_FACILITY_UPDATE",
                entity: Some(AuditEntity::new("facility", facility.id, &facility.name)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&facility))),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(facility)
    }

    pub async fn delete_facility(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let mut tx = pool.begin().await?;
        let before =
            sqlx::query_as::<_, Facility>("SELECT * FROM facilities WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some(before) = before else {
            return Err(AppError::NotFound(format!("Facility {} not found", id)));
        };
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM animals a
               JOIN pens p ON a.pen_id = p.id
               JOIN zones z ON p.zone_id = z.id
               JOIN buildings b ON z.building_id = b.id
               WHERE b.facility_id = $1
                 AND a.deleted_at IS NULL
                 AND a.status NOT IN ('euthanized','sudden_death','transferred')"#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        if count.0 > 0 {
            return Err(crate::AppError::BusinessRule(format!(
                "此設施內仍有 {} 隻活體動物，請先移出後再刪除",
                count.0
            )));
        }
        sqlx::query("UPDATE facilities SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_FACILITY_DELETE",
                entity: Some(AuditEntity::new("facility", before.id, &before.name)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    // ============================================
    // Building
    // ============================================

    pub async fn list_buildings(
        pool: &PgPool,
        facility_id: Option<Uuid>,
    ) -> Result<Vec<BuildingWithFacility>> {
        let buildings = sqlx::query_as::<_, BuildingWithFacility>(
            r#"
            SELECT 
                b.id, b.facility_id, f.code as facility_code, f.name as facility_name,
                b.code, b.name, b.description, b.is_active, b.config, b.sort_order
            FROM buildings b
            INNER JOIN facilities f ON b.facility_id = f.id
            WHERE b.is_active = true
              AND ($1::uuid IS NULL OR b.facility_id = $1)
            ORDER BY f.code, b.sort_order
            "#,
        )
        .bind(facility_id)
        .fetch_all(pool)
        .await?;
        Ok(buildings)
    }

    pub async fn get_building(pool: &PgPool, id: Uuid) -> Result<Building> {
        let building = sqlx::query_as::<_, Building>("SELECT * FROM buildings WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(building)
    }

    pub async fn create_building(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &CreateBuildingRequest,
    ) -> Result<Building> {
        let mut tx = pool.begin().await?;
        let building = sqlx::query_as::<_, Building>(
            r#"
            INSERT INTO buildings (id, facility_id, code, name, description, config, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, 0))
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(payload.facility_id)
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(&payload.config)
        .bind(payload.sort_order)
        .fetch_one(&mut *tx)
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_BUILDING_CREATE",
                entity: Some(AuditEntity::new("building", building.id, &building.name)),
                data_diff: Some(DataDiff::create_only(&building)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(building)
    }

    pub async fn update_building(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateBuildingRequest,
    ) -> Result<Building> {
        let mut tx = pool.begin().await?;
        let before =
            sqlx::query_as::<_, Building>("SELECT * FROM buildings WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
        let building = sqlx::query_as::<_, Building>(
            r#"
            UPDATE buildings
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                is_active = COALESCE($4, is_active),
                config = COALESCE($5, config),
                sort_order = COALESCE($6, sort_order),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(payload.is_active)
        .bind(&payload.config)
        .bind(payload.sort_order)
        .fetch_one(&mut *tx)
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_BUILDING_UPDATE",
                entity: Some(AuditEntity::new("building", building.id, &building.name)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&building))),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(building)
    }

    pub async fn delete_building(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let mut tx = pool.begin().await?;
        let before =
            sqlx::query_as::<_, Building>("SELECT * FROM buildings WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some(before) = before else {
            return Err(AppError::NotFound(format!("Building {} not found", id)));
        };
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM animals a
               JOIN pens p ON a.pen_id = p.id
               JOIN zones z ON p.zone_id = z.id
               WHERE z.building_id = $1
                 AND a.deleted_at IS NULL
                 AND a.status NOT IN ('euthanized','sudden_death','transferred')"#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        if count.0 > 0 {
            return Err(crate::AppError::BusinessRule(format!(
                "此棟舍內仍有 {} 隻活體動物，請先移出後再刪除",
                count.0
            )));
        }
        sqlx::query("UPDATE buildings SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_BUILDING_DELETE",
                entity: Some(AuditEntity::new("building", before.id, &before.name)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    // ============================================
    // Zone
    // ============================================

    pub async fn list_zones(
        pool: &PgPool,
        building_id: Option<Uuid>,
    ) -> Result<Vec<ZoneWithBuilding>> {
        let zones = sqlx::query_as::<_, ZoneWithBuilding>(
            r#"
            SELECT 
                z.id, z.building_id, b.code as building_code, b.name as building_name,
                b.facility_id, f.name as facility_name,
                z.code, z.name, z.color, z.is_active, z.layout_config, z.sort_order
            FROM zones z
            INNER JOIN buildings b ON z.building_id = b.id
            INNER JOIN facilities f ON b.facility_id = f.id
            WHERE z.is_active = true
              AND ($1::uuid IS NULL OR z.building_id = $1)
            ORDER BY b.code, z.sort_order
            "#,
        )
        .bind(building_id)
        .fetch_all(pool)
        .await?;
        Ok(zones)
    }

    pub async fn get_zone(pool: &PgPool, id: Uuid) -> Result<Zone> {
        let zone = sqlx::query_as::<_, Zone>("SELECT * FROM zones WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(zone)
    }

    pub async fn create_zone(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &CreateZoneRequest,
    ) -> Result<Zone> {
        let mut tx = pool.begin().await?;
        let zone = sqlx::query_as::<_, Zone>(
            r#"
            INSERT INTO zones (id, building_id, code, name, color, layout_config, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, 0))
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(payload.building_id)
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(&payload.color)
        .bind(&payload.layout_config)
        .bind(payload.sort_order)
        .fetch_one(&mut *tx)
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_ZONE_CREATE",
                entity: Some(AuditEntity::new("zone", zone.id, &zone.code)),
                data_diff: Some(DataDiff::create_only(&zone)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(zone)
    }

    pub async fn update_zone(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateZoneRequest,
    ) -> Result<Zone> {
        let mut tx = pool.begin().await?;
        let before = sqlx::query_as::<_, Zone>("SELECT * FROM zones WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let zone = sqlx::query_as::<_, Zone>(
            r#"
            UPDATE zones
            SET name = COALESCE($2, name),
                color = COALESCE($3, color),
                is_active = COALESCE($4, is_active),
                layout_config = COALESCE($5, layout_config),
                sort_order = COALESCE($6, sort_order),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&payload.name)
        .bind(&payload.color)
        .bind(payload.is_active)
        .bind(&payload.layout_config)
        .bind(payload.sort_order)
        .fetch_one(&mut *tx)
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_ZONE_UPDATE",
                entity: Some(AuditEntity::new("zone", zone.id, &zone.code)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&zone))),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(zone)
    }

    pub async fn delete_zone(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let mut tx = pool.begin().await?;
        let before = sqlx::query_as::<_, Zone>("SELECT * FROM zones WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        let Some(before) = before else {
            return Err(AppError::NotFound(format!("Zone {} not found", id)));
        };
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM animals a
               JOIN pens p ON a.pen_id = p.id
               WHERE p.zone_id = $1
                 AND a.deleted_at IS NULL
                 AND a.status NOT IN ('euthanized','sudden_death','transferred')"#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        if count.0 > 0 {
            return Err(crate::AppError::BusinessRule(format!(
                "此區域內仍有 {} 隻活體動物，請先移出後再刪除",
                count.0
            )));
        }
        sqlx::query("UPDATE zones SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_ZONE_DELETE",
                entity: Some(AuditEntity::new("zone", before.id, &before.code)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    // ============================================
    // Pen
    // ============================================

    pub async fn list_pens(pool: &PgPool, query: &PenQuery) -> Result<Vec<PenDetails>> {
        let pens = sqlx::query_as::<_, PenDetails>(
            r#"
            SELECT
                p.id, p.code, p.name, p.capacity, p.current_count, p.status,
                p.row_index, p.col_index,
                z.id as zone_id, z.code as zone_code, z.name as zone_name, z.color as zone_color,
                z.layout_config as zone_layout_config,
                b.id as building_id, b.code as building_code, b.name as building_name,
                f.id as facility_id, f.code as facility_code, f.name as facility_name
            FROM pens p
            INNER JOIN zones z ON p.zone_id = z.id
            INNER JOIN buildings b ON z.building_id = b.id
            INNER JOIN facilities f ON b.facility_id = f.id
            WHERE ($1::uuid IS NULL OR p.zone_id = $1)
              AND ($2::uuid IS NULL OR z.building_id = $2)
              AND ($3::uuid IS NULL OR b.facility_id = $3)
              AND ($4::text IS NULL OR p.status = $4)
              AND p.is_active = COALESCE($5, true)
            ORDER BY f.code, b.sort_order, z.sort_order, p.code
            "#,
        )
        .bind(query.zone_id)
        .bind(query.building_id)
        .bind(query.facility_id)
        .bind(&query.status)
        .bind(query.is_active)
        .fetch_all(pool)
        .await?;
        Ok(pens)
    }

    pub async fn get_pen(pool: &PgPool, id: Uuid) -> Result<Pen> {
        let pen = sqlx::query_as::<_, Pen>("SELECT * FROM pens WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(pen)
    }

    pub async fn create_pen(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &CreatePenRequest,
    ) -> Result<Pen> {
        let mut tx = pool.begin().await?;
        let pen = sqlx::query_as::<_, Pen>(
            r#"
            INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
            VALUES ($1, $2, $3, $4, COALESCE($5, 1), $6, $7)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(payload.zone_id)
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(payload.capacity)
        .bind(payload.row_index)
        .bind(payload.col_index)
        .fetch_one(&mut *tx)
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_PEN_CREATE",
                entity: Some(AuditEntity::new("pen", pen.id, &pen.code)),
                data_diff: Some(DataDiff::create_only(&pen)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(pen)
    }

    /// 批次建立欄位
    pub async fn batch_create_pens(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &crate::models::BatchCreatePensRequest,
    ) -> Result<Vec<Pen>> {
        if payload.count < 1 || payload.count > 200 {
            return Err(crate::AppError::BusinessRule(
                "數量需介於 1 到 200 之間".to_string(),
            ));
        }
        if payload.prefix.is_empty() || payload.prefix.len() > 10 {
            return Err(crate::AppError::BusinessRule(
                "前綴長度需介於 1 到 10 之間".to_string(),
            ));
        }

        let is_double = payload.layout == "double";
        let half = if is_double {
            (payload.count + 1) / 2
        } else {
            payload.count
        };
        let mut pens = Vec::with_capacity(payload.count as usize);

        let mut tx = pool.begin().await?;

        for i in 0..payload.count {
            let num = i + 1;
            let code = format!("{}{:02}", payload.prefix, num);
            let (row_index, col_index) = if is_double {
                if num <= half {
                    (num - 1, 0) // 左欄
                } else {
                    (num - half - 1, 1) // 右欄
                }
            } else {
                (i, 0) // 單欄
            };

            // pens 的唯一索引是 partial（uq_pens_zone_code_active ... WHERE is_active = true，
            // 見 migrations/005）。PostgreSQL 推斷 partial unique index 時，ON CONFLICT 的
            // conflict target 必須帶上同一個 WHERE 謂詞；否則計畫階段就丟 42P10
            // （no unique constraint matching the ON CONFLICT specification）→ 整批失敗。
            let pen = sqlx::query_as::<_, Pen>(
                r#"
                INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
                VALUES ($1, $2, $3, $3, $4, $5, $6)
                ON CONFLICT (zone_id, code) WHERE is_active = true DO NOTHING
                RETURNING *
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(payload.zone_id)
            .bind(&code)
            .bind(payload.capacity)
            .bind(row_index)
            .bind(col_index)
            .fetch_optional(&mut *tx)
            .await?;

            // 僅收集實際建立（未被 ON CONFLICT 跳過）的欄位，audit 於迴圈外一次寫。
            if let Some(p) = pen {
                pens.push(p);
            }
        }

        // #649（gemini）：批次 audit 收斂為「單一」事件，不在迴圈內逐筆寫。
        // 原因：同一 tx 內 created_at = now()（交易時間戳，對所有 row 相同），而 HMAC chain
        // 以 (created_at, id) 排序、id 為隨機 UUID → 逐筆寫會使「寫入序」與「最終排序」不一致，
        // compute_and_store_hmac_tx 記錄的 previous_hash 與 verify_chain 重算結果對不上 → 誤判斷鏈。
        // 單一事件無此 intra-tx 排序歧義，且把 DB round-trip 從最多 200 次降為 1 次。
        if !pens.is_empty() {
            #[derive(serde::Serialize)]
            struct BatchPenCreateAudit<'a> {
                zone_id: Uuid,
                count: usize,
                pens: &'a [Pen],
            }
            impl AuditRedact for BatchPenCreateAudit<'_> {}

            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "FACILITY",
                    event_type: "FACILITY_PEN_BATCH_CREATE",
                    entity: Some(AuditEntity::new("zone", payload.zone_id, &payload.prefix)),
                    data_diff: Some(DataDiff::create_only(&BatchPenCreateAudit {
                        zone_id: payload.zone_id,
                        count: pens.len(),
                        pens: &pens,
                    })),
                    request_context: None,
                },
            )
            .await?;
        }

        tx.commit().await?;
        Ok(pens)
    }

    pub async fn update_pen(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdatePenRequest,
    ) -> Result<Pen> {
        let mut tx = pool.begin().await?;
        let before = sqlx::query_as::<_, Pen>("SELECT * FROM pens WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let pen = sqlx::query_as::<_, Pen>(
            r#"
            UPDATE pens
            SET name = COALESCE($2, name),
                capacity = COALESCE($3, capacity),
                status = COALESCE($4, status),
                row_index = COALESCE($5, row_index),
                col_index = COALESCE($6, col_index),
                is_active = COALESCE($7, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&payload.name)
        .bind(payload.capacity)
        .bind(&payload.status)
        .bind(payload.row_index)
        .bind(payload.col_index)
        .bind(payload.is_active)
        .fetch_one(&mut *tx)
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_PEN_UPDATE",
                entity: Some(AuditEntity::new("pen", pen.id, &pen.code)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&pen))),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(pen)
    }

    pub async fn delete_pen(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let mut tx = pool.begin().await?;
        let before = sqlx::query_as::<_, Pen>("SELECT * FROM pens WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        let Some(before) = before else {
            return Err(AppError::NotFound(format!("Pen {} not found", id)));
        };
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM animals
               WHERE pen_id = $1
                 AND deleted_at IS NULL
                 AND status NOT IN ('euthanized','sudden_death','transferred')"#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        if count.0 > 0 {
            return Err(crate::AppError::BusinessRule(format!(
                "此欄位內仍有 {} 隻活體動物，請先移出後再刪除",
                count.0
            )));
        }
        sqlx::query("UPDATE pens SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_PEN_DELETE",
                entity: Some(AuditEntity::new("pen", before.id, &before.code)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    // ============================================
    // Department
    // ============================================

    pub async fn list_departments(pool: &PgPool) -> Result<Vec<DepartmentWithManager>> {
        let departments = sqlx::query_as::<_, DepartmentWithManager>(
            r#"
            SELECT 
                d.id, d.code, d.name, d.parent_id, parent.name as parent_name,
                d.manager_id, manager.display_name as manager_name,
                d.is_active, d.sort_order
            FROM departments d
            LEFT JOIN departments parent ON d.parent_id = parent.id
            LEFT JOIN users manager ON d.manager_id = manager.id
            WHERE d.is_active = true
            ORDER BY d.sort_order
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(departments)
    }

    pub async fn get_department(pool: &PgPool, id: Uuid) -> Result<Department> {
        let department = sqlx::query_as::<_, Department>("SELECT * FROM departments WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(department)
    }

    pub async fn create_department(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &CreateDepartmentRequest,
    ) -> Result<Department> {
        let mut tx = pool.begin().await?;
        let department = sqlx::query_as::<_, Department>(
            r#"
            INSERT INTO departments (id, code, name, parent_id, manager_id, config, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, 0))
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(payload.parent_id)
        .bind(payload.manager_id)
        .bind(&payload.config)
        .bind(payload.sort_order)
        .fetch_one(&mut *tx)
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_DEPARTMENT_CREATE",
                entity: Some(AuditEntity::new(
                    "department",
                    department.id,
                    &department.name,
                )),
                data_diff: Some(DataDiff::create_only(&department)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(department)
    }

    pub async fn update_department(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateDepartmentRequest,
    ) -> Result<Department> {
        let mut tx = pool.begin().await?;
        let before =
            sqlx::query_as::<_, Department>("SELECT * FROM departments WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
        let department = sqlx::query_as::<_, Department>(
            r#"
            UPDATE departments
            SET name = COALESCE($2, name),
                parent_id = COALESCE($3, parent_id),
                manager_id = COALESCE($4, manager_id),
                is_active = COALESCE($5, is_active),
                config = COALESCE($6, config),
                sort_order = COALESCE($7, sort_order),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&payload.name)
        .bind(payload.parent_id)
        .bind(payload.manager_id)
        .bind(payload.is_active)
        .bind(&payload.config)
        .bind(payload.sort_order)
        .fetch_one(&mut *tx)
        .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_DEPARTMENT_UPDATE",
                entity: Some(AuditEntity::new(
                    "department",
                    department.id,
                    &department.name,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&department))),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(department)
    }

    pub async fn delete_department(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let mut tx = pool.begin().await?;
        let before =
            sqlx::query_as::<_, Department>("SELECT * FROM departments WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some(before) = before else {
            return Err(AppError::NotFound(format!("Department {} not found", id)));
        };
        sqlx::query("UPDATE departments SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "FACILITY",
                event_type: "FACILITY_DEPARTMENT_DELETE",
                entity: Some(AuditEntity::new("department", before.id, &before.name)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
