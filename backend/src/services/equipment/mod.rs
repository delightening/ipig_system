// 設備維護管理 Service (實驗室 GLP)
//
// R82-8 巨檔拆分：原 services/equipment.rs（2820 行）依業務域拆為子模組，
// 純搬移不改邏輯、公開 API 不變。各子模組續以 `impl EquipmentService` 掛方法。

mod annual_plan;
mod calibration;
mod core;
mod disposal;
mod idle;
mod maintenance;

use crate::{error::AppError, middleware::CurrentUser, models::EquipmentStatus, Result};

pub struct EquipmentService;

fn check_view_permission(current_user: &CurrentUser) -> Result<()> {
    if !current_user.has_permission("equipment.view")
        && !current_user.has_permission("equipment.manage")
    {
        return Err(AppError::Forbidden("無權查看設備".into()));
    }
    Ok(())
}

fn check_manage_permission(current_user: &CurrentUser) -> Result<()> {
    if !current_user.has_permission("equipment.manage") {
        return Err(AppError::Forbidden("無權管理設備".into()));
    }
    Ok(())
}

/// 驗證設備狀態轉換是否合法（GLP/ISO 合規）
pub fn validate_status_transition(from: &EquipmentStatus, to: &EquipmentStatus) -> Result<()> {
    if from == to {
        return Ok(());
    }
    let allowed = matches!(
        (from, to),
        (EquipmentStatus::Active, EquipmentStatus::UnderRepair)
            | (EquipmentStatus::Active, EquipmentStatus::Inactive)
            | (EquipmentStatus::Active, EquipmentStatus::Decommissioned)
            | (EquipmentStatus::Inactive, EquipmentStatus::Active)
            | (EquipmentStatus::Inactive, EquipmentStatus::Decommissioned)
            | (EquipmentStatus::UnderRepair, EquipmentStatus::Active)
            | (
                EquipmentStatus::UnderRepair,
                EquipmentStatus::Decommissioned
            )
            | (EquipmentStatus::Decommissioned, EquipmentStatus::Active)
    );
    if !allowed {
        return Err(AppError::BadRequest(format!(
            "不允許的狀態轉換：{:?} → {:?}",
            from, to
        )));
    }
    Ok(())
}

/// 重新簽章取代待驗收殘留孤兒簽章時，作廢舊簽章的原因（21 CFR Part 11 簽章唯一性：
/// 同一實體不可同時存在多個有效簽章）。定義為常數供 service 與測試共用單一事實來源。
pub const MAINTENANCE_RESIGN_SUPERSEDE_REASON: &str = "重新簽章取代前次未完成驗收殘留的簽章";

/// SEC-SoD 職權分離守衛：申請 / 登錄者不得核准或驗收自己送出的紀錄。
///
/// 集中設備模組的自核檢查（報廢 `approve_disposal` / 閒置 `approve_idle_request` /
/// 維護驗收 `review_maintenance_record`），避免「大多有、少數漏」再長出離群點
/// （2026-07-04 資安稽核 M-1 / L-2）。
///
/// R97-1 起實作上移到 `services::access`，供 ERP 單據核准共用；此處保留 re-export，
/// 設備模組三個呼叫端的 `use super::assert_not_self_approval` 不受影響。
pub(crate) use crate::services::access::assert_not_self_approval;

#[cfg(test)]
mod status_transition_tests {
    use super::validate_status_transition;
    use crate::models::EquipmentStatus::*;

    #[test]
    fn same_status_is_noop_ok() {
        assert!(validate_status_transition(&Inactive, &Inactive).is_ok());
    }

    #[test]
    fn idle_equipment_can_be_decommissioned_directly() {
        // 閒置設備應可直接報廢，不必先恢復為啟用（Gemini #938 抓出的既有缺口）。
        assert!(validate_status_transition(&Inactive, &Decommissioned).is_ok());
    }

    #[test]
    fn existing_allowed_transitions_still_pass() {
        for (from, to) in [
            (Active, UnderRepair),
            (Active, Inactive),
            (Active, Decommissioned),
            (Inactive, Active),
            (UnderRepair, Active),
            (UnderRepair, Decommissioned),
            (Decommissioned, Active),
        ] {
            assert!(
                validate_status_transition(&from, &to).is_ok(),
                "{:?} → {:?} 應允許",
                from,
                to
            );
        }
    }

    #[test]
    fn nonsensical_transition_still_rejected() {
        // 維修中不可直接閒置（無此業務語意），維持拒絕。
        assert!(validate_status_transition(&UnderRepair, &Inactive).is_err());
    }
}
