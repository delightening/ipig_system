import type { TFunction } from 'i18next'

import i18n from '@/lib/i18n'

// 稽核日誌（操作日誌 / 安全審計）的顯示用對照表。
//
// ⚠️ 這裡存的是 i18n 鍵（labelKey），不是翻譯後的字串：模組頂層呼叫 t() 會讓語言切換後
// 仍停在載入當下的語言。顯示時才用 get*() 取譯（呼叫端傳入 useTranslation() 的 t，
// 語言切換會連帶讓元件重渲染）。
// event_type / event_category / entity_type / alert_type / severity 這些 code 是後端原值，
// 用來比對與送 API，一律不翻譯；找不到對照時呼叫端 fallback 顯示原 code。

const has = (map: Record<string, unknown>, code: string) =>
  Object.prototype.hasOwnProperty.call(map, code)

// ── 事件類別 ──
const CATEGORY_LABEL_KEYS: Record<string, string> = {
  ERP: 'adminUsers.audit.category.erp',
  AUP: 'adminUsers.audit.category.aup',
  ANIMAL: 'adminUsers.audit.category.animal',
  SYSTEM: 'adminUsers.audit.category.system',
  // 補全：user_activity_logs 實際出現的 event_category（先前缺，稽核類別會露原文）
  ADMIN: 'adminUsers.audit.category.admin',
  AUDIT: 'adminUsers.audit.category.audit',
  EQUIPMENT: 'adminUsers.audit.category.equipment',
  HR: 'adminUsers.audit.category.hr',
  MESSAGING: 'adminUsers.audit.category.messaging',
  PLANNED_EXPERIMENT: 'adminUsers.audit.category.plannedExperiment',
  PROTOCOL_TEMPLATE: 'adminUsers.audit.category.protocolTemplate',
  SECURITY: 'adminUsers.audit.category.security',
}

// ── 事件類型（event_type）：labelKey + 徽章顏色 ──
const EVENT_TYPE_CONFIG: Record<string, { labelKey: string; color: string }> = {
  // 通用操作
  DOC_CREATE: { labelKey: 'adminUsers.audit.eventType.docCreate', color: 'bg-status-success-text' },
  DOC_UPDATE: { labelKey: 'adminUsers.audit.eventType.docUpdate', color: 'bg-status-info-text' },
  DOC_DELETE: { labelKey: 'adminUsers.audit.eventType.docDelete', color: 'bg-status-error-text' },
  DOC_SUBMIT: { labelKey: 'adminUsers.audit.eventType.docSubmit', color: 'bg-status-warning-text' },
  DOC_APPROVE: { labelKey: 'adminUsers.audit.eventType.docApprove', color: 'bg-status-purple-text' },
  DOC_CANCEL: { labelKey: 'adminUsers.audit.eventType.docCancel', color: 'bg-status-neutral-text' },
  PARTNER_CREATE: { labelKey: 'adminUsers.audit.eventType.partnerCreate', color: 'bg-status-success-text' },
  PARTNER_UPDATE: { labelKey: 'adminUsers.audit.eventType.partnerUpdate', color: 'bg-status-info-text' },
  PARTNER_DELETE: { labelKey: 'adminUsers.audit.eventType.partnerDelete', color: 'bg-status-error-text' },
  PRODUCT_CREATE: { labelKey: 'adminUsers.audit.eventType.productCreate', color: 'bg-status-success-text' },
  PRODUCT_UPDATE: { labelKey: 'adminUsers.audit.eventType.productUpdate', color: 'bg-status-info-text' },
  PRODUCT_DELETE: { labelKey: 'adminUsers.audit.eventType.productDelete', color: 'bg-status-error-text' },
  CATEGORY_CREATE: { labelKey: 'adminUsers.audit.eventType.categoryCreate', color: 'bg-status-success-text' },
  WAREHOUSE_CREATE: { labelKey: 'adminUsers.audit.eventType.warehouseCreate', color: 'bg-status-success-text' },
  WAREHOUSE_UPDATE: { labelKey: 'adminUsers.audit.eventType.warehouseUpdate', color: 'bg-status-info-text' },
  WAREHOUSE_DELETE: { labelKey: 'adminUsers.audit.eventType.warehouseDelete', color: 'bg-status-error-text' },
  ROLE_CREATE: { labelKey: 'adminUsers.audit.eventType.roleCreate', color: 'bg-status-success-text' },
  ROLE_UPDATE: { labelKey: 'adminUsers.audit.eventType.roleUpdate', color: 'bg-status-info-text' },
  ROLE_DELETE: { labelKey: 'adminUsers.audit.eventType.roleDelete', color: 'bg-status-error-text' },
  // 動物管理
  ANIMAL_CREATE: { labelKey: 'adminUsers.audit.eventType.animalCreate', color: 'bg-status-success-text' },
  ANIMAL_UPDATE: { labelKey: 'adminUsers.audit.eventType.animalUpdate', color: 'bg-status-info-text' },
  ANIMAL_DELETE: { labelKey: 'adminUsers.audit.eventType.animalDelete', color: 'bg-status-error-text' },
  ANIMAL_BATCH_ASSIGN: { labelKey: 'adminUsers.audit.eventType.animalBatchAssign', color: 'bg-status-purple-text' },
  OBSERVATION_CREATE: { labelKey: 'adminUsers.audit.eventType.observationCreate', color: 'bg-status-success-text' },
  OBSERVATION_UPDATE: { labelKey: 'adminUsers.audit.eventType.observationUpdate', color: 'bg-status-info-text' },
  OBSERVATION_DELETE: { labelKey: 'adminUsers.audit.eventType.observationDelete', color: 'bg-status-error-text' },
  SURGERY_CREATE: { labelKey: 'adminUsers.audit.eventType.surgeryCreate', color: 'bg-status-success-text' },
  SURGERY_UPDATE: { labelKey: 'adminUsers.audit.eventType.surgeryUpdate', color: 'bg-status-info-text' },
  SURGERY_DELETE: { labelKey: 'adminUsers.audit.eventType.surgeryDelete', color: 'bg-status-error-text' },
  WEIGHT_CREATE: { labelKey: 'adminUsers.audit.eventType.weightCreate', color: 'bg-status-success-text' },
  WEIGHT_UPDATE: { labelKey: 'adminUsers.audit.eventType.weightUpdate', color: 'bg-status-info-text' },
  WEIGHT_DELETE: { labelKey: 'adminUsers.audit.eventType.weightDelete', color: 'bg-status-error-text' },
  VACCINATION_CREATE: { labelKey: 'adminUsers.audit.eventType.vaccinationCreate', color: 'bg-status-success-text' },
  VACCINATION_UPDATE: { labelKey: 'adminUsers.audit.eventType.vaccinationUpdate', color: 'bg-status-info-text' },
  VACCINATION_DELETE: { labelKey: 'adminUsers.audit.eventType.vaccinationDelete', color: 'bg-status-error-text' },
  SACRIFICE_UPSERT: { labelKey: 'adminUsers.audit.eventType.sacrificeUpsert', color: 'bg-audit-sacrifice' },
  PATHOLOGY_UPSERT: { labelKey: 'adminUsers.audit.eventType.pathologyUpsert', color: 'bg-status-purple-text' },
  VET_RECOMMENDATION_ADD: { labelKey: 'adminUsers.audit.eventType.vetRecommendationAdd', color: 'bg-audit-medical' },
  MEDICAL_EXPORT: { labelKey: 'adminUsers.audit.eventType.medicalExport', color: 'bg-audit-medical' },
  BLOOD_TEST_CREATE: { labelKey: 'adminUsers.audit.eventType.bloodTestCreate', color: 'bg-status-success-text' },
  BLOOD_TEST_UPDATE: { labelKey: 'adminUsers.audit.eventType.bloodTestUpdate', color: 'bg-status-info-text' },
  BLOOD_TEST_DELETE: { labelKey: 'adminUsers.audit.eventType.bloodTestDelete', color: 'bg-status-error-text' },
  TEMPLATE_CREATE: { labelKey: 'adminUsers.audit.eventType.templateCreate', color: 'bg-status-success-text' },
  TEMPLATE_UPDATE: { labelKey: 'adminUsers.audit.eventType.templateUpdate', color: 'bg-status-info-text' },
  TEMPLATE_DELETE: { labelKey: 'adminUsers.audit.eventType.templateDelete', color: 'bg-status-error-text' },
  PANEL_CREATE: { labelKey: 'adminUsers.audit.eventType.panelCreate', color: 'bg-status-success-text' },
  PANEL_UPDATE: { labelKey: 'adminUsers.audit.eventType.panelUpdate', color: 'bg-status-info-text' },
  PANEL_DELETE: { labelKey: 'adminUsers.audit.eventType.panelDelete', color: 'bg-status-error-text' },
  // 計畫書
  PROTOCOL_CREATE: { labelKey: 'adminUsers.audit.eventType.protocolCreate', color: 'bg-audit-protocol' },
  PROTOCOL_UPDATE: { labelKey: 'adminUsers.audit.eventType.protocolUpdate', color: 'bg-audit-protocol' },
  PROTOCOL_SUBMIT: { labelKey: 'adminUsers.audit.eventType.protocolSubmit', color: 'bg-audit-protocol' },
  PROTOCOL_APPROVE: { labelKey: 'adminUsers.audit.eventType.protocolApprove', color: 'bg-audit-protocol' },
  PROTOCOL_REJECT: { labelKey: 'adminUsers.audit.eventType.protocolReject', color: 'bg-status-error-text' },
  PROTOCOL_STATUS_CHANGE: { labelKey: 'adminUsers.audit.eventType.protocolStatusChange', color: 'bg-audit-protocol' },
  PROTOCOL_REVIEWER_ASSIGN: { labelKey: 'adminUsers.audit.eventType.protocolReviewerAssign', color: 'bg-audit-protocol' },
  PROTOCOL_IMPORT_APPROVED: { labelKey: 'adminUsers.audit.eventType.protocolImportApproved', color: 'bg-audit-protocol' },
  PROTOCOL_IMPORT_FINALIZED: { labelKey: 'adminUsers.audit.eventType.protocolImportFinalized', color: 'bg-audit-protocol' },
  PROTOCOL_IMPORT_REVIEWS_RECORDED: { labelKey: 'adminUsers.audit.eventType.protocolImportReviewsRecorded', color: 'bg-audit-protocol' },
  PROTOCOL_DELETED: { labelKey: 'adminUsers.audit.eventType.protocolDeleted', color: 'bg-status-error-text' },
  PROTOCOL_SOFT_DELETED: { labelKey: 'adminUsers.audit.eventType.protocolSoftDeleted', color: 'bg-status-neutral-text' },
  PROTOCOL_PI_PROVISIONED: { labelKey: 'adminUsers.audit.eventType.protocolPiProvisioned', color: 'bg-audit-protocol' },
  PI_INVITE_SEND: { labelKey: 'adminUsers.audit.eventType.piInviteSend', color: 'bg-status-info-text' },
  // 計畫修正案
  AMENDMENT_CLASSIFY_MINOR: { labelKey: 'adminUsers.audit.eventType.amendmentClassifyMinor', color: 'bg-status-info-text' },
  AMENDMENT_CLASSIFY_MAJOR: { labelKey: 'adminUsers.audit.eventType.amendmentClassifyMajor', color: 'bg-status-warning-text' },
  AMENDMENT_REVISION_REQUIRED: { labelKey: 'adminUsers.audit.eventType.amendmentRevisionRequired', color: 'bg-status-warning-text' },
  AMENDMENT_EFFECTIVE: { labelKey: 'adminUsers.audit.eventType.amendmentEffective', color: 'bg-status-purple-text' },
  AMENDMENT_IMPORT_BACKFILLED: { labelKey: 'adminUsers.audit.eventType.amendmentImportBackfilled', color: 'bg-status-info-text' },
  AMENDMENT_IMPORT_FINALIZED: { labelKey: 'adminUsers.audit.eventType.amendmentImportFinalized', color: 'bg-status-info-text' },
  AMENDMENT_IMPORT_REVIEWS_RECORDED: { labelKey: 'adminUsers.audit.eventType.amendmentImportReviewsRecorded', color: 'bg-status-info-text' },
  // 登入 / 安全
  LOGIN_SUCCESS: { labelKey: 'adminUsers.audit.eventType.loginSuccess', color: 'bg-status-success-text' },
  LOGIN_FAILED: { labelKey: 'adminUsers.audit.eventType.loginFailed', color: 'bg-status-error-text' },
  LOGOUT: { labelKey: 'adminUsers.audit.eventType.logout', color: 'bg-status-neutral-text' },
  FORCE_LOGOUT: { labelKey: 'adminUsers.audit.eventType.forceLogout', color: 'bg-status-error-text' },
  IMPERSONATE_START: { labelKey: 'adminUsers.audit.eventType.impersonateStart', color: 'bg-status-warning-text' },
  PASSWORD_SELF_CHANGE: { labelKey: 'adminUsers.audit.eventType.passwordSelfChange', color: 'bg-status-info-text' },
  PASSWORD_ADMIN_RESET: { labelKey: 'adminUsers.audit.eventType.passwordAdminReset', color: 'bg-status-warning-text' },
  PASSWORD_TOKEN_RESET: { labelKey: 'adminUsers.audit.eventType.passwordTokenReset', color: 'bg-status-info-text' },
  TWO_FACTOR_ENABLED: { labelKey: 'adminUsers.audit.eventType.twoFactorEnabled', color: 'bg-status-success-text' },
  TWO_FACTOR_DISABLED: { labelKey: 'adminUsers.audit.eventType.twoFactorDisabled', color: 'bg-status-warning-text' },
  IP_BLOCKLIST_ADD: { labelKey: 'adminUsers.audit.eventType.ipBlocklistAdd', color: 'bg-status-error-text' },
  IP_BLOCKLIST_UNBLOCK: { labelKey: 'adminUsers.audit.eventType.ipBlocklistUnblock', color: 'bg-status-success-text' },
  SIGNATURE_CREATE: { labelKey: 'adminUsers.audit.eventType.signatureCreate', color: 'bg-status-purple-text' },
  // 使用者 / 角色
  USER_CREATE: { labelKey: 'adminUsers.audit.eventType.userCreate', color: 'bg-status-success-text' },
  USER_UPDATE: { labelKey: 'adminUsers.audit.eventType.userUpdate', color: 'bg-status-info-text' },
  USER_STATUS_CHANGE: { labelKey: 'adminUsers.audit.eventType.userStatusChange', color: 'bg-status-info-text' },
  USER_ROLE_CHANGE: { labelKey: 'adminUsers.audit.eventType.userRoleChange', color: 'bg-status-info-text' },
  USER_DEPARTMENT_CHANGE: { labelKey: 'adminUsers.audit.eventType.userDepartmentChange', color: 'bg-status-info-text' },
  USER_DEACTIVATE_SELF: { labelKey: 'adminUsers.audit.eventType.userDeactivateSelf', color: 'bg-status-warning-text' },
  USER_DELETE: { labelKey: 'adminUsers.audit.eventType.userDelete', color: 'bg-status-error-text' },
  ROLE_PERMISSION_CHANGE: { labelKey: 'adminUsers.audit.eventType.rolePermissionChange', color: 'bg-status-info-text' },
  ROLE_DEACTIVATE: { labelKey: 'adminUsers.audit.eventType.roleDeactivate', color: 'bg-status-warning-text' },
  AI_API_KEY_CREATE: { labelKey: 'adminUsers.audit.eventType.aiApiKeyCreate', color: 'bg-status-success-text' },
  AI_API_KEY_DELETE: { labelKey: 'adminUsers.audit.eventType.aiApiKeyDelete', color: 'bg-status-error-text' },
  // HR — 請假
  LEAVE_CREATE: { labelKey: 'adminUsers.audit.eventType.leaveCreate', color: 'bg-status-success-text' },
  LEAVE_UPDATE: { labelKey: 'adminUsers.audit.eventType.leaveUpdate', color: 'bg-status-info-text' },
  LEAVE_DELETE: { labelKey: 'adminUsers.audit.eventType.leaveDelete', color: 'bg-status-error-text' },
  LEAVE_SUBMIT: { labelKey: 'adminUsers.audit.eventType.leaveSubmit', color: 'bg-status-warning-text' },
  LEAVE_PROXY_CONFIRM: { labelKey: 'adminUsers.audit.eventType.leaveProxyConfirm', color: 'bg-status-purple-text' },
  LEAVE_PROXY_REJECT: { labelKey: 'adminUsers.audit.eventType.leaveProxyReject', color: 'bg-status-error-text' },
  LEAVE_APPROVE_INTERIM: { labelKey: 'adminUsers.audit.eventType.leaveApproveInterim', color: 'bg-status-purple-text' },
  LEAVE_APPROVE_FINAL: { labelKey: 'adminUsers.audit.eventType.leaveApproveFinal', color: 'bg-status-purple-text' },
  LEAVE_APPROVE_INTERIM_OVERRIDE: { labelKey: 'adminUsers.audit.eventType.leaveApproveInterimOverride', color: 'bg-status-purple-text' },
  LEAVE_APPROVE_FINAL_OVERRIDE: { labelKey: 'adminUsers.audit.eventType.leaveApproveFinalOverride', color: 'bg-status-purple-text' },
  LEAVE_REJECT: { labelKey: 'adminUsers.audit.eventType.leaveReject', color: 'bg-status-error-text' },
  LEAVE_CANCEL: { labelKey: 'adminUsers.audit.eventType.leaveCancel', color: 'bg-status-neutral-text' },
  LEAVE_CANCEL_RETROACTIVE: { labelKey: 'adminUsers.audit.eventType.leaveCancelRetroactive', color: 'bg-status-neutral-text' },
  // HR — 加班
  OVERTIME_CREATE: { labelKey: 'adminUsers.audit.eventType.overtimeCreate', color: 'bg-status-success-text' },
  OVERTIME_UPDATE: { labelKey: 'adminUsers.audit.eventType.overtimeUpdate', color: 'bg-status-info-text' },
  OVERTIME_DELETE: { labelKey: 'adminUsers.audit.eventType.overtimeDelete', color: 'bg-status-error-text' },
  OVERTIME_SUBMIT: { labelKey: 'adminUsers.audit.eventType.overtimeSubmit', color: 'bg-status-warning-text' },
  OVERTIME_APPROVE_INTERIM: { labelKey: 'adminUsers.audit.eventType.overtimeApproveInterim', color: 'bg-status-purple-text' },
  OVERTIME_APPROVE_FINAL: { labelKey: 'adminUsers.audit.eventType.overtimeApproveFinal', color: 'bg-status-purple-text' },
  OVERTIME_REJECT: { labelKey: 'adminUsers.audit.eventType.overtimeReject', color: 'bg-status-error-text' },
  OVERTIME_VOID: { labelKey: 'adminUsers.audit.eventType.overtimeVoid', color: 'bg-status-error-text' },
  COMP_TIME_REVOKE: { labelKey: 'adminUsers.audit.eventType.compTimeRevoke', color: 'bg-status-error-text' },
  // HR — 出勤 / 特休
  ATTENDANCE_CLOCK_IN: { labelKey: 'adminUsers.audit.eventType.attendanceClockIn', color: 'bg-status-success-text' },
  ATTENDANCE_CLOCK_OUT: { labelKey: 'adminUsers.audit.eventType.attendanceClockOut', color: 'bg-status-info-text' },
  ATTENDANCE_CLOCK_IN_DENIED: { labelKey: 'adminUsers.audit.eventType.attendanceClockInDenied', color: 'bg-status-error-text' },
  ATTENDANCE_CLOCK_OUT_DENIED: { labelKey: 'adminUsers.audit.eventType.attendanceClockOutDenied', color: 'bg-status-error-text' },
  ATTENDANCE_CORRECT: { labelKey: 'adminUsers.audit.eventType.attendanceCorrect', color: 'bg-status-warning-text' },
  ATTENDANCE_BACKFILL: { labelKey: 'adminUsers.audit.eventType.attendanceBackfill', color: 'bg-status-warning-text' },
  ANNUAL_LEAVE_CREATE: { labelKey: 'adminUsers.audit.eventType.annualLeaveCreate', color: 'bg-status-success-text' },
  ANNUAL_LEAVE_ADJUST: { labelKey: 'adminUsers.audit.eventType.annualLeaveAdjust', color: 'bg-status-info-text' },
  ANNUAL_LEAVE_BATCH_AUTO_CALC: { labelKey: 'adminUsers.audit.eventType.annualLeaveBatchAutoCalc', color: 'bg-status-neutral-text' },
  ANNUAL_LEAVE_EXPIRY_RECOMPUTE: { labelKey: 'adminUsers.audit.eventType.annualLeaveExpiryRecompute', color: 'bg-status-neutral-text' },
  // 動物 — 延伸事件
  ANIMAL_ASSIGN: { labelKey: 'adminUsers.audit.eventType.animalAssign', color: 'bg-status-purple-text' },
  ANIMAL_REMARK_UPDATE: { labelKey: 'adminUsers.audit.eventType.animalRemarkUpdate', color: 'bg-status-info-text' },
  ANIMAL_IMPORT: { labelKey: 'adminUsers.audit.eventType.animalImport', color: 'bg-status-success-text' },
  IACUC_CHANGE: { labelKey: 'adminUsers.audit.eventType.iacucChange', color: 'bg-status-info-text' },
  WEIGHT_SOFT_DELETE: { labelKey: 'adminUsers.audit.eventType.weightDelete', color: 'bg-status-error-text' },
  WEIGHT_IMPORT: { labelKey: 'adminUsers.audit.eventType.weightImport', color: 'bg-status-success-text' },
  SUDDEN_DEATH: { labelKey: 'adminUsers.audit.eventType.suddenDeath', color: 'bg-audit-sacrifice' },
  BLOOD_TEST_ITEM_CORRECT: { labelKey: 'adminUsers.audit.eventType.bloodTestItemCorrect', color: 'bg-status-warning-text' },
  PRESET_CREATE: { labelKey: 'adminUsers.audit.eventType.presetCreate', color: 'bg-status-success-text' },
  PRESET_UPDATE: { labelKey: 'adminUsers.audit.eventType.presetUpdate', color: 'bg-status-info-text' },
  PRESET_DELETE: { labelKey: 'adminUsers.audit.eventType.presetDelete', color: 'bg-status-error-text' },
  CARE_RECORD_CREATE: { labelKey: 'adminUsers.audit.eventType.careRecordCreate', color: 'bg-status-success-text' },
  CARE_RECORD_UPDATE: { labelKey: 'adminUsers.audit.eventType.careRecordUpdate', color: 'bg-status-info-text' },
  CARE_RECORD_DELETE: { labelKey: 'adminUsers.audit.eventType.careRecordDelete', color: 'bg-status-error-text' },
  VET_ADVICE_RECORD_CREATE: { labelKey: 'adminUsers.audit.eventType.vetAdviceRecordCreate', color: 'bg-audit-medical' },
  VET_ADVICE_RECORD_UPDATE: { labelKey: 'adminUsers.audit.eventType.vetAdviceRecordUpdate', color: 'bg-audit-medical' },
  VET_ADVICE_RECORD_DELETE: { labelKey: 'adminUsers.audit.eventType.vetAdviceRecordDelete', color: 'bg-status-error-text' },
  VET_PATROL_REPORT_CREATED: { labelKey: 'adminUsers.audit.eventType.vetPatrolReportCreated', color: 'bg-audit-medical' },
  VET_PATROL_REPORT_UPDATED: { labelKey: 'adminUsers.audit.eventType.vetPatrolReportUpdated', color: 'bg-audit-medical' },
  VET_PATROL_REPORT_SUBMITTED_FOR_FOLLOWUP: { labelKey: 'adminUsers.audit.eventType.vetPatrolReportSubmittedForFollowup', color: 'bg-status-warning-text' },
  VET_PATROL_REPORT_ACKNOWLEDGED: { labelKey: 'adminUsers.audit.eventType.vetPatrolReportAcknowledged', color: 'bg-status-purple-text' },
  VET_PATROL_REPORT_COMPLETED: { labelKey: 'adminUsers.audit.eventType.vetPatrolReportCompleted', color: 'bg-status-success-text' },
  VET_PATROL_REPORT_DISCARDED: { labelKey: 'adminUsers.audit.eventType.vetPatrolReportDiscarded', color: 'bg-status-neutral-text' },
  VET_PATROL_REPORT_DELETED: { labelKey: 'adminUsers.audit.eventType.vetPatrolReportDeleted', color: 'bg-status-error-text' },
  VET_PATROL_ENTRY_PHOTO_DELETED: { labelKey: 'adminUsers.audit.eventType.vetPatrolEntryPhotoDeleted', color: 'bg-status-error-text' },
  BYPRODUCT_SAMPLE_CREATE: { labelKey: 'adminUsers.audit.eventType.byproductSampleCreate', color: 'bg-status-success-text' },
  BYPRODUCT_SAMPLE_UPDATE: { labelKey: 'adminUsers.audit.eventType.byproductSampleUpdate', color: 'bg-status-info-text' },
  BYPRODUCT_SAMPLE_DELETE: { labelKey: 'adminUsers.audit.eventType.byproductSampleDelete', color: 'bg-status-error-text' },
  ANIMAL_SOURCE_CREATE: { labelKey: 'adminUsers.audit.eventType.animalSourceCreate', color: 'bg-status-success-text' },
  ANIMAL_SOURCE_UPDATE: { labelKey: 'adminUsers.audit.eventType.animalSourceUpdate', color: 'bg-status-info-text' },
  ANIMAL_SOURCE_DEACTIVATE: { labelKey: 'adminUsers.audit.eventType.animalSourceDeactivate', color: 'bg-status-neutral-text' },
  // 動物 — 轉移流程
  TRANSFER_INITIATE: { labelKey: 'adminUsers.audit.eventType.transferInitiate', color: 'bg-status-info-text' },
  TRANSFER_VET_EVALUATE: { labelKey: 'adminUsers.audit.eventType.transferVetEvaluate', color: 'bg-audit-medical' },
  TRANSFER_ASSIGN_PLAN: { labelKey: 'adminUsers.audit.eventType.transferAssignPlan', color: 'bg-status-info-text' },
  TRANSFER_APPROVE: { labelKey: 'adminUsers.audit.eventType.transferApprove', color: 'bg-status-purple-text' },
  TRANSFER_COMPLETE: { labelKey: 'adminUsers.audit.eventType.transferComplete', color: 'bg-status-success-text' },
  TRANSFER_REJECT: { labelKey: 'adminUsers.audit.eventType.transferReject', color: 'bg-status-error-text' },
  // 醫療匯出
  EXPORT_MEDICAL: { labelKey: 'adminUsers.audit.eventType.medicalExport', color: 'bg-audit-medical' },
  EXPORT_SURGERY: { labelKey: 'adminUsers.audit.eventType.exportSurgery', color: 'bg-audit-medical' },
  EXPORT_BLOOD_TEST: { labelKey: 'adminUsers.audit.eventType.exportBloodTest', color: 'bg-audit-medical' },
  ANIMAL_MEDICAL_WEEKLY_REPORT_READ: { labelKey: 'adminUsers.audit.eventType.animalMedicalWeeklyReportRead', color: 'bg-audit-medical' },
  // ERP — 延伸事件
  DOC_WM_APPROVE: { labelKey: 'adminUsers.audit.eventType.docWmApprove', color: 'bg-status-purple-text' },
  DOC_ADMIN_APPROVE: { labelKey: 'adminUsers.audit.eventType.docAdminApprove', color: 'bg-status-purple-text' },
  DOC_ADMIN_REJECT: { labelKey: 'adminUsers.audit.eventType.docAdminReject', color: 'bg-status-error-text' },
  PRODUCT_CREATE_WITH_SKU: { labelKey: 'adminUsers.audit.eventType.productCreateWithSku', color: 'bg-status-success-text' },
  PRODUCT_STATUS_CHANGE: { labelKey: 'adminUsers.audit.eventType.productStatusChange', color: 'bg-status-info-text' },
  PRODUCT_HARD_DELETE: { labelKey: 'adminUsers.audit.eventType.productHardDelete', color: 'bg-status-error-text' },
  PRODUCT_CATEGORY_CREATE: { labelKey: 'adminUsers.audit.eventType.productCategoryCreate', color: 'bg-status-success-text' },
  PRODUCT_IMPORT: { labelKey: 'adminUsers.audit.eventType.productImport', color: 'bg-status-success-text' },
  PARTNER_IMPORT: { labelKey: 'adminUsers.audit.eventType.partnerImport', color: 'bg-status-success-text' },
  WAREHOUSE_IMPORT: { labelKey: 'adminUsers.audit.eventType.warehouseImport', color: 'bg-status-success-text' },
  SKU_CATEGORY_UPDATE: { labelKey: 'adminUsers.audit.eventType.skuCategoryUpdate', color: 'bg-status-info-text' },
  SKU_CATEGORY_DELETE: { labelKey: 'adminUsers.audit.eventType.skuCategoryDelete', color: 'bg-status-error-text' },
  SKU_SUBCATEGORY_CREATE: { labelKey: 'adminUsers.audit.eventType.skuSubcategoryCreate', color: 'bg-status-success-text' },
  SKU_SUBCATEGORY_UPDATE: { labelKey: 'adminUsers.audit.eventType.skuSubcategoryUpdate', color: 'bg-status-info-text' },
  SKU_SUBCATEGORY_DELETE: { labelKey: 'adminUsers.audit.eventType.skuSubcategoryDelete', color: 'bg-status-error-text' },
  STORAGE_INVENTORY_UPDATE: { labelKey: 'adminUsers.audit.eventType.storageInventoryUpdate', color: 'bg-status-info-text' },
  AP_PAYMENT_CREATED: { labelKey: 'adminUsers.audit.eventType.apPaymentCreated', color: 'bg-status-success-text' },
  AR_RECEIPT_CREATED: { labelKey: 'adminUsers.audit.eventType.arReceiptCreated', color: 'bg-status-success-text' },
  DATA_EXPORT: { labelKey: 'adminUsers.audit.eventType.dataExport', color: 'bg-audit-data' },
  DATA_IMPORT: { labelKey: 'adminUsers.audit.eventType.dataImport', color: 'bg-audit-data' },
  // 設施 / 設備
  MAINTENANCE_CREATE: { labelKey: 'adminUsers.audit.eventType.maintenanceCreate', color: 'bg-status-success-text' },
  MAINTENANCE_UPDATE: { labelKey: 'adminUsers.audit.eventType.maintenanceUpdate', color: 'bg-status-info-text' },
  MAINTENANCE_DELETE: { labelKey: 'adminUsers.audit.eventType.maintenanceDelete', color: 'bg-status-error-text' },
  MAINTENANCE_REVIEWER_SIGNATURE: { labelKey: 'adminUsers.audit.eventType.maintenanceReviewerSignature', color: 'bg-status-purple-text' },
  DISPOSAL_APPLICANT_SIGNATURE: { labelKey: 'adminUsers.audit.eventType.disposalApplicantSignature', color: 'bg-status-warning-text' },
  DISPOSAL_APPROVER_SIGNATURE: { labelKey: 'adminUsers.audit.eventType.disposalApproverSignature', color: 'bg-status-purple-text' },
  DISPOSAL_RESTORE: { labelKey: 'adminUsers.audit.eventType.disposalRestore', color: 'bg-status-neutral-text' },
  EQUIPMENT_AUTO_RESTORE: { labelKey: 'adminUsers.audit.eventType.equipmentAutoRestore', color: 'bg-status-neutral-text' },
  FACILITY_SPECIES_CREATE: { labelKey: 'adminUsers.audit.eventType.facilitySpeciesCreate', color: 'bg-status-success-text' },
  FACILITY_SPECIES_UPDATE: { labelKey: 'adminUsers.audit.eventType.facilitySpeciesUpdate', color: 'bg-status-info-text' },
  FACILITY_SPECIES_DELETE: { labelKey: 'adminUsers.audit.eventType.facilitySpeciesDelete', color: 'bg-status-error-text' },
  FACILITY_FACILITY_CREATE: { labelKey: 'adminUsers.audit.eventType.facilityFacilityCreate', color: 'bg-status-success-text' },
  FACILITY_FACILITY_UPDATE: { labelKey: 'adminUsers.audit.eventType.facilityFacilityUpdate', color: 'bg-status-info-text' },
  FACILITY_FACILITY_DELETE: { labelKey: 'adminUsers.audit.eventType.facilityFacilityDelete', color: 'bg-status-error-text' },
  FACILITY_BUILDING_CREATE: { labelKey: 'adminUsers.audit.eventType.facilityBuildingCreate', color: 'bg-status-success-text' },
  FACILITY_BUILDING_UPDATE: { labelKey: 'adminUsers.audit.eventType.facilityBuildingUpdate', color: 'bg-status-info-text' },
  FACILITY_BUILDING_DELETE: { labelKey: 'adminUsers.audit.eventType.facilityBuildingDelete', color: 'bg-status-error-text' },
  FACILITY_ZONE_CREATE: { labelKey: 'adminUsers.audit.eventType.facilityZoneCreate', color: 'bg-status-success-text' },
  FACILITY_ZONE_UPDATE: { labelKey: 'adminUsers.audit.eventType.facilityZoneUpdate', color: 'bg-status-info-text' },
  FACILITY_ZONE_DELETE: { labelKey: 'adminUsers.audit.eventType.facilityZoneDelete', color: 'bg-status-error-text' },
  FACILITY_PEN_CREATE: { labelKey: 'adminUsers.audit.eventType.facilityPenCreate', color: 'bg-status-success-text' },
  FACILITY_PEN_BATCH_CREATE: { labelKey: 'adminUsers.audit.eventType.facilityPenBatchCreate', color: 'bg-status-success-text' },
  FACILITY_PEN_UPDATE: { labelKey: 'adminUsers.audit.eventType.facilityPenUpdate', color: 'bg-status-info-text' },
  FACILITY_PEN_DELETE: { labelKey: 'adminUsers.audit.eventType.facilityPenDelete', color: 'bg-status-error-text' },
  FACILITY_DEPARTMENT_CREATE: { labelKey: 'adminUsers.audit.eventType.facilityDepartmentCreate', color: 'bg-status-success-text' },
  FACILITY_DEPARTMENT_UPDATE: { labelKey: 'adminUsers.audit.eventType.facilityDepartmentUpdate', color: 'bg-status-info-text' },
  FACILITY_DEPARTMENT_DELETE: { labelKey: 'adminUsers.audit.eventType.facilityDepartmentDelete', color: 'bg-status-error-text' },
  // 站內信
  MESSAGE_THREAD_CREATED: { labelKey: 'adminUsers.audit.eventType.messageThreadCreated', color: 'bg-status-success-text' },
  MESSAGE_SENT: { labelKey: 'adminUsers.audit.eventType.messageSent', color: 'bg-status-info-text' },
  MESSAGE_DELETED: { labelKey: 'adminUsers.audit.eventType.messageDeleted', color: 'bg-status-error-text' },
  // 通用動詞（GLP 各模組共用）
  CREATE: { labelKey: 'adminUsers.audit.eventType.create', color: 'bg-status-success-text' },
  UPDATE: { labelKey: 'adminUsers.audit.eventType.update', color: 'bg-status-info-text' },
  DELETE: { labelKey: 'adminUsers.audit.eventType.delete', color: 'bg-status-error-text' },
  APPROVE: { labelKey: 'adminUsers.audit.eventType.approve', color: 'bg-status-purple-text' },
  ACKNOWLEDGE: { labelKey: 'adminUsers.audit.eventType.acknowledge', color: 'bg-status-info-text' },
  // 補全：user_activity_logs 實際出現但先前缺映射（操作日誌會露原始 event_type）
  ACCOUNT_LOCKOUT: { labelKey: 'adminUsers.audit.eventType.accountLockout', color: 'bg-status-error-text' },
  PERMISSION_DENIED: { labelKey: 'adminUsers.audit.eventType.permissionDenied', color: 'bg-status-error-text' },
  RATE_LIMIT_API: { labelKey: 'adminUsers.audit.eventType.rateLimitApi', color: 'bg-status-warning-text' },
  USER_AUTO_SUSPENDED: { labelKey: 'adminUsers.audit.eventType.userAutoSuspended', color: 'bg-status-warning-text' },
  DOC_HARD_DELETE: { labelKey: 'adminUsers.audit.eventType.docHardDelete', color: 'bg-status-error-text' },
  MAINTENANCE_REVIEW: { labelKey: 'adminUsers.audit.eventType.maintenanceReview', color: 'bg-status-info-text' },
  MAINTENANCE_REVIEW_APPROVE: { labelKey: 'adminUsers.audit.eventType.maintenanceReviewApprove', color: 'bg-status-purple-text' },
  ANIMAL_RESERVED: { labelKey: 'adminUsers.audit.eventType.animalReserved', color: 'bg-status-purple-text' },
  PLANNED_EXPERIMENT_CREATED: { labelKey: 'adminUsers.audit.eventType.plannedExperimentCreated', color: 'bg-status-success-text' },
  APPLICATION_NOTICE_CREATED: { labelKey: 'adminUsers.audit.eventType.applicationNoticeCreated', color: 'bg-status-success-text' },
  APPLICATION_NOTICE_CONTENT_UPDATED: { labelKey: 'adminUsers.audit.eventType.applicationNoticeContentUpdated', color: 'bg-status-info-text' },
  APPLICATION_NOTICE_ACTIVATED: { labelKey: 'adminUsers.audit.eventType.applicationNoticeActivated', color: 'bg-status-purple-text' },
  PROTOCOL_COEDITOR_ASSIGN: { labelKey: 'adminUsers.audit.eventType.protocolCoeditorAssign', color: 'bg-audit-protocol' },
  PROTOCOL_VET_ASSIGN: { labelKey: 'adminUsers.audit.eventType.protocolVetAssign', color: 'bg-audit-protocol' },
  PROTOCOL_IMPORT_DELETED: { labelKey: 'adminUsers.audit.eventType.protocolImportDeleted', color: 'bg-status-error-text' },
  PROTOCOL_TEMPLATE_VERSION_CREATED: { labelKey: 'adminUsers.audit.eventType.protocolTemplateVersionCreated', color: 'bg-status-success-text' },
  PROTOCOL_TEMPLATE_VERSION_DELETED: { labelKey: 'adminUsers.audit.eventType.protocolTemplateVersionDeleted', color: 'bg-status-error-text' },
  PROTOCOL_TEMPLATE_VERSION_SET_CURRENT: { labelKey: 'adminUsers.audit.eventType.protocolTemplateVersionSetCurrent', color: 'bg-status-purple-text' },
  VET_PATROL_REPORT_SUBMITTED: { labelKey: 'adminUsers.audit.eventType.vetPatrolReportSubmitted', color: 'bg-status-warning-text' },
}

// ── 資料類型（entity_type）──
//
// 涵蓋來源：backend 全 codebase `AuditEntity::new("...")` 字面值掃描（Gemini PR #412
// review 補完後）。新增 entity_type 時，後端 author 應同步 PR 此檔（並補 zh-TW / en 語言包
// 的 adminUsers.audit.entityType.*），未補映射時 fallback 顯示原值（UI 不會白屏，但會露出 raw 表名）。
const ENTITY_TYPE_LABEL_KEYS: Record<string, string> = {
  // 既有 ERP / Animal / Protocol entity 對應
  document: 'adminUsers.audit.entityType.document',
  partner: 'adminUsers.audit.entityType.partner',
  partner_import_job: 'adminUsers.audit.entityType.partnerImportJob',
  product: 'adminUsers.audit.entityType.product',
  product_category: 'adminUsers.audit.entityType.productCategory',
  warehouse: 'adminUsers.audit.entityType.warehouse',
  role: 'adminUsers.audit.entityType.role',
  user: 'adminUsers.audit.entityType.user',
  session: 'adminUsers.audit.entityType.session',
  database: 'adminUsers.audit.entityType.database',
  ai_api_key: 'adminUsers.audit.entityType.aiApiKey',
  system_setting: 'adminUsers.audit.entityType.systemSetting',
  // 動物相關
  animal: 'adminUsers.audit.entityType.animal',
  animal_observation: 'adminUsers.audit.entityType.animalObservation',
  animal_surgery: 'adminUsers.audit.entityType.animalSurgery',
  animal_surgeries: 'adminUsers.audit.entityType.animalSurgery',
  animal_weight: 'adminUsers.audit.entityType.animalWeight',
  animal_vaccination: 'adminUsers.audit.entityType.animalVaccination',
  animal_sacrifice: 'adminUsers.audit.entityType.animalSacrifice',
  animal_sudden_deaths: 'adminUsers.audit.entityType.animalSuddenDeaths',
  animal_pathology: 'adminUsers.audit.entityType.animalPathology',
  animal_blood_test: 'adminUsers.audit.entityType.animalBloodTest',
  animal_blood_test_item: 'adminUsers.audit.entityType.animalBloodTestItem',
  animal_source: 'adminUsers.audit.entityType.animalSource',
  animal_transfers: 'adminUsers.audit.entityType.animalTransfers',
  animal_import_batch: 'adminUsers.audit.entityType.animalImportBatch',
  blood_test_template: 'adminUsers.audit.entityType.bloodTestTemplate',
  blood_test_panel: 'adminUsers.audit.entityType.bloodTestPanel',
  blood_test_preset: 'adminUsers.audit.entityType.bloodTestPreset',
  care_medication_record: 'adminUsers.audit.entityType.careMedicationRecord',
  vet_recommendation: 'adminUsers.audit.entityType.vetRecommendation',
  vet_patrol_reports: 'adminUsers.audit.entityType.vetPatrolReports',
  euthanasia: 'adminUsers.audit.entityType.euthanasia',
  // 計畫書 / 修正案
  protocol: 'adminUsers.audit.entityType.protocol',
  amendment: 'adminUsers.audit.entityType.amendment',
  // 設施 / 設備
  facility: 'adminUsers.audit.entityType.facility',
  equipment: 'adminUsers.audit.entityType.equipment',
  maintenance_record: 'adminUsers.audit.entityType.maintenanceRecord',
  // HR
  overtime_record: 'adminUsers.audit.entityType.overtimeRecord',
  leave_request: 'adminUsers.audit.entityType.leaveRequest',
  attendance_record: 'adminUsers.audit.entityType.attendanceRecord',
  // 站內信
  message_threads: 'adminUsers.audit.entityType.messageThreads',
  messages: 'adminUsers.audit.entityType.messages',
  // QA / GLP（Gemini PR #412 review：補完整 table 名稱對應，避免縮寫漏失）
  qa_inspections: 'adminUsers.audit.entityType.qaInspections',
  qa_inspection: 'adminUsers.audit.entityType.qaInspections',
  qa_non_conformances: 'adminUsers.audit.entityType.qaNonConformances',
  qa_nc: 'adminUsers.audit.entityType.qaNonConformances',
  qa_sop_documents: 'adminUsers.audit.entityType.qaSopDocuments',
  qa_sop: 'adminUsers.audit.entityType.qaSopDocuments',
  qa_schedules: 'adminUsers.audit.entityType.qaSchedules',
  qa_schedule: 'adminUsers.audit.entityType.qaSchedules',
  qa_schedule_items: 'adminUsers.audit.entityType.qaScheduleItems',
  // 後端有時送複數型（plural）— 加 alias 避免白屏
  protocols: 'adminUsers.audit.entityType.protocol',
  animals: 'adminUsers.audit.entityType.animal',
  facilities: 'adminUsers.audit.entityType.facility',
  users: 'adminUsers.audit.entityType.user',
  // 補全：user_activity_logs 實際出現但先前缺映射（QAU 稽核摘要會露原始 code）
  annual_leave_entitlement: 'adminUsers.audit.entityType.annualLeaveEntitlement',
  application_notice: 'adminUsers.audit.entityType.applicationNotice',
  electronic_signature: 'adminUsers.audit.entityType.electronicSignature',
  planned_experiment: 'adminUsers.audit.entityType.plannedExperiment',
  protocol_template_version: 'adminUsers.audit.entityType.protocolTemplateVersion',
  pi_account_invite: 'adminUsers.audit.entityType.piAccountInvite',
  // 補全：後端 AuditEntity::new 有、先前缺映射（操作日誌／稽核檢視會露原始 code）
  // 設施
  species: 'adminUsers.audit.entityType.species',
  building: 'adminUsers.audit.entityType.building',
  zone: 'adminUsers.audit.entityType.zone',
  pen: 'adminUsers.audit.entityType.pen',
  department: 'adminUsers.audit.entityType.department',
  // 設備 / 會計 / 廢棄物再利用
  equipment_disposals: 'adminUsers.audit.entityType.equipmentDisposals',
  ap_payment: 'adminUsers.audit.entityType.apPayment',
  ar_receipt: 'adminUsers.audit.entityType.arReceipt',
  byproduct_sample: 'adminUsers.audit.entityType.byproductSample',
  // 動物（獸醫建議另一 code）
  animal_vet_advice: 'adminUsers.audit.entityType.vetRecommendation',
  // GLP 合規
  reference_standard: 'adminUsers.audit.entityType.referenceStandard',
  document_revision: 'adminUsers.audit.entityType.documentRevision',
  risk_entry: 'adminUsers.audit.entityType.riskEntry',
  change_request: 'adminUsers.audit.entityType.changeRequest',
  monitoring_point: 'adminUsers.audit.entityType.monitoringPoint',
  environment_reading: 'adminUsers.audit.entityType.environmentReading',
  competency_assessment: 'adminUsers.audit.entityType.competencyAssessment',
  training_requirement: 'adminUsers.audit.entityType.trainingRequirement',
  formulation_record: 'adminUsers.audit.entityType.formulationRecord',
  // 安全
  ip_blocklist: 'adminUsers.audit.entityType.ipBlocklist',
}

// ── 事件類別 → 可選資料類型 配對（labelKey 沿用 entity_type 的鍵）──
export const categoryEntityMap: Record<string, { value: string; labelKey: string }[]> = {
  all: [
    { value: 'document', labelKey: 'adminUsers.audit.entityType.document' },
    { value: 'product', labelKey: 'adminUsers.audit.entityType.product' },
    { value: 'warehouse', labelKey: 'adminUsers.audit.entityType.warehouse' },
    { value: 'partner', labelKey: 'adminUsers.audit.entityType.partner' },
    { value: 'blood_test_template', labelKey: 'adminUsers.audit.entityType.bloodTestTemplate' },
    { value: 'blood_test_panel', labelKey: 'adminUsers.audit.entityType.bloodTestPanel' },
    { value: 'animal', labelKey: 'adminUsers.audit.entityType.animal' },
    { value: 'animal_observation', labelKey: 'adminUsers.audit.entityType.animalObservation' },
    { value: 'animal_surgery', labelKey: 'adminUsers.audit.entityType.animalSurgery' },
    { value: 'animal_weight', labelKey: 'adminUsers.audit.entityType.animalWeight' },
    { value: 'animal_vaccination', labelKey: 'adminUsers.audit.entityType.animalVaccination' },
    { value: 'animal_sacrifice', labelKey: 'adminUsers.audit.entityType.animalSacrifice' },
    { value: 'animal_pathology', labelKey: 'adminUsers.audit.entityType.animalPathology' },
    { value: 'animal_blood_test', labelKey: 'adminUsers.audit.entityType.animalBloodTest' },
    { value: 'vet_recommendation', labelKey: 'adminUsers.audit.entityType.vetRecommendation' },
    { value: 'protocol', labelKey: 'adminUsers.audit.entityType.protocol' },
    { value: 'role', labelKey: 'adminUsers.audit.entityType.role' },
  ],
  ERP: [
    { value: 'document', labelKey: 'adminUsers.audit.entityType.document' },
    { value: 'product', labelKey: 'adminUsers.audit.entityType.product' },
    { value: 'warehouse', labelKey: 'adminUsers.audit.entityType.warehouse' },
    { value: 'partner', labelKey: 'adminUsers.audit.entityType.partner' },
    { value: 'blood_test_template', labelKey: 'adminUsers.audit.entityType.bloodTestTemplate' },
    { value: 'blood_test_panel', labelKey: 'adminUsers.audit.entityType.bloodTestPanel' },
  ],
  AUP: [
    { value: 'protocol', labelKey: 'adminUsers.audit.entityType.protocol' },
  ],
  ANIMAL: [
    { value: 'animal', labelKey: 'adminUsers.audit.entityType.animal' },
    { value: 'animal_observation', labelKey: 'adminUsers.audit.entityType.animalObservation' },
    { value: 'animal_surgery', labelKey: 'adminUsers.audit.entityType.animalSurgery' },
    { value: 'animal_weight', labelKey: 'adminUsers.audit.entityType.animalWeight' },
    { value: 'animal_vaccination', labelKey: 'adminUsers.audit.entityType.animalVaccination' },
    { value: 'animal_sacrifice', labelKey: 'adminUsers.audit.entityType.animalSacrifice' },
    { value: 'animal_pathology', labelKey: 'adminUsers.audit.entityType.animalPathology' },
    { value: 'animal_blood_test', labelKey: 'adminUsers.audit.entityType.animalBloodTest' },
    { value: 'vet_recommendation', labelKey: 'adminUsers.audit.entityType.vetRecommendation' },
  ],
}

// ── 安全稽核警示類型（security_alerts.alert_type）──
const ALERT_TYPE_LABEL_KEYS: Record<string, string> = {
  CSP_VIOLATION: 'adminUsers.audit.alertType.cspViolation',
  CSP_VIOLATION_REPORT_ONLY: 'adminUsers.audit.alertType.cspViolationReportOnly',
  REFRESH_TOKEN_REUSE: 'adminUsers.audit.alertType.refreshTokenReuse',
  brute_force: 'adminUsers.audit.alertType.bruteForce',
  idor_probe: 'adminUsers.audit.alertType.idorProbe',
  unusual_login: 'adminUsers.audit.alertType.unusualLogin',
}

// ── 警示嚴重度（security_alerts.severity）──
const SEVERITY_LABEL_KEYS: Record<string, string> = {
  critical: 'adminUsers.audit.severity.critical',
  high: 'adminUsers.audit.severity.high',
  warning: 'adminUsers.audit.severity.warning',
  medium: 'adminUsers.audit.severity.medium',
  info: 'adminUsers.audit.severity.info',
  low: 'adminUsers.audit.severity.low',
}

// ── 取譯函式（找不到對照回 undefined，呼叫端自行 fallback 顯示原 code）──

/** 事件類別（event_category）顯示名稱 */
export function getCategoryLabel(t: TFunction, category: string): string | undefined {
  return has(CATEGORY_LABEL_KEYS, category) ? t(CATEGORY_LABEL_KEYS[category]) : undefined
}

/** 事件類型（event_type）的顯示名稱與徽章顏色 */
export function getEventTypeConfig(
  t: TFunction,
  eventType: string,
): { label: string; color: string } | undefined {
  if (!has(EVENT_TYPE_CONFIG, eventType)) return undefined
  const { labelKey, color } = EVENT_TYPE_CONFIG[eventType]
  return { label: t(labelKey), color }
}

/** 資料類型（entity_type）顯示名稱 */
export function getEntityTypeLabel(t: TFunction, entityType: string): string | undefined {
  return has(ENTITY_TYPE_LABEL_KEYS, entityType) ? t(ENTITY_TYPE_LABEL_KEYS[entityType]) : undefined
}

/** 安全警示類型（security_alerts.alert_type）顯示名稱 */
export function getAlertTypeLabel(t: TFunction, alertType: string): string | undefined {
  return has(ALERT_TYPE_LABEL_KEYS, alertType) ? t(ALERT_TYPE_LABEL_KEYS[alertType]) : undefined
}

/** 警示嚴重度（security_alerts.severity）顯示名稱 */
export function getSeverityLabel(t: TFunction, severity: string): string | undefined {
  return has(SEVERITY_LABEL_KEYS, severity) ? t(SEVERITY_LABEL_KEYS[severity]) : undefined
}

/**
 * @deprecated 相容匯出：`pages/admin/QAUDashboardPage.tsx`（不同 slice）仍以
 * `entityTypeLabels[type] ?? type` 讀取。改以 Proxy 在「存取當下」用 i18n.t 取譯，
 * 語言切換後讀到的就是新語言；新程式請改用 getEntityTypeLabel(t, code)。
 */
export const entityTypeLabels: Record<string, string> = new Proxy(
  {} as Record<string, string>,
  {
    get: (_target, prop) =>
      typeof prop === 'string' && has(ENTITY_TYPE_LABEL_KEYS, prop)
        ? i18n.t(ENTITY_TYPE_LABEL_KEYS[prop])
        : undefined,
    has: (_target, prop) => typeof prop === 'string' && has(ENTITY_TYPE_LABEL_KEYS, prop),
    ownKeys: () => Object.keys(ENTITY_TYPE_LABEL_KEYS),
    getOwnPropertyDescriptor: (_target, prop) =>
      typeof prop === 'string' && has(ENTITY_TYPE_LABEL_KEYS, prop)
        ? { enumerable: true, configurable: true, value: i18n.t(ENTITY_TYPE_LABEL_KEYS[prop]) }
        : undefined,
  },
)
