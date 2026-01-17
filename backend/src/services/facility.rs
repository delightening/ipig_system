// Facility Service
// 包含：Species, Facility, Building, Zone, Pen, Department

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        Building, BuildingWithFacility, CreateBuildingRequest, CreateDepartmentRequest,
        CreateFacilityRequest, CreatePenRequest, CreateSpeciesRequest, CreateZoneRequest,
        Department, DepartmentWithManager, Facility, Pen, PenDetails, PenQuery, Species,
        UpdateBuildingRequest, UpdateDepartmentRequest, UpdateFacilityRequest, UpdatePenRequest,
        UpdateSpeciesRequest, UpdateZoneRequest, Zone, ZoneWithBuilding,
    },
    Result,
};

pub struct FacilityService;

impl FacilityService {
    // ============================================
    // Species
    // ============================================

    pub async fn list_species(pool: &PgPool) -> Result<Vec<Species>> {
        let species =
            sqlx::query_as::<_, Species>("SELECT * FROM species WHERE is_active = true ORDER BY sort_order")
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

    pub async fn create_species(pool: &PgPool, payload: &CreateSpeciesRequest) -> Result<Species> {
        let species = sqlx::query_as::<_, Species>(
            r#"
            INSERT INTO species (id, code, name, name_en, icon, config, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, 0))
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(&payload.name_en)
        .bind(&payload.icon)
        .bind(&payload.config)
        .bind(payload.sort_order)
        .fetch_one(pool)
        .await?;
        Ok(species)
    }

    pub async fn update_species(
        pool: &PgPool,
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
                config = COALESCE($6, config),
                sort_order = COALESCE($7, sort_order),
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
        .bind(&payload.config)
        .bind(payload.sort_order)
        .fetch_one(pool)
        .await?;
        Ok(species)
    }

    pub async fn delete_species(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE species SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
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
        payload: &CreateFacilityRequest,
    ) -> Result<Facility> {
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
        .fetch_one(pool)
        .await?;
        Ok(facility)
    }

    pub async fn update_facility(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateFacilityRequest,
    ) -> Result<Facility> {
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
        .fetch_one(pool)
        .await?;
        Ok(facility)
    }

    pub async fn delete_facility(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE facilities SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
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
        payload: &CreateBuildingRequest,
    ) -> Result<Building> {
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
        .fetch_one(pool)
        .await?;
        Ok(building)
    }

    pub async fn update_building(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateBuildingRequest,
    ) -> Result<Building> {
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
        .fetch_one(pool)
        .await?;
        Ok(building)
    }

    pub async fn delete_building(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE buildings SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
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

    pub async fn create_zone(pool: &PgPool, payload: &CreateZoneRequest) -> Result<Zone> {
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
        .fetch_one(pool)
        .await?;
        Ok(zone)
    }

    pub async fn update_zone(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateZoneRequest,
    ) -> Result<Zone> {
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
        .fetch_one(pool)
        .await?;
        Ok(zone)
    }

    pub async fn delete_zone(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE zones SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
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
                z.id as zone_id, z.code as zone_code, z.name as zone_name, z.color as zone_color,
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
              AND ($5::bool IS NULL OR p.is_active = $5)
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

    pub async fn create_pen(pool: &PgPool, payload: &CreatePenRequest) -> Result<Pen> {
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
        .fetch_one(pool)
        .await?;
        Ok(pen)
    }

    pub async fn update_pen(pool: &PgPool, id: Uuid, payload: &UpdatePenRequest) -> Result<Pen> {
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
        .fetch_one(pool)
        .await?;
        Ok(pen)
    }

    pub async fn delete_pen(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE pens SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
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
        let department =
            sqlx::query_as::<_, Department>("SELECT * FROM departments WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await?;
        Ok(department)
    }

    pub async fn create_department(
        pool: &PgPool,
        payload: &CreateDepartmentRequest,
    ) -> Result<Department> {
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
        .fetch_one(pool)
        .await?;
        Ok(department)
    }

    pub async fn update_department(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateDepartmentRequest,
    ) -> Result<Department> {
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
        .fetch_one(pool)
        .await?;
        Ok(department)
    }

    pub async fn delete_department(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE departments SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
