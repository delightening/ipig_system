# iPig System Security Audit Report

> **Date:** 2026-04-14  
> **Scope:** Full-stack static security analysis (Backend Rust/Axum + Frontend React/TS)  
> **Method:** AI-assisted code review + dependency audit  
> **Environment:** Development (worktree: naughty-cray)

---

## Executive Summary

| Category | Critical | High | Medium | Low | Info |
|----------|----------|------|--------|-----|------|
| Authentication & Session | 0 | 3 | 3 | 1 | 0 |
| Authorization (IDOR/RBAC) | 1 | 1 | 1 | 0 | 0 |
| Injection (SQL/SSTI/File) | 0 | 1 | 0 | 1 | 0 |
| Configuration & Headers | 0 | 0 | 2 | 2 | 2 |
| Dependencies (CVE) | 0 | 1 | 1 | 0 | 7 |
| Frontend (XSS/Client) | 0 | 1 | 1 | 1 | 0 |
| **Total** | **1** | **7** | **8** | **5** | **9** |

**Overall Assessment:** iPig demonstrates **enterprise-grade security architecture** with defense-in-depth. However, **10 actionable findings** (1 Critical + 7 High + 2 Medium) require remediation before production deployment.

---

## Finding Details

### CRITICAL

#### SEC-AUDIT-001: Document Handler Missing `check_access()` Implementation
- **Location:** `backend/src/handlers/document.rs:106,136,176,472`
- **Category:** IDOR (Insecure Direct Object Reference)
- **Description:** `DocumentService::check_access()` is called but **not implemented**. All document CRUD operations lack resource ownership verification. User A can view, edit, submit, and delete documents created by User B.
- **Impact:** Complete bypass of document-level access control
- **CVSS Estimate:** 8.6 (High)
- **Remediation:**
  ```rust
  impl DocumentService {
      pub fn check_access(current_user: &CurrentUser, owner_id: Uuid) -> Result<()> {
          if current_user.id == owner_id || current_user.is_admin() {
              Ok(())
          } else {
              Err(AppError::Forbidden("No permission".into()))
          }
      }
  }
  ```

---

### HIGH

#### SEC-AUDIT-002: Login Timing Side-Channel Enables Email Enumeration
- **Location:** `backend/src/services/auth/login.rs:59-89`
- **Category:** Authentication
- **Description:** When a user does not exist, the response returns after ~1ms (DB query only). When a user exists but password is wrong, response takes ~100ms+ (DB query + Argon2 verification). Attackers can enumerate valid email addresses by measuring response times.
- **Note:** The `forgot-password` endpoint correctly implements a fixed 200ms delay — but `login` does not.
- **Remediation:** Add constant-time delay to login responses regardless of user existence.

#### SEC-AUDIT-003: Reauth/2FA Token Missing Algorithm Enforcement
- **Location:** `backend/src/services/auth/password.rs:204` and `two_factor.rs:140`
- **Category:** Authentication
- **Description:** `Validation::default()` does not enforce HS256 algorithm. The main auth middleware correctly uses `Validation::new(Algorithm::HS256)`, but reauth tokens and 2FA temp tokens use the default, which may accept `alg: none`.
- **Remediation:** Change to `Validation::new(Algorithm::HS256)` in both locations.

#### SEC-AUDIT-004: Session Limit Race Condition (TOCTOU)
- **Location:** `backend/src/services/session_manager.rs:207-235`
- **Category:** Session Management
- **Description:** `end_excess_sessions()` performs check-then-act with separate SQL queries. Concurrent logins can each pass the count check and create sessions beyond the limit (e.g., 10 sessions instead of max 5).
- **Remediation:** Use `SELECT FOR UPDATE` or PostgreSQL advisory locks within a single transaction.

#### SEC-AUDIT-005: `impersonated_by` JWT Field Deserializable
- **Location:** `backend/src/middleware/auth.rs:26`
- **Category:** Privilege Escalation (requires JWT secret leak)
- **Description:** The `impersonated_by` field in JWT Claims uses `#[serde(default)]`, meaning it's deserialized from the token payload. If JWT_SECRET is compromised, an attacker can forge tokens claiming impersonation of any admin.
- **Remediation:** Use `#[serde(skip_deserializing)]` and set the field only during serialization in the service layer.

#### SEC-AUDIT-006: Document Receipt Status Endpoint Lacks Ownership Check
- **Location:** `backend/src/handlers/document.rs:503-512`
- **Category:** IDOR
- **Description:** `get_po_receipt_status` only checks `erp.document.view` permission but not document ownership. Any authenticated user can query receipt status of any document by UUID.
- **Remediation:** Add `DocumentService::check_access()` call before querying status.

#### SEC-AUDIT-007: SQL Query Built via `format!()` in AI Repository
- **Location:** `backend/src/repositories/ai.rs:171-181`
- **Category:** SQL Injection (Antipattern)
- **Description:** Dynamic `ORDER BY` clause is built using `format!()`. While `validate_sort_field()` whitelists column names, this pattern bypasses SQLx compile-time safety guarantees. If the validation function is modified or bypassed in the future, SQL injection becomes possible.
- **Remediation:** Replace with parameterized approach (CASE statement or static query variants).

#### SEC-AUDIT-008: `dangerouslySetInnerHTML` for SVG Signatures
- **Location:** `frontend/src/components/animal/SacrificeFormDialog.tsx:127`
- **Category:** XSS
- **Description:** SVG handwriting signatures rendered via `dangerouslySetInnerHTML`. Mitigated by `sanitizeSvg()` (DOMPurify with strict whitelist), but pattern is inherently risky.
- **Remediation:** Convert SVG to `<img src="data:image/svg+xml;base64,...">` to eliminate innerHTML entirely.

---

### MEDIUM

#### SEC-AUDIT-009: No Absolute Session Timeout
- **Location:** `backend/src/services/session_manager.rs:39-56`
- **Category:** Session Management
- **Description:** Sessions have idle timeout (30 min) but no absolute lifetime limit. A continuously active session (via heartbeat) can persist indefinitely, expanding the window for session hijacking.
- **Remediation:** Add `AND (NOW() - started_at) < INTERVAL '480 minutes'` to session validation queries.

#### SEC-AUDIT-010: 2FA Temp Token Brute-Force Window
- **Location:** `backend/src/services/auth/two_factor.rs:162-192`
- **Category:** Authentication
- **Description:** While per-email 2FA failure rate limiting exists (5 failures/5min), an attacker can trigger multiple temp tokens (by successfully entering passwords from different sessions) and brute-force each independently.
- **Remediation:** Add per-token attempt tracking or single-active-token enforcement.

#### SEC-AUDIT-011: Missing Content-Security-Policy (CSP) Header
- **Location:** `backend/src/startup/server.rs`
- **Category:** Configuration
- **Description:** No CSP header configured. While the SPA architecture limits XSS surface, CSP provides defense-in-depth against injected scripts.
- **Remediation:** Add `Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'`.

#### SEC-AUDIT-012: Rate Limiter Bypassable via IP Spoofing
- **Location:** `backend/src/middleware/rate_limiter.rs:107,174,196`
- **Category:** Configuration
- **Description:** When `TRUST_PROXY_HEADERS=true`, rate limiter trusts `X-Forwarded-For` header. With 50k IP slots, an attacker can generate ~2.5M requests/min by cycling spoofed IPs. Default is `false` (safe), but risk exists if misconfigured.
- **Remediation:** Document that `TRUST_PROXY_HEADERS` requires validated reverse proxy. Consider adding trusted proxy IP allowlist.

#### SEC-AUDIT-013: UUID Enumeration via Error Code Inconsistency
- **Location:** `backend/src/handlers/document.rs` (pattern)
- **Category:** Information Disclosure
- **Description:** Different HTTP status codes for "resource not found" (404) vs "no permission" (403) allow attackers to enumerate valid UUIDs by observing response codes.
- **Remediation:** Return 403 for both cases in sensitive resource endpoints.

#### SEC-AUDIT-014: ESLint Rule Disabled in File Upload Component
- **Location:** `frontend/src/components/ui/file-upload.tsx:65`
- **Category:** Code Quality / Security
- **Description:** `react-hooks/exhaustive-deps` disabled. While justified for blob URL cleanup, disabled linter rules can mask real dependency issues.
- **Remediation:** Add explicit dependency tracking or document exemption reason.

---

### LOW

#### SEC-AUDIT-015: Server Header Not Explicitly Removed
- **Location:** `backend/src/startup/server.rs`
- **Category:** Information Disclosure
- **Description:** Axum may expose `Server` header revealing framework identity.
- **Remediation:** Add `SetResponseHeaderLayer::remove("Server")`.

#### SEC-AUDIT-016: Refresh Token Max-Age Hardcoded
- **Location:** `backend/src/handlers/auth/cookie.rs:71`
- **Category:** Configuration
- **Description:** Refresh token expiry hardcoded to 7 days instead of configurable via environment variable.
- **Remediation:** Read from `config.jwt_refresh_expiration_days`.

#### SEC-AUDIT-017: Permission Cache 5-Minute Stale Window
- **Location:** `backend/src/middleware/auth.rs:127-166`
- **Category:** Authorization
- **Description:** After admin revokes a permission, the user retains access for up to 5 minutes (cache TTL). Acceptable trade-off for performance.
- **Remediation:** Optional: add cache invalidation on permission changes, or reduce TTL for sensitive permissions.

#### SEC-AUDIT-018: Default `APP_URL` is `localhost`
- **Location:** `backend/src/config.rs:202`
- **Category:** Configuration
- **Description:** Default `APP_URL` is `http://localhost`. If not overridden in production, password reset emails may contain invalid URLs.
- **Remediation:** Require explicit `APP_URL` in production (reject startup if unset and `COOKIE_SECURE=true`).

#### SEC-AUDIT-019: AI Rate Limiter In-Memory Only
- **Location:** `backend/src/middleware/ai_auth.rs:20-48`
- **Category:** Rate Limiting
- **Description:** AI API key rate limits are per-process. In multi-instance deployments, limits can be bypassed. Acceptable for single-instance but risky at scale.
- **Remediation:** Use Redis-backed rate limiting for multi-instance deployments.

---

### INFO (Unmaintained Dependencies)

| # | Package | Source | Advisory | Notes |
|---|---------|--------|----------|-------|
| 1 | bincode 1.3.3 | printpdf | RUSTSEC-2025-0141 | Unmaintained |
| 2 | fxhash 0.2.1 | printpdf | RUSTSEC-2025-0057 | Unmaintained |
| 3 | kuchiki 0.8.1 | printpdf | RUSTSEC-2023-0019 | Unmaintained |
| 4 | proc-macro-error 1.0.4 | printpdf | RUSTSEC-2024-0370 | Unmaintained |
| 5 | rand 0.7.3/0.8.5 | multiple | RUSTSEC-2026-0097 | Unsound with custom logger |
| 6 | rsa 0.9.10 | sqlx-mysql, jsonwebtoken | RUSTSEC-2023-0071 | Timing side-channel (no fix available) |
| 7 | follow-redirects 1.15.11 | axios (frontend) | GHSA-r4q5-vmmm-2653 | Auth header leak on cross-domain redirect |

---

## Dependency Audit Summary

### Backend (cargo audit)
- **2 vulnerabilities:** quinn-proto (High 8.7, DoS) + rsa (Medium 5.9, timing)
- **7 warnings:** Unmaintained/unsound transitive dependencies (mostly from `printpdf`)
- **Action:** Upgrade reqwest/quinn-proto; evaluate printpdf alternatives

### Frontend (npm audit)
- **1 vulnerability:** follow-redirects (Moderate, info exposure)
- **Action:** `npm audit fix`

---

## Security Architecture Strengths

The iPig system demonstrates mature security engineering:

| Feature | Implementation | Grade |
|---------|---------------|-------|
| Password Hashing | Argon2 + OsRng salt | A |
| CSRF Protection | Signed Double Submit Cookie (HMAC-SHA256, session-bound) | A |
| JWT Validation | HS256 enforced, aud/iss validated, dual-layer blacklist | A- |
| File Upload Security | Magic byte validation, ZIP traversal check, filename sanitization | A |
| Rate Limiting | 4-tier system, 50k IP tracking limit, HashMap DoS protection | A- |
| Account Lockout | 5 attempts / 30-min cooldown, configurable | A |
| Password Reset | 256-bit CSPRNG token, hashed storage, 1-hour expiry, timing-safe | A |
| Audit Logging | HMAC-SHA256 integrity, comprehensive coverage | A |
| Security Headers | X-Content-Type-Options, X-Frame-Options, HSTS, Cache-Control | B+ |
| Startup Validation | Rejects prod/dev conflicts, enforces JWT_SECRET length | A |
| Animal/Protocol RBAC | Union-based multi-role access checks | A |
| Input Validation | SQLx parameterized queries + Zod (frontend) + validator (backend) | A |

---

## Remediation Priority

### Immediate (Before Production)
1. **SEC-AUDIT-001** — Implement `DocumentService::check_access()` (CRITICAL)
2. **SEC-AUDIT-003** — Enforce HS256 in reauth/2FA token validation (HIGH)
3. **SEC-AUDIT-007** — Replace `format!()` SQL building in AI repository (HIGH)
4. **SEC-AUDIT-006** — Add ownership check to PO receipt status (HIGH)

### Short-Term (Sprint 1)
5. **SEC-AUDIT-002** — Add constant-time delay to login (HIGH)
6. **SEC-AUDIT-004** — Fix session limit race condition (HIGH)
7. **SEC-AUDIT-005** — Restrict `impersonated_by` deserialization (HIGH)
8. **SEC-AUDIT-011** — Add CSP header (MEDIUM)

### Medium-Term (Sprint 2-3)
9. **SEC-AUDIT-009** — Add absolute session timeout (MEDIUM)
10. **SEC-AUDIT-008** — Replace `dangerouslySetInnerHTML` with `<img>` for SVG (HIGH)
11. **SEC-AUDIT-012** — Document proxy trust requirements (MEDIUM)
12. **SEC-AUDIT-013** — Normalize error codes for IDOR prevention (MEDIUM)
13. Upgrade quinn-proto >= 0.11.14 (dependency CVE)
14. Run `npm audit fix` for follow-redirects (dependency CVE)

### Long-Term
15. Evaluate printpdf alternatives (7 unmaintained transitive deps)
16. Redis-backed rate limiting for multi-instance deployment
17. Add absolute session timeout enforcement

---

## Test Artifacts

| File | Description |
|------|-------------|
| `dev/CYBERSECURITY_AI_TEST_PLAN.md` | Full test plan with 6 scenarios |
| `dev/SECURITY_AUDIT_REPORT.md` | This report |
