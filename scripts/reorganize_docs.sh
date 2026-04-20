#!/usr/bin/env bash
# docs/ 重組腳本 — Option C: 角色 + 生命週期
# 執行前確保在 repo 根目錄: cd "C:/System Coding/ipig_system/.claude/worktrees/busy-diffie-c0fa30"
set -euo pipefail

# ── Phase 1: 建立目錄結構 ────────────────────────────────────────────
mkdir -p docs/spec/{architecture,modules,guides}
mkdir -p docs/{dev,ops,runbooks,db,security,deploy,user}
mkdir -p docs/deploy/nas-migration
mkdir -p docs/dev/e2e
mkdir -p docs/archive/{code-reviews,improvement-plans,assessments,compliance-reports,heartbeat,feature-designs,legacy}
mkdir -p docs/archive/legacy/spec_v2

echo "✅ Directories created"

# ── Phase 2: spec/ (原 Profiling_Spec) ───────────────────────────────
git mv docs/Profiling_Spec/README.md                   docs/spec/README.md

# architecture/ 群組
git mv docs/Profiling_Spec/01_ARCHITECTURE_OVERVIEW.md  docs/spec/architecture/
git mv docs/Profiling_Spec/02_CORE_DOMAIN_MODEL.md       docs/spec/architecture/
git mv docs/Profiling_Spec/03_MODULES_AND_BOUNDARIES.md  docs/spec/architecture/
git mv docs/Profiling_Spec/04_DATABASE_SCHEMA.md         docs/spec/architecture/
git mv docs/Profiling_Spec/05_API_SPECIFICATION.md       docs/spec/architecture/
git mv docs/Profiling_Spec/06_PERMISSIONS_RBAC.md        docs/spec/architecture/
git mv docs/Profiling_Spec/07_SECURITY_AUDIT.md          docs/spec/architecture/
git mv docs/Profiling_Spec/09_EXTENSIBILITY.md           docs/spec/architecture/
git mv docs/Profiling_Spec/database_erd.md               docs/spec/architecture/
git mv docs/ARCHITECTURE.md                              docs/spec/architecture/
git mv docs/SYSTEM_RELATIONSHIPS.md                      docs/spec/architecture/

# modules/ 群組
git mv docs/Profiling_Spec/08_ATTENDANCE_MODULE.md        docs/spec/modules/
git mv docs/Profiling_Spec/modules/ANIMAL_MANAGEMENT.md   docs/spec/modules/
git mv docs/Profiling_Spec/modules/AUP_SYSTEM.md          docs/spec/modules/
git mv docs/Profiling_Spec/modules/ERP_SYSTEM.md          docs/spec/modules/
git mv docs/Profiling_Spec/modules/HR_SYSTEM.md           docs/spec/modules/
git mv docs/Profiling_Spec/modules/NOTIFICATION_SYSTEM.md docs/spec/modules/
git mv docs/project/AUP.md                                docs/spec/modules/
git mv docs/project/EMS_deployment.md                     docs/spec/modules/
git mv docs/MCP_Review_Server.md                          docs/spec/modules/
git mv docs/walkthrough_ai_api.md                         docs/spec/modules/AI_API_GUIDE.md

# guides/ 群組
git mv docs/Profiling_Spec/guides/AUDIT_LOGGING.md        docs/spec/guides/
git mv docs/Profiling_Spec/guides/NAMING_CONVENTIONS.md   docs/spec/guides/
git mv docs/Profiling_Spec/guides/README.md               docs/spec/guides/
git mv docs/Profiling_Spec/guides/STORAGE_SETUP.md        docs/spec/guides/
git mv docs/Profiling_Spec/guides/UI_UX_GUIDELINES.md     docs/spec/guides/

echo "✅ spec/ done"

# ── Phase 3: dev/ ────────────────────────────────────────────────────
git mv docs/development/INTEGRATION_TESTS.md  docs/dev/
git mv docs/development/ci-local.md           docs/dev/
git mv docs/e2e/README.md                     docs/dev/e2e/
git mv docs/e2e/FLOW.md                       docs/dev/e2e/

echo "✅ dev/ done"

# ── Phase 4: ops/ ────────────────────────────────────────────────────
git mv docs/operations/OPERATIONS.md                     docs/ops/
git mv docs/operations/COMPOSE.md                        docs/ops/
git mv docs/operations/ENV_AND_DB.md                     docs/ops/
git mv docs/operations/TUNNEL.md                         docs/ops/
git mv docs/operations/SSL_SETUP.md                      docs/ops/
git mv docs/operations/WINDOWS_BUILD.md                  docs/ops/
git mv docs/operations/infrastructure.md                 docs/ops/
git mv docs/operations/IMAGE_DIGESTS.md                  docs/ops/
git mv docs/operations/SMTP_CREDENTIALS.md               docs/ops/
git mv docs/operations/PRODUCT_IMPORT_LLM_SKU_GUIDELINES.md docs/ops/
git mv docs/operations/CSV_TO_MD_PRODUCT_LIST_RULES.md   docs/ops/
git mv docs/operations/STOCKLIST_TO_PRODUCT_IMPORT.md    docs/ops/
git mv docs/operations/SKU_CATEGORY_MANAGEMENT.md        docs/ops/
git mv docs/operations/PRODUCT_IMPORT_ROLLBACK.md        docs/ops/
git mv docs/VPS_CHEATSHEET.md                            docs/ops/
git mv docs/OBSERVABILITY_PLAN.md                        docs/ops/

echo "✅ ops/ done"

# ── Phase 5: runbooks/ (原位不動) ───────────────────────────────────
# runbooks/ 已在正確位置，不搬

# ── Phase 6: db/ (原 database/) ──────────────────────────────────────
git mv docs/database/DATA_IMPORT_RULES.md          docs/db/
git mv docs/database/DATA_IMPORT_TROUBLESHOOTING.md docs/db/
git mv docs/database/DB_ROLLBACK.md                docs/db/
git mv docs/database/FULL_DB_EXPORT_PLAN.md        docs/db/
git mv docs/database/RESTORE_OLD_DUMP.md           docs/db/
git mv docs/database/ZERO_DOWNTIME_MIGRATIONS.md   docs/db/

echo "✅ db/ done"

# ── Phase 7: security/ (原 security-compliance/) ─────────────────────
git mv docs/security-compliance/CREDENTIAL_ROTATION.md           docs/security/
git mv docs/security-compliance/DATA_RETENTION_POLICY.md         docs/security/
git mv docs/security-compliance/ELECTRONIC_SIGNATURE_COMPLIANCE.md docs/security/
git mv docs/security-compliance/GLP_VALIDATION.md                docs/security/
git mv docs/security-compliance/SESSION_LOGOUT_MANAGEMENT.md     docs/security/
git mv docs/security-compliance/SLA.md                           docs/security/
git mv docs/security-compliance/SOC2_READINESS.md                docs/security/
git mv docs/security-compliance/security.md                      docs/security/
git mv docs/THREAT_MODEL.md                                       docs/security/

echo "✅ security/ done"

# ── Phase 8: deploy/ ─────────────────────────────────────────────────
git mv docs/DEPLOYMENT.md                               docs/deploy/
git mv docs/DEPLOYMENT_NAS_DS923.md                    docs/deploy/
git mv docs/nas-migration/migration-sop.md              docs/deploy/nas-migration/
git mv docs/nas-migration/cloudflare-tunnel-config.yml  docs/deploy/nas-migration/
git mv docs/nas-migration/data-migration.sh             docs/deploy/nas-migration/
git mv docs/nas-migration/docker-compose.nas.yml        docs/deploy/nas-migration/

echo "✅ deploy/ done"

# ── Phase 9: user/ ───────────────────────────────────────────────────
git mv docs/QUICK_START.md  docs/user/
git mv docs/USER_GUIDE.md   docs/user/

echo "✅ user/ done"

# ── Phase 10: archive/code-reviews/ ──────────────────────────────────
git mv docs/2026_March15_code_review.md                          docs/archive/code-reviews/
git mv docs/2026_March15_code_review_1.md                        docs/archive/code-reviews/
git mv docs/codeReview/2026_March26_code_review.md               docs/archive/code-reviews/
git mv docs/codeReview/todo.md                                   docs/archive/code-reviews/code_review_todo.md
git mv docs/codeReview/recommended_tools.md                      docs/archive/code-reviews/
git mv docs/security_audit_v2_final.md                           docs/archive/code-reviews/
git mv docs/walkthrough_security_audit_2026_04_14.md             docs/archive/code-reviews/
git mv docs/walkthrough.md                                       docs/archive/code-reviews/
git mv docs/walkthrough_equipment_maintenance.md                 docs/archive/code-reviews/

echo "✅ archive/code-reviews/ done"

# ── Phase 11: archive/improvement-plans/ ─────────────────────────────
git mv docs/development/IMPROVEMENT_PLAN_R1.md             docs/archive/improvement-plans/
git mv docs/development/IMPROVEMENT_PLAN_R2.md             docs/archive/improvement-plans/
git mv docs/development/IMPROVEMENT_PLAN_R3.md             docs/archive/improvement-plans/
git mv docs/development/IMPROVEMENT_PLAN_R4.md             docs/archive/improvement-plans/
git mv docs/development/IMPROVEMENT_PLAN_R7.md             docs/archive/improvement-plans/
git mv docs/development/IMPROVEMENT_PLAN_MARKET_REVIEW.md  docs/archive/improvement-plans/
git mv docs/development/CI_IMPROVEMENT_PLAN.md             docs/archive/improvement-plans/
git mv docs/development/DEPENDABOT_MIGRATION_PLAN.md       docs/archive/improvement-plans/

echo "✅ archive/improvement-plans/ done"

# ── Phase 12: archive/assessments/ ───────────────────────────────────
git mv docs/assessments/R6-4_FINANCE_PHASE2_5_ASSESSMENT.md   docs/archive/assessments/
git mv docs/assessments/R6-5_DEPENDABOT_PHASE25_ASSESSMENT.md docs/archive/assessments/
git mv docs/assessments/PERFORMANCE_BENCHMARK.md               docs/archive/assessments/

echo "✅ archive/assessments/ done"

# ── Phase 13: archive/compliance-reports/ ────────────────────────────
git mv docs/AUP_HR_COMPLIANCE_ANALYSIS.md    docs/archive/compliance-reports/
git mv docs/COMPLIANCE_GAP_ANALYSIS.md       docs/archive/compliance-reports/
git mv docs/COMPLIANCE_DELIVERY_SUMMARY.md   docs/archive/compliance-reports/
git mv docs/session_summary_20260403.md      docs/archive/compliance-reports/

echo "✅ archive/compliance-reports/ done"

# ── Phase 14: archive/heartbeat/ ─────────────────────────────────────
git mv docs/heartbeat/README.md              docs/archive/heartbeat/
git mv docs/heartbeat/2026-03-30.md          docs/archive/heartbeat/
git mv docs/heartbeat/health-2026-03-29.md   docs/archive/heartbeat/
git mv docs/heartbeatImprovement.md          docs/archive/heartbeat/

echo "✅ archive/heartbeat/ done"

# ── Phase 15: archive/feature-designs/ ───────────────────────────────
git mv docs/development/DATA_EXPORT_IMPORT_DESIGN.md      docs/archive/feature-designs/
git mv docs/development/MIGRATION_REFACTOR_2026-03-01.md  docs/archive/feature-designs/
git mv docs/development/PERMISSION_AUDIT_2026-03-01.md    docs/archive/feature-designs/
git mv "docs/development/P2-R4-13_ANIMAL_ANY_ELIMINATION_PLAN.md" docs/archive/feature-designs/
git mv docs/development/REFACTOR_PLAN_USESTATE_TO_HOOKS.md docs/archive/feature-designs/
git mv docs/development/DRUG_OPTIONS_DEDUPLICATION.md      docs/archive/feature-designs/
git mv docs/development/CALENDAR_PAGE_DESIGN_AND_IMPROVEMENT.md docs/archive/feature-designs/
git mv docs/development/OPENAPI_AND_TESTS_STATUS.md        docs/archive/feature-designs/
git mv "docs/development/E2E TDD plan.md"                  docs/archive/feature-designs/
git mv docs/clientsAccess.md                               docs/archive/feature-designs/
git mv docs/AIReview.md                                    docs/archive/feature-designs/
git mv docs/r22-log-aggregation.md                         docs/archive/feature-designs/
git mv docs/R20_real_review_patterns.md                    docs/archive/feature-designs/
git mv docs/merge-plan.md                                  docs/archive/feature-designs/
git mv docs/ipig_implementation_plan.md                    docs/archive/feature-designs/
git mv "docs/建議_設備狀態與維護流程.md"                    docs/archive/feature-designs/
git mv "docs/符合性審查_設備狀態管理.md"                    docs/archive/feature-designs/
git mv "docs/專案目錄樹狀圖與職責定義.md"                   docs/archive/feature-designs/
git mv docs/operations/EDIT_CATEGORIES_REVIEW.md           docs/archive/feature-designs/
git mv "docs/project/手機端架構.md"                        docs/archive/feature-designs/

echo "✅ archive/feature-designs/ done"

# ── Phase 16: archive/legacy/ ────────────────────────────────────────
git mv "docs/laravel branch/現況分析.md"   docs/archive/legacy/laravel_現況分析.md
git mv "docs/laravel branch/重構計畫.md"   docs/archive/legacy/laravel_重構計畫.md

git mv docs/Profiling_Spec/archive/00_INDEX.md                    docs/archive/legacy/spec_v2/
git mv docs/Profiling_Spec/archive/ERPSpec.md                     docs/archive/legacy/spec_v2/
git mv docs/Profiling_Spec/archive/HR_SYSTEM_IMPLEMENTATION_PLAN.md docs/archive/legacy/spec_v2/
git mv "docs/Profiling_Spec/archive/_Spec.md"                     docs/archive/legacy/spec_v2/
git mv docs/Profiling_Spec/archive/database_tables.md             docs/archive/legacy/spec_v2/
git mv docs/Profiling_Spec/archive/database_tables_annotated.md   docs/archive/legacy/spec_v2/
git mv docs/Profiling_Spec/archive/ipigmanager.md                 docs/archive/legacy/spec_v2/
git mv docs/Profiling_Spec/archive/migration_plan.md              docs/archive/legacy/spec_v2/
git mv docs/Profiling_Spec/archive/product.md                     docs/archive/legacy/spec_v2/
git mv docs/Profiling_Spec/archive/role.md                        docs/archive/legacy/spec_v2/
git mv docs/Profiling_Spec/archive/skuSpec.md                     docs/archive/legacy/spec_v2/
git mv docs/Profiling_Spec/archive/uiSpec.md                      docs/archive/legacy/spec_v2/

git mv docs/project/VERSION_HISTORY.md                 docs/archive/legacy/
git mv docs/project/walkthrough.md                     docs/archive/legacy/project_walkthrough.md
git mv "docs/project/解構.md"                          docs/archive/legacy/
git mv docs/project/e2e-animals-failure-investigation.md docs/archive/legacy/
git mv docs/project/e2e-four-items-check.md            docs/archive/legacy/
git mv docs/project/e2e-vs-real-usage-discussion.md    docs/archive/legacy/
git mv docs/project/handover-2026-04-12.md             docs/archive/legacy/
git mv docs/project/description.md                     docs/archive/legacy/description_v8.md

echo "✅ archive/legacy/ done"

echo ""
echo "🎉 Phase 2-16 git mv complete. Now running sed link-fix pass..."
