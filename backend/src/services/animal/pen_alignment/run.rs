//! 欄位對齊的 I/O 部分：讀舊系統 CSV、載入快照、逐隻透過 `AnimalService::update` 寫入。
//!
//! 寫入**只**經 `AnimalService::update`：同一交易內 UPDATE animals、重算舊欄與新欄的
//! current_count、寫 `ANIMAL_UPDATE` 稽核（與一般的欄位變更同一事件類別）。
//! 本檔沒有任何會改 animals / pens 的裸 SQL。

use chrono::NaiveDate;
use sqlx::PgPool;

use super::super::AnimalService;
use super::plan::{LegacyRow, Plan};
use crate::{
    middleware::ActorContext,
    models::UpdateAnimalRequest,
    repositories::pen_alignment::{AnimalSnap, PenAlignmentRepository, PenSnap},
    AppError, Result,
};

/// 解析舊系統 CSV。只取 `old_pen`、`ear`、`birth` 三欄（其餘欄位忽略）；
/// 標頭容許 UTF-8 BOM 與前後空白。`birth`（YYYY-MM-DD）必填：缺欄、
/// 任一有資料的列空白或格式錯誤，整份拒收（出生日期用來確認同耳號是同一隻）。
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
    let birth_idx = find("birth")?;

    let mut rows = Vec::new();
    for (i, rec) in rdr.records().enumerate() {
        let rec = rec
            .map_err(|e| AppError::BadRequest(format!("CSV 第 {} 筆資料列解析失敗：{e}", i + 1)))?;
        // 只略過「整列皆空」的列（試算表尾端常見）。不能只看耳號與舊欄位：有出生日期等其他資料、
        // 卻漏填耳號與舊欄位的列若被靜默丟掉，不會出現在報告的「待人工」，操作者會誤以為整份
        // 清單都評估過。這類列照常進入計畫，因對不到動物而列為待人工。
        if rec.iter().all(|cell| cell.trim().is_empty()) {
            continue;
        }
        let ear = rec.get(ear_idx).unwrap_or_default().trim().to_string();
        let old_pen = rec.get(pen_idx).unwrap_or_default().trim().to_string();
        let birth_raw = rec.get(birth_idx).unwrap_or_default().trim();
        if birth_raw.is_empty() {
            return Err(AppError::BadRequest(format!(
                "CSV 第 {} 筆資料列 birth 為空（出生日期必填）",
                i + 1
            )));
        }
        let birth = NaiveDate::parse_from_str(birth_raw, "%Y-%m-%d").map_err(|e| {
            AppError::BadRequest(format!(
                "CSV 第 {} 筆資料列 birth 日期無法解析（需 YYYY-MM-DD）：{e}",
                i + 1
            ))
        })?;
        rows.push(LegacyRow {
            old_pen,
            ear,
            birth,
        });
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
    fn csv_reads_only_old_pen_ear_birth_and_tolerates_bom() {
        let data = "\u{feff}old_pen,ear,birth,old_system_id,new_pen_location\nA03,010,2025-01-31,x1,B01\nG03 , 011 , 2025-02-01 ,x2,\n,,,,\n";
        let rows = read_legacy_csv(data.as_bytes()).expect("parse csv");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].old_pen, "A03");
        assert_eq!(rows[0].ear, "010");
        assert_eq!(rows[0].birth.to_string(), "2025-01-31");
        assert_eq!(rows[1].old_pen, "G03");
        assert_eq!(rows[1].ear, "011");
        assert_eq!(rows[1].birth.to_string(), "2025-02-01");
    }

    #[test]
    fn csv_missing_birth_column_blank_or_bad_birth_is_rejected() {
        let no_col = read_legacy_csv("old_pen,ear\nA03,010\n".as_bytes()).expect_err("no birth");
        assert!(no_col.to_string().contains("birth"));
        let blank = read_legacy_csv("old_pen,ear,birth\nA03,010,\n".as_bytes()).expect_err("blank");
        assert!(blank.to_string().contains("birth"));
        let bad = read_legacy_csv("old_pen,ear,birth\nA03,010,2025/01/31\n".as_bytes())
            .expect_err("bad date");
        assert!(bad.to_string().contains("birth"));
    }

    /// 回歸：有資料卻漏填耳號與舊欄位的列，過去因「耳號與舊欄位同時為空」被靜默丟掉，
    /// 不會出現在報告的待人工。現在只略過整列皆空的列。
    #[test]
    fn row_with_data_but_no_ear_and_no_pen_is_kept_not_dropped() {
        let head = "old_pen,ear,birth,old_system_id,new_pen_location\n";
        // 只有生日與其他欄 → 保留（耳號與舊欄位為空），交給計畫階段列為待人工
        let kept =
            read_legacy_csv(format!("{head},,2025-01-31,x1,B01\n").as_bytes()).expect("keep");
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].ear, "");
        assert_eq!(kept[0].old_pen, "");
        assert_eq!(kept[0].birth.to_string(), "2025-01-31");
        // 整列皆空（含只有空白）仍照舊略過
        let blank = read_legacy_csv(format!("{head},,,,\n , ,  ,,\n").as_bytes()).expect("blank");
        assert!(blank.is_empty());
    }

    /// 有其他資料卻缺出生日期的列：整份拒收並指出是哪一筆，而不是靜默略過。
    #[test]
    fn row_with_other_data_but_no_birth_is_rejected_with_line_number() {
        let head = "old_pen,ear,birth,old_system_id,new_pen_location\n";
        let err = read_legacy_csv(format!("{head}A03,010,2025-01-31,x1,\n,,,x9,\n").as_bytes())
            .expect_err("second row has data but no birth");
        let msg = err.to_string();
        assert!(msg.contains("第 2 筆"), "{msg}");
        assert!(msg.contains("birth"), "{msg}");
    }

    #[test]
    fn csv_missing_required_column_is_rejected() {
        let err = read_legacy_csv("pen,ear,birth\nA03,010,2025-01-31\n".as_bytes())
            .expect_err("missing old_pen");
        assert!(err.to_string().contains("old_pen"));
    }
}
