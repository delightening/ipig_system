# iPig System — AI-Assisted Cybersecurity Test Plan

> **Version:** 1.0  
> **Date:** 2026-04-14  
> **Scope:** Authorized penetration testing of iPig ERP (lab animal management system)  
> **Authorization:** Internal security audit, development environment only

---

## 1. System Attack Surface Summary

| Layer | Technology | Key Assets |
|-------|-----------|------------|
| Auth | JWT (HS256) + TOTP 2FA + Session mgmt | `/api/v1/auth/*` |
| API | Rust/Axum REST, 60+ endpoints | `/api/v1/*` |
| Frontend | React SPA + Axios + CSRF cookie | `frontend/` |
| Database | PostgreSQL + SQLx parameterized queries | 30+ tables |
| Middleware | Rate limiter, CSRF, Guest guard, ETag | `middleware/` |
| File Upload | Category-based size limits (10–100 MB) | `/api/v1/documents/*` |
| Email | SMTP (Lettre) — password reset, notifications | `services/email.rs` |
| PDF Gen | Gotenberg + Tera templates | `services/report.rs` |

---

## 2. AI-Assisted Test Scenarios

### Scenario A — Authentication & Session Attack Chain

**Goal:** Test the entire auth lifecycle for weaknesses using AI-driven fuzzing and pattern analysis.

#### A1. Brute Force & Account Lockout Bypass

| Item | Detail |
|------|--------|
| **Target** | `POST /api/v1/auth/login` |
| **AI Role** | Generate adaptive credential payloads that rotate timing, headers, and IP spoofing patterns to test rate limiter (30 req/min) and lockout (5 attempts → 30-min lock) |
| **Test Cases** | 1) Distributed brute force across multiple IPs to test per-IP vs per-account limiting<br>2) Timing analysis on error responses (constant-time comparison verification)<br>3) Lockout reset race condition — concurrent login attempts at exactly the boundary<br>4) `DISABLE_ACCOUNT_LOCKOUT` env var leak detection |
| **AI Technique** | Reinforcement learning agent that adapts request patterns based on response codes and timing |
| **Expected Defense** | Rate limiter blocks at 30/min; account locks after 5 failures; no timing side-channel |
| **Tool** | Custom Python script + `httpx` async client |

#### A2. JWT Token Manipulation

| Item | Detail |
|------|--------|
| **Target** | JWT validation in `middleware/auth.rs` |
| **AI Role** | LLM-generated token payloads: modify claims (`roles`, `permissions`, `exp`), test algorithm confusion (none/HS256/RS256), forge `jti` to bypass blacklist |
| **Test Cases** | 1) Algorithm substitution attack (`alg: none`)<br>2) Privilege escalation via tampered `roles` claim<br>3) Expired token replay after blacklist cleanup window<br>4) Cross-environment token reuse (wrong `aud`/`iss`)<br>5) `impersonated_by` field injection by non-admin |
| **AI Technique** | Generative fuzzer that mutates JWT structure guided by schema knowledge |
| **Expected Defense** | HS256 enforced; `aud`/`iss` validated; blacklist dual-layer (memory + DB) |

#### A3. Session Fixation & Concurrent Session Abuse

| Item | Detail |
|------|--------|
| **Target** | `services/session_manager.rs`, max 5 concurrent sessions |
| **AI Role** | Simulate rapid concurrent session creation to test race conditions in session counting logic |
| **Test Cases** | 1) Create 100 sessions simultaneously — verify max 5 enforced<br>2) Session fixation: reuse session ID across different users<br>3) Session hijacking via stolen cookie + IP mismatch detection<br>4) Heartbeat endpoint abuse for session keepalive flood |
| **Expected Defense** | Excess sessions terminated; IP/UA tracked per session |

---

### Scenario B — CSRF & Cross-Origin Attacks

**Goal:** Validate the Signed Double Submit Cookie (SEC-24) implementation.

#### B1. CSRF Token Forgery

| Item | Detail |
|------|--------|
| **Target** | `middleware/csrf.rs` |
| **AI Role** | Generate CSRF bypass attempts: predict HMAC nonces, replay old tokens, test cross-user token reuse |
| **Test Cases** | 1) Submit request with CSRF token from a different user session<br>2) Replay a valid CSRF token after logout/re-login<br>3) Timing attack on HMAC comparison (should be constant-time)<br>4) Requests to exempt endpoints (`login`, `refresh`) carry no CSRF — verify no state mutation |
| **AI Technique** | Statistical timing analysis + token pattern recognition |
| **Expected Defense** | HMAC bound to `user_id`; constant-time comparison; token rotation on auth state change |

#### B2. CORS Misconfiguration Probing

| Item | Detail |
|------|--------|
| **Target** | CORS layer in `routes/mod.rs` |
| **AI Role** | Enumerate allowed origins with wildcard patterns, test `null` origin, credentialed cross-origin requests |
| **Test Cases** | 1) `Origin: null` with credentials<br>2) Subdomain takeover reflection (`evil.allowed-domain.com`)<br>3) Pre-flight cache poisoning |

---

### Scenario C — Authorization & Privilege Escalation

**Goal:** Test RBAC enforcement across all 60+ endpoints.

#### C1. AI-Driven RBAC Fuzzing

| Item | Detail |
|------|--------|
| **Target** | All protected API endpoints |
| **AI Role** | Crawl OpenAPI spec / route definitions → build a permission matrix → systematically test every endpoint with every role combination |
| **Test Cases** | 1) Guest user accessing write endpoints (Guest guard bypass)<br>2) Regular user accessing `/api/v1/admin/*` endpoints<br>3) User A accessing User B's resources (IDOR via UUID enumeration)<br>4) Permission cache poisoning — modify permissions then verify 5-min TTL cache invalidation<br>5) Impersonation endpoint without reauth token |
| **AI Technique** | Graph-based access control model + automated endpoint enumeration |
| **Expected Defense** | Guest guard blocks mutations; admin middleware rejects non-admin; IDOR prevented by ownership checks |

#### C2. Horizontal Privilege Escalation (IDOR)

| Item | Detail |
|------|--------|
| **Target** | Resource endpoints with user-scoped data |
| **AI Role** | AI enumerates UUID patterns and tests cross-user resource access |
| **Endpoints** | `GET /api/v1/me/preferences/:key` — access other user's preferences<br>`PUT /api/v1/animals/:id` — modify animal not in user's scope<br>`GET /api/v1/hr/attendance` — view other user's attendance<br>`DELETE /api/v1/documents/:id` — delete other user's documents |

---

### Scenario D — Input Validation & Injection

**Goal:** Test all input surfaces for injection vulnerabilities.

#### D1. SQL Injection via SQLx Boundaries

| Item | Detail |
|------|--------|
| **Target** | All endpoints accepting user input |
| **AI Role** | Generate context-aware SQL injection payloads that specifically target PostgreSQL + SQLx patterns |
| **Test Cases** | 1) Standard SQLi in query parameters, JSON body, path params<br>2) Second-order SQLi — store payload in profile, trigger in report generation<br>3) SQLx compile-time check bypass — search for any `query!` with string interpolation<br>4) LIKE/ILIKE wildcard injection (`%`, `_`) for data exfiltration |
| **AI Technique** | LLM-guided payload generation with PostgreSQL dialect awareness |
| **Expected Defense** | SQLx parameterized queries prevent injection; no string concatenation in SQL |

#### D2. Template Injection (SSTI via Tera/Gotenberg)

| Item | Detail |
|------|--------|
| **Target** | PDF generation pipeline |
| **AI Role** | Inject Tera template expressions (`{{ }}`, `{% %}`) through user-controlled data that flows into PDF templates |
| **Test Cases** | 1) Animal name containing `{{ config }}` or `{% import %}` — rendered in PDF<br>2) Protocol description with template escape sequences<br>3) File name injection in document metadata |
| **Expected Defense** | Auto-escaping enabled; user data never treated as template code |

#### D3. File Upload Exploitation

| Item | Detail |
|------|--------|
| **Target** | `/api/v1/documents/*` upload endpoints |
| **AI Role** | Craft malicious files that bypass content-type validation |
| **Test Cases** | 1) Polyglot file (valid JPEG header + embedded script)<br>2) ZIP bomb / decompression bomb<br>3) Path traversal in filename (`../../../etc/passwd`)<br>4) Size limit bypass via chunked transfer encoding<br>5) SVG with embedded JavaScript |
| **Expected Defense** | Content-type validation; size limits enforced; filename sanitization |

---

### Scenario E — Business Logic Abuse

**Goal:** Test domain-specific logic flaws that scanners miss.

#### E1. AI-Modeled Business Flow Attacks

| Item | Detail |
|------|--------|
| **Target** | Multi-step business workflows |
| **AI Role** | Model state machines for critical workflows and find invalid state transitions |
| **Test Cases** | 1) **Inventory race condition:** Two concurrent stock adjustments on the same warehouse slot (capacity=0 means unlimited — test edge case)<br>2) **Protocol approval bypass:** Submit amendment then immediately access resources as if approved<br>3) **Blood test result tampering:** Modify finalized blood test results<br>4) **Attendance fraud:** Clock-in with spoofed GPS/IP, then clock-out from different location<br>5) **Audit log integrity:** Attempt to tamper with HMAC-SHA256 audit entries (SEC-34) |
| **AI Technique** | Finite state machine modeling + concurrency race detection |

#### E2. Password Reset Flow Abuse

| Item | Detail |
|------|--------|
| **Target** | `POST /api/v1/auth/forgot-password` → `POST /api/v1/auth/reset-password` |
| **AI Role** | Token prediction, timing attacks on token validation, race conditions |
| **Test Cases** | 1) Reset token reuse after password change<br>2) Enumerate valid emails via response timing difference<br>3) Concurrent reset requests — verify only latest token is valid<br>4) Token brute force (test entropy and rate limiting) |

---

### Scenario F — Infrastructure & Configuration

#### F1. Security Header Validation

| Item | Detail |
|------|--------|
| **AI Role** | Automated scan of all response headers against SEC-27 requirements |
| **Checks** | `X-Content-Type-Options: nosniff` ✓<br>`X-Frame-Options: DENY` ✓<br>`Strict-Transport-Security` (prod only) ✓<br>`Cache-Control: no-store` on API responses ✓<br>No `Server` header leaking Axum version |

#### F2. Environment Variable Leak Detection

| Item | Detail |
|------|--------|
| **AI Role** | Probe for config exposure via error messages, debug endpoints, stack traces |
| **Checks** | 1) `JWT_SECRET` not in any response body or log<br>2) `DATABASE_URL` not leaked in error responses<br>3) `SEED_DEV_USERS=true` detection in production<br>4) `/metrics` endpoint access control (currently bypasses rate limiting)<br>5) `/api/health` information disclosure |

#### F3. Dependency Supply Chain Audit

| Item | Detail |
|------|--------|
| **AI Role** | Analyze `Cargo.toml` and `package.json` for known CVEs, abandoned crates, typosquatting |
| **Tool** | `cargo audit` + `npm audit` + AI-powered dependency graph analysis |

---

## 3. AI Test Automation Architecture

```
┌─────────────────────────────────────────────────┐
│                  AI Test Orchestrator             │
│  (LLM-based decision engine — Claude/GPT-4)      │
├─────────────┬───────────────┬───────────────────┤
│  Recon Agent │  Attack Agent │  Analysis Agent   │
│  ───────────│  ────────────│  ────────────────  │
│  • OpenAPI   │  • Auth fuzzer│  • Response diff  │
│    parsing   │  • RBAC probe │  • Timing stats   │
│  • Route     │  • Injection  │  • Vuln classify  │
│    discovery │    payloads   │  • Risk scoring    │
│  • Tech      │  • Race cond. │  • Report gen     │
│    fingerpr. │    simulator  │  • Fix suggest     │
└──────┬───────┴───────┬──────┴────────┬──────────┘
       │               │               │
       v               v               v
┌─────────────────────────────────────────────────┐
│              Execution Layer                     │
│  • httpx async client (Python)                   │
│  • Playwright (browser-level CSRF tests)         │
│  • Custom Rust harness (for timing precision)    │
│  • Docker test environment (isolated)            │
└──────────────────────┬──────────────────────────┘
                       │
                       v
┌─────────────────────────────────────────────────┐
│           iPig Dev Environment                   │
│  • Backend: localhost:3000                        │
│  • Frontend: localhost:5173                       │
│  • PostgreSQL: localhost:5432                     │
│  • Gotenberg: localhost:3001                      │
└─────────────────────────────────────────────────┘
```

---

## 4. Risk Severity Matrix

| Scenario | Risk | Impact | Likelihood | Priority |
|----------|------|--------|------------|----------|
| A2 — JWT algorithm confusion | Critical | Full auth bypass | Low (Rust JWT crate strict) | P1 |
| C2 — IDOR on resources | High | Data breach | Medium | P1 |
| D2 — Template injection in PDF | High | RCE potential | Low-Medium | P1 |
| A1 — Brute force bypass | Medium | Account compromise | Medium | P2 |
| D3 — File upload exploit | High | RCE / data exfil | Low | P2 |
| E1 — Inventory race condition | Medium | Data integrity | Medium | P2 |
| B1 — CSRF token forgery | Medium | Unauthorized mutations | Low | P3 |
| F2 — Config leak in errors | Medium | Info disclosure | Medium | P3 |

---

## 5. Implementation Phases

### Phase 1 — Reconnaissance (Week 1)
- [ ] Parse route definitions → build complete endpoint inventory
- [ ] Extract permission requirements per endpoint from middleware chain
- [ ] Map data flows: user input → handler → service → repository → DB
- [ ] Identify all user-controlled inputs that reach template/SQL/file systems

### Phase 2 — Automated Scanning (Week 2)
- [ ] Deploy AI recon agent to classify endpoints by risk tier
- [ ] Run dependency audit (`cargo audit` + `npm audit`)
- [ ] Security header validation sweep
- [ ] Rate limiter stress test

### Phase 3 — Targeted Attacks (Week 3-4)
- [ ] Auth chain attacks (Scenarios A1–A3)
- [ ] RBAC fuzzing with full role matrix (Scenario C1)
- [ ] IDOR testing on all resource endpoints (Scenario C2)
- [ ] Injection testing (Scenarios D1–D3)
- [ ] Business logic abuse (Scenario E1)

### Phase 4 — Reporting & Remediation (Week 5)
- [ ] AI-generated vulnerability report with reproduction steps
- [ ] Risk-prioritized remediation plan
- [ ] Regression test suite for discovered vulnerabilities
- [ ] Update security documentation

---

## 6. Toolchain

| Tool | Purpose | License |
|------|---------|---------|
| **Claude Code** | AI orchestrator — payload generation, code review, report writing | Anthropic |
| **httpx** (Python) | Async HTTP client for API fuzzing | BSD-3 |
| **Playwright** | Browser-level CSRF/CORS testing | Apache 2.0 |
| **cargo audit** | Rust dependency CVE scanner | Apache 2.0 |
| **npm audit** | Node dependency CVE scanner | Built-in |
| **sqlmap** | SQL injection verification (manual mode) | GPL |
| **jwt_tool** | JWT manipulation and testing | MIT |
| **Burp Suite CE** | Manual interception and replay | Free |

---

## 7. Rules of Engagement

1. **Environment:** Development/staging only — never production
2. **Data:** Use seeded test accounts (`SEED_DEV_USERS=true`), no real user data
3. **Scope:** Only iPig application endpoints — no infrastructure (OS, network, cloud)
4. **Reporting:** All findings logged in `dev/security_findings/` with severity and reproduction steps
5. **Rollback:** All test data and state changes must be reversible
6. **Escalation:** Critical findings (RCE, auth bypass) immediately reported to project owner
