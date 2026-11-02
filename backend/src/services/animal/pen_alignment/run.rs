//! 欄位對齊的 I/O 部分：讀舊系統 CSV、載入快照、逐隻透過 `AnimalService::update` 寫入。
//!
//! 寫入**只**經 `AnimalService::update`：同一交易內 UPDATE animals、重算舊欄與新欄的
//! current_count、寫 `ANIMAL_UPDATE` 稽核（與一般的欄位變更同一事件類別）。
//! 本檔沒有任何會改 animals / pens 的裸 SQL。

use sqlx::PgPool;

use super::super::AnimalService;
use super::plan::{LegacyRow, Plan};
use crate::{
    middleware::ActorContext,
    models::UpdateAnimalRequest,
    repositories::pen_alignment::{AnimalSnap, PenAlignmentRepository, PenSnap},
    AppError, Result,
};

/// 解析舊系統 CSV。只取 `old_pen` 與 `ear` 兩欄（其餘欄位忽略）；
/// 標頭容許 UTF-8 BOM 與前後空白。
pub fn read_legacy_csv<R: std::io::Read>(reader: R) -> Result<Vec<LegacyRow>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);
    let headers = rdr
        .headers()
        .map_err(|e| AppError::BadRequest(format!("讀取 CSV 標頭失敗：{e}")))?
        .clone();
    let find = |name: &str| {
        headers
            .iter()
            .position(|h| {
                h.trim_start_matches('\u{feff}')
                    .trim()
                    .eq_ignore_ascii_case(name)
            })
            .ok_or_else(|| AppError::BadRequest(format!("CSV 缺少 `{name}` 欄")))
    };
    let pen_idx = find("old_pen")?;
    let ear_idx = find("ear")?;

    let mut rows = Vec::new();
    for (i, rec) in rdr.records().enumerate() {
        let rec = rec
            .map_err(|e| AppError::BadRequest(format!("CSV 第 {} 筆資料列解析失敗：{e}", i + 1)))?;
        let ear = rec.get(ear_idx).unwrap_or_default().trim().to_string();
        let old_pen = rec.get(pen_idx).unwrap_or_default().trim().to_string();
        if ear.is_empty() && old_pen.is_empty() {
            continue;
        }
        rows.push(LegacyRow { old_pen, ear });
    }
    Ok(rows)
}

pub struct Snapshots {
    pub animals: Vec<AnimalSnap>,
    pub pens: Vec<PenSnap>,
}

pub async fn load_snapshots(pool: &PgPool) -> Result<Snapshots> {
    Ok(Snapshots {
        animals: PenAlignmentRepository::list_live_animals(pool).await?,
        pens: PenAlignmentRepository::list_pens(pool, None).await?,
    })
}

/// 受影響欄位目前的狀態（對帳用）。
pub async fn load_affected_pens(pool: &PgPool, plan: &Plan) -> Result<Vec<PenSnap>> {
    let ids = plan.affected_pen_ids();
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    PenAlignmentRepository::list_pens(pool, Some(&ids)).await
}

#[derive(Debug, Clone)]
pub struct ApplyFailure {
    pub ear: String,
    pub to_code: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct ApplyOutcome {
    pub applied: usize,
    /// 執行時才失敗的（例如版本衝突、執行期容量檢查不過）。不中止，繼續下一隻。
    pub failed: Vec<ApplyFailure>,
}

/// 依規劃順序逐隻更新。只動 `pen_location` 與（必要時）`pen_id`；
/// status / iacuc_no / experiment_date 一律不送（`UpdateAnimalRequest` 的 `None` = 不變）。
pub async fn apply_plan(pool: &PgPool, actor: &ActorContext, plan: &Plan) -> ApplyOutcome {
    let mut out = ApplyOutcome::default();
    for (item, mv) in plan.moves_in_order() {
        let req = UpdateAnimalRequest {
            pen_location: Some(mv.to_code.clone()),
            pen_id: mv.send_pen_id.then_some(mv.to_pen_id),
            version: Some(mv.version),
            ..Default::default()
        };
        match AnimalService::update(pool, actor, mv.animal_id, &req).await {
            Ok(_) => out.applied += 1,
            Err(e) => out.failed.push(ApplyFailure {
                ear: item.ear.clone(),
                to_code: mv.to_code.clone(),
                reason: e.to_string(),
            }),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_reads_only_old_pen_and_ear_and_tolerates_bom() {
        let data = "\u{feff}old_pen,ear,old_system_id,new_pen_location\nA03,010,x1,B01\nG03 , 011 ,x2,\n,,,\n";
        let rows = read_legacy_csv(data.as_bytes()).expect("parse csv");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].old_pen, "A03");
        assert_eq!(rows[0].ear, "010");
        assert_eq!(rows[1].old_pen, "G03");
        assert_eq!(rows[1].ear, "011");
    }

    #[test]
    fn csv_missing_required_column_is_rejected() {
        let err = read_legacy_csv("pen,ear\nA03,010\n".as_bytes()).expect_err("missing old_pen");
        assert!(err.to_string().contains("old_pen"));
    }
}
