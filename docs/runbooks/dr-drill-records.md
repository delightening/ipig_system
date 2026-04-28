# DR Drill 年度演練紀錄

> **用途**：保存歷年災難復原（Disaster Recovery）演練紀錄，作為 GLP §10 / SOC 2 A1.2 / ISO 27001 A.8.13 / 21 CFR §11.10(c) 還原能力證明。
> **適用範圍**：本系統 production / staging 環境之備份還原與服務復原驗證。
> **維護者**：SRE lead + QAU。每年至少 1 次（建議每季 1 次），演練後 7 個工作日內必須回填本表。
> **配套文件**：[`DR_DRILL_CHECKLIST.md`](DR_DRILL_CHECKLIST.md)（演練檢核項目）、[`DR_RUNBOOK.md`](DR_RUNBOOK.md)（執行步驟）

---

## 1. 演練頻率與要求

| 項目 | 要求 |
|---|---|
| **最低頻率** | 每年 1 次（GLP / FDA pre-audit 強制） |
| **建議頻率** | 每季 1 次（SOC 2 Type II 持續性證據） |
| **RTO 目標** | < 4 小時 |
| **RPO 目標** | < 1 小時 |
| **演練範圍** | 至少 1 次「完整毀損模擬」（非僅 backup restore） |
| **參與角色** | SRE on-call + admin + QAU 觀察員（≥1 名） |
| **通過標準** | 所有 DR_DRILL_CHECKLIST.md 項目綠燈 + RTO / RPO 達標 |

---

## 2. 演練紀錄表

| 日期 | 主持人 | 涵蓋範圍 | 發現問題 | 修補時程 | 通過? | 報告連結 |
|---|---|---|---|---|---|---|
| 2026-XX-XX | TBD | TBD | — | — | — | — |
| 2027-XX-XX | TBD | TBD | — | — | — | — |
| 2028-XX-XX | TBD | TBD | — | — | — | — |

> **欄位說明**：
> - **涵蓋範圍**：例「DB backup restore only」/「完整 cold-start」/「跨 region failover」
> - **發現問題**：未通過或勉強通過的 checklist 項目；無問題填「無」
> - **修補時程**：發現問題的修補 PR / issue 與預計完成日
> - **通過?**：✅ / 🟡（部分通過，已修補）/ ❌（未通過，需重演）
> - **報告連結**：相對路徑指向 `docs/audit/dr-drills/YYYY-MM-DD.md`（**TODO[使用者]**：報告子目錄結構待確認）

---

## 3. 每次演練必填欄位（報告 template）

新建 `docs/audit/dr-drills/YYYY-MM-DD.md` 時建議含：

```markdown
# DR Drill — YYYY-MM-DD

## 演練資訊
- 主持人：
- 參與者：
- 開始時間：YYYY-MM-DD HH:MM (GMT+8)
- 結束時間：YYYY-MM-DD HH:MM (GMT+8)
- 涵蓋範圍：

## RTO / RPO 量測
- 偵測到中斷 → 開始復原：M 分鐘
- 開始復原 → 服務恢復：M 分鐘
- 資料遺失視窗（最後 backup → 中斷時刻）：M 分鐘
- RTO 達標？✅ / ❌
- RPO 達標？✅ / ❌

## DR_DRILL_CHECKLIST 結果
（逐項打勾並附 evidence 連結 / screenshot）

## 發現問題與後續行動
| # | 問題 | 嚴重度 | 後續 action item / PR | 負責人 | 完成日 |

## QAU 觀察員簽核
- 觀察員：
- 簽核時間：
- 結論：通過 / 部分通過 / 未通過
```

---

## 4. 反向引用

- 演練檢核項目：[`DR_DRILL_CHECKLIST.md`](DR_DRILL_CHECKLIST.md)
- 演練執行步驟：[`DR_RUNBOOK.md`](DR_RUNBOOK.md)
- Traceability：[`../glp/traceability-matrix.md`](../glp/traceability-matrix.md) §11.10(c)
- 合規對應：[`../R26_compliance_requirements.md`](../R26_compliance_requirements.md) R26-3
