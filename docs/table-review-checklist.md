# 表格逐一審查清單

共 85 個檔案，分成三級完成狀態：

- **✅ 完整 RWD** — `@container` queries + Card layout（< 600px），已做人工 Q1-Q4 討論
- **🔧 批次修復** — 移除 `[&>div]:overflow-x-hidden` bug，表格現在可以橫向捲動（不被裁切）；未加 Card layout
- **⏳ 待改** — 尚未處理

人工驗收時：
- ✅ 項目在瀏覽器檢查 375/600/768/1024/1280 五個寬度
- 🔧 項目只需驗證在窄螢幕可橫向捲動（若 UX 不佳再升級為 ✅）

---

## Animal — `frontend/src/components/animal/` (8/10)

### Tabs (8/8 ✅)

- [x] ✅ [BloodTestTab.tsx](../frontend/src/components/animal/BloodTestTab.tsx) — 6 欄，2 斷點 (600px)
- [x] ✅ [ObservationsTab.tsx](../frontend/src/components/animal/ObservationsTab.tsx) — 8 欄，3 斷點 (600/720px)
- [x] ✅ [PainAssessmentTab.tsx](../frontend/src/components/animal/PainAssessmentTab.tsx) — 12 欄，3 斷點 (600/810px)
- [x] ✅ [PathologyTab.tsx](../frontend/src/components/animal/PathologyTab.tsx) — 4 欄，2 斷點
- [x] ✅ [SurgeriesTab.tsx](../frontend/src/components/animal/SurgeriesTab.tsx) — 8 欄，3 斷點 (600/690px)
- [x] ✅ [VaccinationsTab.tsx](../frontend/src/components/animal/VaccinationsTab.tsx) — 6 欄，3 斷點 (600/690px)
- [x] ✅ [VetRecommendationsTab.tsx](../frontend/src/components/animal/VetRecommendationsTab.tsx) — 4 欄，2 斷點
- [x] ✅ [WeightsTab.tsx](../frontend/src/components/animal/WeightsTab.tsx) — 5+ 欄，3 斷點 (600/620px)

### Blood Test Dialogs (2/2 ✅)

- [x] ✅ [blood-test/BloodTestDetailDialog.tsx](../frontend/src/components/animal/blood-test/BloodTestDetailDialog.tsx) — 6 欄，3 斷點 (500/650/750px)
- [x] ✅ [blood-test/BloodTestFormDialog.tsx](../frontend/src/components/animal/blood-test/BloodTestFormDialog.tsx) — 7 欄，2 斷點 (700px)

---

## Protocol Tabs — `frontend/src/components/protocol/` (7/7 ✅)

- [x] ✅ [AmendmentsTab.tsx](../frontend/src/components/protocol/AmendmentsTab.tsx) — 6 欄，3 斷點 (600/740px)
- [x] ✅ [AttachmentsTab.tsx](../frontend/src/components/protocol/AttachmentsTab.tsx) — 5 欄，2 斷點
- [x] ✅ [CoEditorsTab.tsx](../frontend/src/components/protocol/CoEditorsTab.tsx) — 4 欄，2 斷點
- [x] ✅ [ReviewersTab.tsx](../frontend/src/components/protocol/ReviewersTab.tsx) — 5 欄，3 斷點 (600/780px)
- [x] ✅ [VersionsTab.tsx](../frontend/src/components/protocol/VersionsTab.tsx) — 3-4 欄，2 斷點
- [x] ✅ [comments/CommentsTableView.tsx](../frontend/src/components/protocol/comments/CommentsTableView.tsx) — 3-4 欄，2 斷點
- [x] ✅ [content-sections/PersonnelSection.tsx](../frontend/src/components/protocol/content-sections/PersonnelSection.tsx) — 6 欄，3 斷點 (600/630/810px)

---

## Admin — `frontend/src/pages/admin/` (1/31)

- [ ] 🔧 [AnimalFieldCorrectionsPage.tsx](../frontend/src/pages/admin/AnimalFieldCorrectionsPage.tsx)
- [ ] 🔧 [ChangeControlPage.tsx](../frontend/src/pages/admin/ChangeControlPage.tsx)
- [ ] 🔧 [CompetencyAssessmentPage.tsx](../frontend/src/pages/admin/CompetencyAssessmentPage.tsx)
- [ ] 🔧 [DocumentControlPage.tsx](../frontend/src/pages/admin/DocumentControlPage.tsx)
- [ ] 🔧 [EnvironmentMonitoringPage.tsx](../frontend/src/pages/admin/EnvironmentMonitoringPage.tsx)
- [ ] 🔧 [FormulationRecordsPage.tsx](../frontend/src/pages/admin/FormulationRecordsPage.tsx)
- [x] ✅ [InvitationsPage.tsx](../frontend/src/pages/admin/InvitationsPage.tsx) — 7 欄，3 斷點 (600/750px)
- [ ] 🔧 [ManagementReviewPage.tsx](../frontend/src/pages/admin/ManagementReviewPage.tsx)
- [ ] 🔧 [QAInspectionPage.tsx](../frontend/src/pages/admin/QAInspectionPage.tsx)
- [ ] 🔧 [QANonConformancePage.tsx](../frontend/src/pages/admin/QANonConformancePage.tsx)
- [ ] 🔧 [QASchedulePage.tsx](../frontend/src/pages/admin/QASchedulePage.tsx)
- [ ] 🔧 [QASopPage.tsx](../frontend/src/pages/admin/QASopPage.tsx)
- [ ] 🔧 [QAUDashboardPage.tsx](../frontend/src/pages/admin/QAUDashboardPage.tsx)
- [ ] 🔧 [RiskRegisterPage.tsx](../frontend/src/pages/admin/RiskRegisterPage.tsx)
- [ ] 🔧 [StudyFinalReportPage.tsx](../frontend/src/pages/admin/StudyFinalReportPage.tsx)
- [ ] 🔧 [NotificationRouting/components/RoutingTable.tsx](../frontend/src/pages/admin/NotificationRouting/components/RoutingTable.tsx)
- [ ] 🔧 [components/AiApiKeySection.tsx](../frontend/src/pages/admin/components/AiApiKeySection.tsx)
- [ ] 🔧 [components/AnnualPlanTabContent.tsx](../frontend/src/pages/admin/components/AnnualPlanTabContent.tsx)
- [ ] 🔧 [components/AuditActivitiesTab.tsx](../frontend/src/pages/admin/components/AuditActivitiesTab.tsx)
- [ ] 🔧 [components/AuditAlertsTab.tsx](../frontend/src/pages/admin/components/AuditAlertsTab.tsx)
- [ ] 🔧 [components/AuditLogTable.tsx](../frontend/src/pages/admin/components/AuditLogTable.tsx)
- [ ] 🔧 [components/AuditLoginsTab.tsx](../frontend/src/pages/admin/components/AuditLoginsTab.tsx)
- [ ] 🔧 [components/AuditSessionsTab.tsx](../frontend/src/pages/admin/components/AuditSessionsTab.tsx)
- [ ] 🔧 [components/BuildingTab.tsx](../frontend/src/pages/admin/components/BuildingTab.tsx)
- [ ] 🔧 [components/DepartmentTab.tsx](../frontend/src/pages/admin/components/DepartmentTab.tsx)
- [ ] 🔧 [components/FacilityTab.tsx](../frontend/src/pages/admin/components/FacilityTab.tsx)
- [ ] 🔧 [components/PenTab.tsx](../frontend/src/pages/admin/components/PenTab.tsx)
- [ ] 🔧 [components/SpeciesTab.tsx](../frontend/src/pages/admin/components/SpeciesTab.tsx)
- [ ] 🔧 [components/TrainingRecordsTab.tsx](../frontend/src/pages/admin/components/TrainingRecordsTab.tsx)
- [ ] 🔧 [components/UserTable.tsx](../frontend/src/pages/admin/components/UserTable.tsx)
- [ ] 🔧 [components/ZoneTab.tsx](../frontend/src/pages/admin/components/ZoneTab.tsx)

---

## Animals — `frontend/src/pages/animals/`

- [ ] 🔧 [AnimalSourcesPage.tsx](../frontend/src/pages/animals/AnimalSourcesPage.tsx)
- [ ] 🔧 [components/AnimalListTable.tsx](../frontend/src/pages/animals/components/AnimalListTable.tsx)

---

## Amendments — `frontend/src/pages/amendments/`

- [ ] 🔧 [MyAmendmentsPage.tsx](../frontend/src/pages/amendments/MyAmendmentsPage.tsx)

---

## Dashboard — `frontend/src/pages/dashboard/`

- [ ] 🔧 [components/ErpWidgets.tsx](../frontend/src/pages/dashboard/components/ErpWidgets.tsx)

---

## Documents — `frontend/src/pages/documents/` (4/4 ✅)

- [x] ✅ [DocumentDetailPage.tsx](../frontend/src/pages/documents/DocumentDetailPage.tsx) — 8 欄，3 斷點 (600/750/900px)
- [x] ✅ [components/DocumentLineEditor.tsx](../frontend/src/pages/documents/components/DocumentLineEditor.tsx) — 10 欄條件式，2 斷點 (900px) + LineCard
- [x] ✅ [components/DocumentTable.tsx](../frontend/src/pages/documents/components/DocumentTable.tsx) — 10 欄，3 斷點 (600/750/900px)
- [x] ✅ [components/ProductSearchDialog.tsx](../frontend/src/pages/documents/components/ProductSearchDialog.tsx) — 多 row renderer，內嵌顯示 + 3 斷點 (400/450/500/600px)

---

## HR — `frontend/src/pages/hr/` (4/4 ✅)

- [x] ✅ [HrAnnualLeavePage.tsx](../frontend/src/pages/hr/HrAnnualLeavePage.tsx) — 2 個表格 (6/7 欄)，2 斷點 (500/600px) + Card
- [x] ✅ [calendar/ConflictsTab.tsx](../frontend/src/pages/hr/calendar/ConflictsTab.tsx) — 6 欄，3 斷點 (700/900/1000px) + Card
- [x] ✅ [components/AllRecordsTabContent.tsx](../frontend/src/pages/hr/components/AllRecordsTabContent.tsx) — 8 欄，4 斷點 (600/750/850/900/1000px) + Card
- [x] ✅ [components/AttendanceHistoryTab.tsx](../frontend/src/pages/hr/components/AttendanceHistoryTab.tsx) — 8 欄，4 斷點 (600/750/900/1050px) + Card

---

## Inventory — `frontend/src/pages/inventory/`

- [ ] 🔧 [StockLedgerPage.tsx](../frontend/src/pages/inventory/StockLedgerPage.tsx)

---

## Master — `frontend/src/pages/master/` (模板表格)

- [ ] 🔧 [BloodTestPanelsPage.tsx](../frontend/src/pages/master/BloodTestPanelsPage.tsx)
- [ ] 🔧 [BloodTestPresetsPage.tsx](../frontend/src/pages/master/BloodTestPresetsPage.tsx)
- [ ] 🔧 [WarehousesPage.tsx](../frontend/src/pages/master/WarehousesPage.tsx)
- [ ] 🔧 [components/BloodTestTemplateTable.tsx](../frontend/src/pages/master/components/BloodTestTemplateTable.tsx)
- [x] ✅ [components/ProductTable.tsx](../frontend/src/pages/master/components/ProductTable.tsx) — 整個拆掉改 @container：9 欄，4 斷點 (600/750/900/1050px) + Card，移除 ResizeObserver / COL_WIDTHS / MIN_TABLE_WIDTH
- [ ] 🔧 [partners/components/PartnerTable.tsx](../frontend/src/pages/master/partners/components/PartnerTable.tsx)

---

## My Projects — `frontend/src/pages/my-projects/` (2/2 ✅)

- [x] ✅ [MyProjectsPage.tsx](../frontend/src/pages/my-projects/MyProjectsPage.tsx) — 8 欄，4 斷點 (600/800/950/1100px) + Card
- [x] ✅ [MyProjectDetailPage.tsx](../frontend/src/pages/my-projects/MyProjectDetailPage.tsx) — 8 欄動物表，4 斷點 (600/800/900/1050px) + Card

---

## Protocols — `frontend/src/pages/protocols/`

- [ ] 🔧 [ProtocolsPage.tsx](../frontend/src/pages/protocols/ProtocolsPage.tsx)

---

## Reports — `frontend/src/pages/reports/` (報表類，資料密集)

- [ ] 🔧 [BloodTestAnalysisPage.tsx](../frontend/src/pages/reports/BloodTestAnalysisPage.tsx)
- [ ] 🔧 [BloodTestCostReportPage.tsx](../frontend/src/pages/reports/BloodTestCostReportPage.tsx)
- [ ] 🔧 [CostSummaryReportPage.tsx](../frontend/src/pages/reports/CostSummaryReportPage.tsx)
- [ ] 🔧 [PurchaseLinesReportPage.tsx](../frontend/src/pages/reports/PurchaseLinesReportPage.tsx)
- [ ] 🔧 [PurchaseSalesSummaryPage.tsx](../frontend/src/pages/reports/PurchaseSalesSummaryPage.tsx)
- [ ] 🔧 [SalesLinesReportPage.tsx](../frontend/src/pages/reports/SalesLinesReportPage.tsx)
- [ ] 🔧 [StockLedgerReportPage.tsx](../frontend/src/pages/reports/StockLedgerReportPage.tsx)
- [ ] 🔧 [StockOnHandReportPage.tsx](../frontend/src/pages/reports/StockOnHandReportPage.tsx)
- [ ] 🔧 [components/AnalysisChartTabs.tsx](../frontend/src/pages/reports/components/AnalysisChartTabs.tsx)
- [ ] 🔧 [components/ApAgingTab.tsx](../frontend/src/pages/reports/components/ApAgingTab.tsx)
- [ ] 🔧 [components/ArAgingTab.tsx](../frontend/src/pages/reports/components/ArAgingTab.tsx)
- [ ] 🔧 [components/JournalEntriesTab.tsx](../frontend/src/pages/reports/components/JournalEntriesTab.tsx)
- [ ] 🔧 [components/ProfitLossTab.tsx](../frontend/src/pages/reports/components/ProfitLossTab.tsx)
- [ ] 🔧 [components/TrialBalanceTab.tsx](../frontend/src/pages/reports/components/TrialBalanceTab.tsx)

---

## UI 基礎元件

- [x] ✅ [ui/data-table.tsx](../frontend/src/components/ui/data-table.tsx) — 新增 `ColumnDef.hideClassName`（Tailwind 字面量字串）+ `mobileCard` renderer + `cardBreakpoint` (500/600/700/800)

---

## 執行紀要

**已完成 30 個 ✅ 完整 RWD + 55 個 🔧 批次修復（共 85 個表格可正常顯示，⏳ 已歸零）**

### 2026-04-18 手機場景優先升級（+12 ✅）
手機使用者不會接觸 admin 區，故以下 4 個區塊（animal blood-test / documents / HR / my-projects）共 12 個檔案優先由 🔧 升級為 ✅：
- Animal blood-test: `BloodTestDetailDialog`, `BloodTestFormDialog`
- Documents: `DocumentDetailPage`, `DocumentLineEditor`, `DocumentTable`, `ProductSearchDialog`
- HR: `HrAnnualLeavePage`, `ConflictsTab`, `AllRecordsTabContent`, `AttendanceHistoryTab`
- My Projects: `MyProjectsPage`, `MyProjectDetailPage`

⏳ 清單已完成：
- `components/ui/data-table.tsx` ✅ 2026-04-18 升級（`hideClassName` + `mobileCard`）
- `pages/master/components/ProductTable.tsx` ✅ 2026-04-18 整個拆掉改 @container

### 附帶改動
- `formatFileSize()` util 升級：KB 整數、> 1000 KB 切換 MB（`frontend/src/lib/utils.ts`）
- PathologyTab 改用 `formatFileSize`

### 產出的 HTML 預覽工具（拖拉滑桿驗證 RWD）
位於 `docs/table-preview-*.html`，共 9 個：
- ObservationsTab, BloodTestTab, PainAssessmentTab, PathologyTab
- SurgeriesTab, VaccinationsTab, VetRecommendationsTab, WeightsTab
- AmendmentsTab

### 核心 bug 修正
移除所有 `[&>div]:overflow-x-hidden` — 這個 class 會把表格窄螢幕下的溢出**裁切掉**而非顯示捲軸，修正後所有表格在窄螢幕都能橫向捲動。

### 🔧 項目後續升級路徑
若某表格常被使用者在手機上檢視、UX 不佳，透過 `/system_table_chats <file>` 單獨討論後升級為 ✅。
