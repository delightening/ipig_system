# iPig 系統 SLA 與可用性目標

> **版本**：1.0  
> **建立日期**：2026-03-01  
> **適用範圍**：正式環境服務水準定義  
> **相關文件**：[OPERATIONS.md](OPERATIONS.md)、[DR_RUNBOOK.md](DR_RUNBOOK.md)、[DEPLOYMENT.md](DEPLOYMENT.md)

---

## 1. 概述

本文件定義豬博士 iPig 系統的服務水準目標（SLO）與復原目標，供維運、容量規劃與 SOC2 合規參考。

---

## 2. 復原目標

| 指標 | 目標 | 說明 |
|------|------|------|
| **RPO (Recovery Point Objective)** | < 1 小時 | 最多可接受損失 1 小時內的資料；依每日備份頻率與保留政策達成 |
| **RTO (Recovery Time Objective)** | < 4 小時 | 從故障發生到服務恢復，目標 4 小時內完成 |
| **MTTR (Mean Time To Recovery)** | < 2 小時 | 平均復原時間目標；依 [DR_RUNBOOK.md](DR_RUNBOOK.md) 流程執行 |

### 2.1 達成方式

- **RPO**：備份腳本每日執行、必要時提高備份頻率；GPG 加密儲存
- **RTO**：依 DR Runbook 流程；定期演練以驗證可達成
- **驗證**：見 [DR_RUNBOOK.md](DR_RUNBOOK.md) 還原程序與演練紀錄

---

## 3. 可用性目標

| 指標 | 目標 | 說明 |
|------|------|------|
| **系統可用性** | ≥ 99% | 每月計畫外停機時間 < 7.2 小時 |
| **API 回應時間** | P95 < 500ms | 一般 API 端點（不含報表、匯出等重負載） |
| **健康檢查** | `/api/health` 可探測 | 供負載平衡與監控使用 |

### 3.1 排除項目

以下不計入可用性統計：

- 計畫性維護（提前公告）
- 使用者端網路問題
- 第三方服務（如 SMTP、Google Calendar）中斷

---

## 4. 監控與告警

| 項目 | 工具 | 說明 |
|------|------|------|
| 健康檢查 | `/api/health` | 含 DB、基本功能探測 |
| 效能指標 | `/metrics`、Prometheus | 請求延遲、錯誤率、DB Pool |
| 告警 | Alertmanager | 依規則通知 on-call |
| 儀表板 | Grafana | 視覺化監控 |

詳見 [DEPLOYMENT.md](DEPLOYMENT.md) 監控章節。

---

## 5. 檢討與更新

- SLA 目標應至少每年檢討一次
- 重大架構變更或合規要求變更時應更新
- 演練結果若顯示 RTO/RPO 無法達成，應調整流程或目標

---

*文件產出於 2026-03-01*
