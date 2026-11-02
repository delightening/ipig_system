//! 犧牲補登的 I/O：讀 CSV、查候選、逐隻補登、輸出報告。
//!
//! 本檔沒有任何會改 animals / animal_sacrifices 的裸 SQL；寫入只經
//! `AnimalService::update`（補 IACUC）與 `AnimalMedicalService::upsert_sacrifice`。
//! 唯一的直接寫入是補登後對 `pens.current_count` 的重算（見 repository）。
//! 報告含耳號等營運資料：只印到 stdout，請勿存進 repo。

use std::fmt::Write as _;
use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use super::super::AnimalService;
use super::plan::{decide, Decision, LegacySacRow};
use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{CreateSacrificeRequest, UpdateAnimalRequest},
    repositories::sacrifice_backfill::SacrificeBackfillRepository,
    services::{
        access::{AnimalWrite, Scoped},
        AnimalMedicalService,
    },
    AppError, Result,
};

fn opt(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

/// 解析輸入 CSV。欄位：ear, legacy_id, birth, iacuc, sac_date, zoletil, electro, bleed,
/// other, sampling, sampling_other, blood_ml（日期 YYYY-MM-DD；electro/bleed 為 0/1）。
pub fn read_legacy_csv<R: std::io::Read>(reader: R) -> Result<Vec<LegacySacRow>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);
    let headers = rdr
        .headers()
        .map_err(|e| AppError::BadRequest(format!("讀取 CSV 標頭失敗：{e}")))?
        .clone();
    let idx = |name: &str| {
        headers
            .iter()
            .position(|h| {
                h.trim_start_matches('\u{feff}')
                    .trim()
                    .eq_ignore_ascii_case(name)
            })
            .ok_or_else(|| AppError::BadRequest(format!("CSV 缺少 `{name}` 欄")))
    };
    let (i_ear, i_lid, i_birth, i_iacuc) =
        (idx("ear")?, idx("legacy_id")?, idx("birth")?, idx("iacuc")?);
    let (i_sac, i_zol, i_el, i_bl) = (
        idx("sac_date")?,
        idx("zoletil")?,
        idx("electro")?,
        idx("bleed")?,
    );
    let (i_oth, i_smp, i_smo, i_blood) = (
        idx("other")?,
        idx("sampling")?,
        idx("sampling_other")?,
        idx("blood_ml")?,
    );

    let date = |s: &str, what: &str, line: usize| -> Result<Option<NaiveDate>> {
        match opt(s) {
            None => Ok(None),
            Some(t) => NaiveDate::parse_from_str(&t, "%Y-%m-%d")
                .map(Some)
                .map_err(|e| {
                    AppError::BadRequest(format!("CSV 第 {line} 筆 {what} 日期無法解析：{e}"))
                }),
        }
    };

    let mut rows = Vec::new();
    for (n, rec) in rdr.records().enumerate() {
        let line = n + 1;
        let rec =
            rec.map_err(|e| AppError::BadRequest(format!("CSV 第 {line} 筆解析失敗：{e}")))?;
        let get = |i: usize| rec.get(i).unwrap_or_default();
        if get(i_ear).trim().is_empty() {
            continue;
        }
        let birth = date(get(i_birth), "birth", line)?
            .ok_or_else(|| AppError::BadRequest(format!("CSV 第 {line} 筆缺 birth")))?;
        let blood_ml = match opt(get(i_blood)) {
            None => None,
            Some(t) => Some(Decimal::from_str(&t).map_err(|e| {
                AppError::BadRequest(format!("CSV 第 {line} 筆 blood_ml 不是數字：{e}"))
            })?),
        };
        rows.push(LegacySacRow {
            ear: get(i_ear).trim().to_string(),
            legacy_id: get(i_lid).trim().to_string(),
            birth,
            iacuc: opt(get(i_iacuc)),
            sac_date: date(get(i_sac), "sac_date", line)?,
            zoletil: opt(get(i_zol)),
            electro: get(i_el).trim() == "1",
            bleed: get(i_bl).trim() == "1",
            other: opt(get(i_oth)),
            sampling: opt(get(i_smp)),
            sampling_other: opt(get(i_smo)),
            blood_ml,
        });
    }
    Ok(rows)
}

/// 以 email 找啟用中的使用者，並載入其**真實**角色（授權檢查要用，不偽造）。
pub async fn load_actor(pool: &PgPool, email: &str) -> Result<ActorContext> {
    let id: Uuid = sqlx::query_scalar(
        "SELECT id FROM users WHERE lower(email) = lower($1) AND is_active = true LIMIT 1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("找不到啟用中的 actor：{email}")))?;
    let roles = SacrificeBackfillRepository::user_role_codes(pool, id).await?;
    Ok(ActorContext::User(CurrentUser {
        id,
        email: email.to_string(),
        roles,
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    }))
}

pub struct Evaluated {
    pub row: LegacySacRow,
    pub decision: Decision,
}

/// 逐筆查候選並判斷（唯讀）。
pub async fn evaluate(pool: &PgPool, rows: Vec<LegacySacRow>) -> Result<Vec<Evaluated>> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let cands = SacrificeBackfillRepository::find_candidates(pool, &row.ear, row.birth).await?;
        let decision = decide(&row, &cands);
        out.push(Evaluated { row, decision });
    }
    Ok(out)
}

#[derive(Debug, Clone)]
pub enum RowOutcome {
    Done,
    Skipped,
    Failed(String),
}

fn sacrifice_request(row: &LegacySacRow) -> CreateSacrificeRequest {
    CreateSacrificeRequest {
        sacrifice_date: row.sac_date,
        zoletil_dose: row.zoletil.clone(),
        method_electrocution: row.electro,
        method_bloodletting: row.bleed,
        method_other: row.other.clone(),
        sampling: row.sampling.clone(),
        sampling_other: row.sampling_other.clone(),
        blood_volume_ml: row.blood_ml,
        confirmed_sacrifice: true,
    }
}

async fn apply_one(
    pool: &PgPool,
    actor: &ActorContext,
    row: &LegacySacRow,
    animal_id: Uuid,
    version: i32,
    pen_id: Option<Uuid>,
    set_iacuc: Option<&String>,
) -> Result<()> {
    if let Some(iacuc) = set_iacuc {
        let req = UpdateAnimalRequest {
            iacuc_no: Some(iacuc.clone()),
            version: Some(version),
            ..Default::default()
        };
        AnimalService::update(pool, actor, animal_id, &req).await?;
    }
    let user = actor.require_user()?;
    let scope = Scoped::<AnimalWrite>::authorize(pool, user, animal_id).await?;
    AnimalMedicalService::upsert_sacrifice(pool, actor, scope, &sacrifice_request(row)).await?;
    if let Some(pid) = pen_id {
        SacrificeBackfillRepository::recount_pen(pool, pid).await?;
    }
    Ok(())
}

/// 逐隻補登；單隻失敗不中止。IACUC 補寫與犧牲補登是兩個交易，
/// 若中途失敗，重跑即可（IACUC 已補上、判斷仍為 Ready，冪等）。
pub async fn apply(pool: &PgPool, actor: &ActorContext, items: &[Evaluated]) -> Vec<RowOutcome> {
    let mut out = Vec::with_capacity(items.len());
    for it in items {
        let r = match &it.decision {
            Decision::Ready {
                animal_id,
                version,
                pen_id,
                set_iacuc,
                ..
            } => {
                match apply_one(
                    pool,
                    actor,
                    &it.row,
                    *animal_id,
                    *version,
                    *pen_id,
                    set_iacuc.as_ref(),
                )
                .await
                {
                    Ok(()) => RowOutcome::Done,
                    Err(e) => RowOutcome::Failed(e.to_string()),
                }
            }
            Decision::AlreadyDone { .. } | Decision::Manual(_) => RowOutcome::Skipped,
        };
        out.push(r);
    }
    out
}

/// 報告：每隻一行 + 彙總。`outcomes` 為 `None` 表示 dry-run。
pub fn render_report(items: &[Evaluated], outcomes: Option<&[RowOutcome]>) -> String {
    let mut s = String::new();
    let (mut ready, mut done, mut manual) = (0, 0, 0);
    let _ = writeln!(s, "耳號 | 舊#id | 結論 | 說明");
    for (i, it) in items.iter().enumerate() {
        let (verdict, note) = match &it.decision {
            Decision::Ready {
                from_status,
                set_iacuc,
                ..
            } => {
                ready += 1;
                let iacuc = set_iacuc
                    .as_deref()
                    .map_or("IACUC 不動".to_string(), |v| format!("先補 IACUC={v}"));
                (
                    "可補登".to_string(),
                    format!(
                        "{from_status}→euthanized；犧牲日 {}；{iacuc}",
                        it.row.sac_date.map_or("-".to_string(), |d| d.to_string())
                    ),
                )
            }
            Decision::AlreadyDone { .. } => {
                done += 1;
                (
                    "已完成".to_string(),
                    "已有犧牲紀錄且為 euthanized".to_string(),
                )
            }
            Decision::Manual(why) => {
                manual += 1;
                ("待人工".to_string(), why.clone())
            }
        };
        let result = match outcomes.and_then(|o| o.get(i)) {
            Some(RowOutcome::Done) => "；執行：成功".to_string(),
            Some(RowOutcome::Failed(e)) => format!("；執行：失敗 {e}"),
            _ => String::new(),
        };
        let _ = writeln!(
            s,
            "{} | {} | {} | {}{}",
            it.row.ear, it.row.legacy_id, verdict, note, result
        );
    }
    let _ = writeln!(
        s,
        "\n彙總：可補登 {ready}／已完成 {done}／待人工 {manual}／共 {}",
        items.len()
    );
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_parses_quoted_commas_blanks_and_bom() {
        let data = "\u{feff}ear,legacy_id,birth,iacuc,sac_date,zoletil,electro,bleed,other,sampling,sampling_other,blood_ml\n\
                    AAA,10,2025-01-01,PIG-1,2026-01-02,4,0,1,KCl,\"心, 脾\",,\n\
                    BBB,11,2025-02-02,,,,0,0,,,,12.5\n";
        let rows = read_legacy_csv(data.as_bytes()).expect("parse");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].sampling.as_deref(), Some("心, 脾"));
        assert!(rows[0].bleed && !rows[0].electro);
        assert_eq!(rows[1].iacuc, None);
        assert_eq!(rows[1].sac_date, None);
        assert_eq!(rows[1].blood_ml, Some(Decimal::new(125, 1)));
    }

    #[test]
    fn csv_bad_date_is_rejected() {
        let data = "ear,legacy_id,birth,iacuc,sac_date,zoletil,electro,bleed,other,sampling,sampling_other,blood_ml\nAAA,1,2025/01/01,,,,0,0,,,,\n";
        assert!(read_legacy_csv(data.as_bytes()).is_err());
    }
}
