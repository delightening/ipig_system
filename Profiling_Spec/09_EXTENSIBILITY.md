# 擴展性

> **版本**：5.0  
> **最後更新**：2026-02-17  
> **對象**：架構師、產品經理

---

## 1. 概覽

iPig 系統從設計之初即考量擴展性，支援模組化新增功能、資料表擴充及 API 延伸。

---

## 2. 已完成的擴展

### 2.1 ERP 進銷存系統
- ✅ 完整的產品/SKU/倉庫/儲位管理
- ✅ 採購、銷售、調撥、盤點等單據流程
- ✅ 庫存追蹤與報表
- ✅ 低庫存與效期警報

### 2.2 AUP 審查系統
- ✅ IACUC 計畫書線上提交
- ✅ 多層審查（Pre-Review → Vet → Committee）
- ✅ 審查意見與巢狀回覆
- ✅ 共同編輯者
- ✅ 版本控制與歷程追蹤
- ✅ PDF 匯出

### 2.3 變更申請 (Amendment)
- ✅ 計畫書核准後變更管理
- ✅ 重大/次要變更分類
- ✅ 獨立審查流程

### 2.4 人事管理 (HR)
- ✅ 出勤打卡
- ✅ 多級請假審核
- ✅ 加班管理
- ✅ 特休餘額管理
- ✅ Google 行事曆同步

### 2.5 安全與稽核
- ✅ Activity Logger 中間件
- ✅ 登入追蹤與帳號鎖定
- ✅ 工作階段管理與強制登出
- ✅ 安全警報偵測
- ✅ GeoIP 地理定位
- ✅ 安全儀表板（前端）

### 2.6 動物管理進階
- ✅ 血液檢查子系統（模板、組合、費用追蹤）
- ✅ 醫療資料匯出（PDF/Excel）
- ✅ 安樂死申請與核准流程
- ✅ 電子簽章（GLP 合規）
- ✅ 紀錄版本控制
- ✅ 獸醫建議系統
- ✅ 疼痛評估（Pain Assessment）
- ✅ **Pig → Animal 重新命名**（資料庫、API、前端全面重構）
- ✅ **動物狀態生命週期重構**（新增 euthanized/sudden_death/transferred 狀態）
- ✅ **動物轉讓流程**（6 步 API + Stepper UI + 資料隔離）
- ✅ **猝死登記**（animal_sudden_deaths 表 + API）
- ✅ **手寫電子簽章**（signature_pad + HandwrittenSignaturePad.tsx，4 場景）
- ✅ **資料隔離機制**（data-boundary API + 特權角色繞過）
- ✅ **資料庫遷移整合**（14 個合併為 8 個）

### 2.7 通知與排程
- ✅ Email 通知（SMTP）
- ✅ 站內通知系統
- ✅ 排程報表
- ✅ 通知路由（各模組獨立通知服務）
- ✅ **通知路由可配置化**（notification_routing 表 + Admin UI）

### 2.8 設施管理
- ✅ 多物種支援
- ✅ 設施 → 棟舍 → 區域 → 欄位 階層管理
- ✅ 部門管理

### 2.9 響應式設計
- ✅ 移動端自適應佈局
- ✅ 主要頁面移動端優化

---

## 3. 未來擴展規劃

### 3.1 動物物種擴展

**現狀**：系統已完成 pig → animal 重新命名，基礎架構支援多物種。設施管理已建立 species 表。

**規劃**：
- [ ] 將 `animals` 增加 `species_id` 欄位關聯至 `species` 表
- [ ] 動態表單依物種調整欄位（如：不同動物的品種列舉）
- [ ] 前端元件進一步通用化（依物種切換顯示）

### 3.2 多語系深化

**現狀**：已有 `i18next` 框架，支援 `zh-TW` 和 `en`。

**規劃**：
- [ ] 完善英文翻譯覆蓋率
- [ ] 後端錯誤訊息國際化
- [ ] PDF 匯出多語系支援

### 3.3 進階報表

**規劃**：
- [ ] 動物健康趨勢分析
- [ ] 實驗結果統計圖表
- [ ] 機構層級的 KPI 儀表板
- [ ] 自訂報表建構器

### 3.4 外部系統整合

**規劃**：
- [ ] 實驗室資訊管理系統 (LIMS) 介接
- [ ] 財務系統 ERP 對接
- [ ] 研究資料庫（如 ClinicalTrials.gov）串接
- [ ] 電子發票系統

### 3.5 AI / 自動化

**規劃**：
- [ ] 動物行為異常自動偵測
- [ ] 實驗排程智能建議
- [ ] 庫存需求預測
- [ ] 自然語言報表查詢

---

## 4. 擴展指南

### 4.1 新增後端模組

```
1. 定義 Model (models/new_module.rs)
   - 結構體定義、列舉
   
2. 實作 Service (services/new_module.rs)
   - 商業邏輯、資料存取

3. 建立 Handler (handlers/new_module.rs)
   - API 端點處理

4. 註冊路由 (routes.rs)
   - 加入路由群組

5. 建立遷移 (migrations/xxx_new_module.sql)
   - 資料表與索引

6. 新增權限 (002 migration seed 或 API)
   - 在 permissions 表加入權限
```

### 4.2 新增前端頁面

```
1. 建立頁面 (src/pages/NewModule/)
   - Page 組件、相關子組件

2. 建立 API 層 (src/api/newModule.ts)
   - API 呼叫函數

3. 建立型別 (src/types/newModule.ts)
   - TypeScript 型別定義

4. 註冊路由 (src/routes/)
   - 加入路由定義

5. 加入導航 (src/components/layout/)
   - 側邊欄連結

6. 權限控制
   - 使用 useAuth hook 檢查權限
```

### 4.3 新增通知類型

```
1. 定義通知類型 (notification_type ENUM)
2. 建立通知服務 (services/notification/new_type.rs)
3. 在相關 Service 中觸發通知
4. 前端處理新通知類型的顯示
```

---

## 5. 架構約束

| 約束 | 說明 |
|------|------|
| 向後相容 | 遷移檔案只新增，不修改舊遷移 |
| 權限檢查 | 所有新端點必須有權限控制 |
| 稽核追蹤 | Activity Logger 自動記錄所有操作 |
| 軟刪除 | 重要實體使用軟刪除模式 |
| 時區處理 | 所有時間使用 TIMESTAMPTZ |
| 輸入驗證 | 前端 Zod + 後端 Validator |

---

*上一章：[出勤模組](./08_ATTENDANCE_MODULE.md)*
