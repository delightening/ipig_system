# iPig ERP 系統 — AWS 部署方案與定價分析

> 更新日期：2026-03-22
> 基準區域：us-east-1（美東），價格以 USD 計算，TWD 匯率以 1 USD ≈ 30.8 TWD 估算

---

## 目錄

1. [工作負載分析](#1-工作負載分析)
2. [EC2 Savings Plan 定價明細](#2-ec2-savings-plan-定價明細)
3. [RDS Reserved Instance 定價明細](#3-rds-reserved-instance-定價明細)
4. [部署方案比較](#4-部署方案比較)
5. [與自建工業電腦比較](#5-與自建工業電腦比較)
6. [Savings Plan 購買指南](#6-savings-plan-購買指南)

---

## 1. 工作負載分析

| 服務 | CPU 特性 | 記憶體特性 | I/O 特性 |
|------|----------|-----------|----------|
| PostgreSQL | 中等，查詢併發 | 重要，shared_buffers 需大 | SSD 必須，IOPS 關鍵 |
| Rust API (Axum) | 低～中，async 多工 | 低（Rust 很省） | 低 |
| Gotenberg/Chromium | 突發高，PDF 生成 | 中高，每次渲染 ~200MB | 低 |
| Prometheus + Loki | 低，持續寫入 | 中 | 中，持續寫 TSDB |
| Nginx | 極低 | 極低 | 低 |

瓶頸排序：磁碟 I/O > 記憶體 > CPU > 網路

---

## 2. EC2 Savings Plan 定價明細

### t3.xlarge (4 vCPU / 16GB) — 單機跑全部 Docker 服務

| 方案 | 每小時 (USD) | 每月 (USD) | 年費 (USD) | vs On-Demand |
|------|-------------|-----------|-----------|-------------|
| On-Demand | $0.1664 | $121.5 | $1,458 | — |
| 1yr No Upfront | $0.1040 | $75.9 | $911 | -37% |
| 1yr Partial Upfront | $0.1087 | $79.4 | $952 | -35% |
| **1yr All Upfront** | **$0.0888** | **$64.8** | **$778** | **-47%** |
| 3yr No Upfront | $0.0720 | $52.6 | $631 | -57% |
| 3yr Partial Upfront | $0.0749 | $54.7 | $656 | -55% |
| **3yr All Upfront** | **$0.0627** | **$45.8** | **$549** | **-62%** |

### t3.large (2 vCPU / 8GB) — 搭配 RDS 時使用

| 方案 | 每小時 (USD) | 每月 (USD) | 年費 (USD) | vs On-Demand |
|------|-------------|-----------|-----------|-------------|
| On-Demand | $0.0832 | $59.9 | $719 | — |
| 1yr No Upfront | $0.0742 | $54.2 | $650 | -10% |
| **1yr All Upfront** | **$0.0635** | **$46.4** | **$556** | **-24%** |
| 3yr No Upfront | $0.0620 | $45.3 | $543 | -25% |
| **3yr All Upfront** | **$0.0519** | **$37.9** | **$455** | **-37%** |

---

## 3. RDS Reserved Instance 定價明細

### db.t4g.medium (2 vCPU / 4GB) — PostgreSQL

| 方案 | 每小時 (USD) | 每月 (USD) | 年費 (USD) | vs On-Demand |
|------|-------------|-----------|-----------|-------------|
| On-Demand | $0.065 | $47.5 | $570 | — |
| **1yr No Upfront** | **$0.047** | **$34.3** | **$412** | **-28%** |
| 1yr Partial Upfront | $0.0558 | $40.7 | $489 | -14% |
| 1yr All Upfront | $0.0525 | $38.3 | $460 | -19% |

> 注意：db.t4g.medium 無 3 年 Reserved Instance 選項。

---

## 4. 部署方案比較

### 組合 A：EC2 單機（全部跑 Docker，架構零改動）

| 項目 | On-Demand | 1yr All Upfront | 3yr All Upfront |
|------|-----------|-----------------|-----------------|
| EC2 t3.xlarge | $121.5 | $64.8 | $45.8 |
| EBS 330GB gp3 | $26.4 | $26.4 | $26.4 |
| 資料傳輸 100GB | $9.0 | $9.0 | $9.0 |
| **月費合計** | **$156.9** | **$100.2** | **$81.2** |
| **年費合計** | **$1,883** | **$1,202** | **$974** |
| **年費 TWD** | **~58,000** | **~37,000** | **~30,000** |

### 組合 B：EC2 + RDS（DB 託管，推薦正式部署）

| 項目 | On-Demand | 1yr All Upfront |
|------|-----------|-----------------|
| EC2 t3.large | $59.9 | $46.4 |
| RDS db.t4g.medium | $47.5 | $38.3 |
| RDS 儲存 50GB gp3 | $5.8 | $5.8 |
| EBS 100GB gp3 | $8.0 | $8.0 |
| 資料傳輸 100GB | $9.0 | $9.0 |
| **月費合計** | **$130.2** | **$107.5** |
| **年費合計** | **$1,562** | **$1,290** |
| **年費 TWD** | **~48,000** | **~40,000** |

### 組合 C：Lightsail（最便宜，價格固定）

| 項目 | 月費 (USD) | 年費 (USD) | 年費 TWD |
|------|-----------|-----------|---------|
| Lightsail $80 方案 (4C/16GB/320GB) | $80 | $960 | ~29,600 |
| 額外磁碟 100GB | $10 | $120 | ~3,700 |
| **合計** | **$90** | **$1,080** | **~33,300** |

### 總覽

| 方案 | 月費 USD | 年費 TWD | 3 年總成本 TWD | 適合情境 |
|------|---------|---------|--------------|---------|
| A. EC2 單機 3yr All Upfront | $81 | ~30,000 | ~90,000 | 確定長期使用、不改架構 |
| A. EC2 單機 1yr All Upfront | $100 | ~37,000 | ~111,000 | 想保留彈性 |
| B. EC2+RDS 1yr All Upfront | $108 | ~40,000 | ~120,000 | 想省運維、DB 自動備份 |
| C. Lightsail | $90 | ~33,300 | ~99,900 | 最簡單、價格透明 |

---

## 5. 與自建工業電腦比較

| 項目 | 工業電腦（自建） | AWS EC2 3yr | AWS Lightsail |
|------|----------------|-------------|---------------|
| 首年成本 | ~35,000 (設備) + ~3,700 (電費) = ~38,700 | ~30,000 | ~33,300 |
| 第 2 年起年費 | ~3,700 (電費) | ~30,000 | ~33,300 |
| 3 年總成本 TWD | **~45,000–60,000** | ~90,000 | ~99,900 |
| 需自管項目 | 硬體、網路、電力、UPS、OS | OS、Docker | OS、Docker |
| 外部存取 | 需固定 IP / DDNS | 內建 Elastic IP | 內建固定 IP |
| 高可用性 | 無（單點故障） | 可搭配 Multi-AZ | 無 |
| 擴充彈性 | 低（換硬體） | 高（換機型） | 低 |

> 損益平衡點：工業電腦約在第 1 年後開始比 AWS 方案划算（前提是辦公室有穩定網路與電力）。

---

## 6. Savings Plan 購買指南

### Savings Plan 類型選擇

| 類型 | 折扣幅度 | 彈性 | 建議 |
|------|---------|------|------|
| **Compute Savings Plans** | 最高 ~66% | 可跨機型、跨區域、含 Lambda/Fargate | 推薦，最彈性 |
| EC2 Instance Savings Plans | 最高 ~72% | 綁定特定機型家族 + 區域 | 確定不換機型時用 |

### 購買步驟

1. 進入 AWS Console → Cost Management → Savings Plans
2. 點選「Purchase Savings Plans」
3. 選擇類型：**Compute Savings Plans**（推薦）
4. 填入承諾金額（每小時）：
   - 組合 A (t3.xlarge)：`$0.0627`（3yr All Upfront）或 `$0.0888`（1yr All Upfront）
   - 組合 B (t3.large)：`$0.0519`（3yr）或 `$0.0635`（1yr）
5. 選擇期限：1 年或 3 年
6. 選擇付款方式：All Upfront（最省）
7. 確認購買

### 注意事項

- Savings Plan 一經購買**無法取消或退款**
- 建議先用 On-Demand 跑 1–2 個月，觀察實際用量後再購買
- 承諾金額超出實際用量的部分仍會被收費
- Compute Savings Plans 可自動套用到 EC2、Lambda、Fargate
- 價格以購買時為準，合約期間內不受 AWS 調價影響

---

## 參考資料

- [AWS EC2 On-Demand 定價](https://aws.amazon.com/ec2/pricing/on-demand/)
- [AWS Savings Plans 定價](https://aws.amazon.com/savingsplans/compute-pricing/)
- [AWS Savings Plans 說明文件](https://docs.aws.amazon.com/savingsplans/latest/userguide/what-is-savings-plans.html)
- [AWS RDS PostgreSQL 定價](https://aws.amazon.com/rds/postgresql/pricing/)
- [AWS Lightsail 定價](https://aws.amazon.com/lightsail/pricing/)
- [EC2 Instance 比較工具 (Vantage)](https://instances.vantage.sh/)
- [AWS Pricing Calculator](https://calculator.aws/)
