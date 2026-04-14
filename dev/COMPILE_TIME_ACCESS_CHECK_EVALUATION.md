# Compile-Time Access Control Verification Evaluation

> **Report Date:** 2026-04-14  
> **Scope:** Feasibility assessment of compile-time permission checking mechanisms  
> **Target:** Replace runtime string-based `require_permission!()` checks with compile-time guarantees

---

## Executive Summary

The current permission system relies on **runtime string validation** (`require_permission!("animal.vet.recommend")`), which cannot detect missing checks at compile time. This report evaluates three architectural approaches to add compile-time verification, enabling zero-runtime-overhead checking while maintaining backward compatibility.

**Current State:**
- Permission coverage: 301/580 handlers (~52%)
- Zero compile-time detection of missing `require_permission!()` calls
- No automatic CI scanning for undeclared permission usage
- Risk: Silent failures if permission string misspelled or handler omitted check

**Key Finding:** Method A (Newtype Pattern + PhantomData) offers the lowest friction introduction with **immediate adoption in new code** while preserving existing code; Methods B & C provide stronger guarantees at higher integration cost.

---

## 1. Current State Analysis

### 1.1 Existing Permission Mechanism

**Location:** `backend/src/middleware/auth.rs:207-218`

```rust
#[macro_export]
macro_rules! require_permission {
    ($user:expr, $permission:expr) => {
        if !$user.has_permission($permission) {
            return Err($crate::AppError::Forbidden(format!(
                "Permission denied: requires {}",
                $permission
            )));
        }
    };
}
```

**Characteristics:**
- ✅ Simple macro, flexible string matching
- ✅ Works with dynamic permission strings
- ❌ No compile-time verification of permission names
- ❌ No type safety; `"animal.vet.reccommend"` typo undetected until runtime
- ❌ Cannot enforce that all public handlers call this macro

### 1.2 Coverage Analysis

**Statistics:**
- Total public async handler functions: 580
- Handlers with `require_permission!()`: 301 (~52%)
- Known gaps: `delete_vet_advice_record`, `delete_equipment`, `delete_vet_patrol_report`

**Why 52%?**
- Many handlers have coarser permission checks via middleware or parent service layer
- Some handlers re-use safe parent context (e.g., authenticated routes)
- Legacy code predates permission framework standardization

### 1.3 Defect Detection Gap

**Current CI/CD:**
- Clippy rules: Enabled (catches unused code, unsafe patterns)
- Custom grep rules: Only check permission name duplicates
- Missing: **Detection of handlers without `require_permission!()` call**

**Consequence:** A developer can write a new delete handler and forget the permission check; CI passes, code ships, access control bug exists in production.

---

## 2. Three Candidate Approaches

### 2.1 Method A: Newtype Pattern + PhantomData (Recommended for Phased Introduction)

#### Description

Create distinct Rust types for each permission, allowing the type system to enforce that a handler declares which permissions it needs.

```rust
// Step 1: Define permission tokens
pub struct PermissionToken<const NAME: &'static str>;

// More ergonomic wrapper via newtype
pub struct VetAdviceDelete;
pub struct VetAdviceRead;

// Step 2: Handler declares required permissions
pub async fn delete_vet_advice_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(_perm_check): Extension<PermissionToken<"animal.vet_advice.delete">>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    // compile-time guarantee: handler cannot be called without the permission check
    VetAdviceRecordService::delete(&state.db, id).await?;
    Ok(Json(()))
}

// Step 3: Middleware enforces check at route registration
// (Handled by request layer before handler invocation)
```

#### Trade-offs

| Aspect | Evaluation |
|--------|-----------|
| **Compile-time Strength** | ⭐⭐⭐ — Type system enforces permission token presence; typos caught at compile time |
| **Runtime Overhead** | ✅ None — tokens are zero-cost abstractions (compile-time only) |
| **Integration Friction** | ⭐⭐⭐⭐⭐ — Minimal; can adopt incrementally (new code uses it, old code unchanged) |
| **Learning Curve** | ⭐⭐⭐⭐ — Moderate; developers need to understand phantom types & generics |
| **Backward Compatibility** | ✅ Full — existing handlers unaffected; new handlers opt-in |
| **Maintenance Burden** | ⭐⭐⭐⭐ — Low; changes isolated to handler layer |
| **Interop with Macros** | ⚠️ Medium — requires careful macro design to inject tokens |

#### Implementation Roadmap

**Phase 1 (Sprint 1):** Prototype & validation
1. Define `PermissionToken<const NAME: &'static str>` type
2. Create ~5 token instances for common operations (create, read, update, delete)
3. Refactor 2-3 delete handlers as proof-of-concept
4. Run tests; measure zero-cost overhead

**Phase 2 (Sprint 2-3):** Systematic rollout
1. Generate permission tokens via proc-macro or constant generator
2. Update all sensitive handlers (create/update/delete first)
3. Add CI rule: warn if public async fn declared without permission token Extension

**Phase 3 (Ongoing):** Complete migration
1. Migrate remaining handlers to token-based checks
2. Deprecate raw `require_permission!()` macro in favor of token-based pattern

#### Example: Delete Vet Advice

**Before (Current):**
```rust
pub async fn delete_vet_advice_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    // ❌ Permission check missing or incorrectly named
    // require_permission!(current_user, "animal.vet.recommend.delete"); // optional at compile time
    VetAdviceRecordService::delete(&state.db, id).await?;
    Ok(Json(()))
}
```

**After (Method A):**
```rust
pub async fn delete_vet_advice_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(_perm_token): Extension<PermissionToken<"animal.vet_advice.delete">>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    // ✅ Compile-time guarantee: this handler cannot be called without the permission check
    VetAdviceRecordService::delete(&state.db, id).await?;
    Ok(Json(()))
}
```

**Route Registration:**
```rust
.route(
    "/vet-advice-records/:id",
    delete(
        // Middleware verifies token before handler runs
        require_perm_token::<"animal.vet_advice.delete">(delete_vet_advice_record)
    ),
)
```

---

### 2.2 Method B: Proc-Macro Code Generation

#### Description

Generate permission constants from a declarative attribute on handlers. Compiler enforces that every public handler declares required permissions.

```rust
// Input: attribute-based declaration
#[require_permissions("animal.vet_advice.delete", "animal.vet.recommend")]
pub async fn delete_vet_advice_record(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    // ... handler code
}

// Generated at compile time: 
// - Proc-macro expands attribute
// - Verifies permission constants exist in `permissions.rs`
// - Injects `require_permission!()` check or token
// - Stores metadata for CI analysis
```

#### Trade-offs

| Aspect | Evaluation |
|--------|-----------|
| **Compile-time Strength** | ⭐⭐⭐⭐ — Proc-macro can validate permission names against constant list |
| **Runtime Overhead** | ✅ None (macro generates inline code) |
| **Integration Friction** | ⭐⭐⭐ — Moderate; requires macro framework setup + attribute syntax |
| **Learning Curve** | ⭐⭐ — High; developers must understand proc-macros & derive tracing |
| **Backward Compatibility** | ⚠️ Partial — existing handlers need annotation audit before adoption |
| **Maintenance Burden** | ⭐⭐ — High; proc-macro requires careful testing & rustc version tracking |
| **Metadata Collection** | ✅ Excellent — macro can generate JSON/YAML for static analysis |

#### Implementation Roadmap

**Phase 1:** Proc-macro framework
1. Create `ipig-permission-derive` crate
2. Implement `#[require_permissions(...)]` derive macro
3. Code generation: Validate permission names at compile time

**Phase 2:** Gradual adoption
1. Audit all 580 handlers and annotate with required permissions
2. Proc-macro emits compile error if permission name not found in seed data

**Phase 3:** CI Analysis
1. Export metadata from proc-macro (JSON list of handlers → permissions)
2. CI rule: cross-reference handler list with API endpoint list; ensure all endpoints covered

#### Risk

- Proc-macros are **notoriously fragile** across rustc versions (especially during minor/major updates)
- High maintenance cost if async/await semantics change in future Rust editions
- Debugging macro errors is slow (full recompilation + error messages are often opaque)

---

### 2.3 Method C: Trait-Bound Handler Wrapper (Strongest Guarantee, Highest Cost)

#### Description

Define a sealed trait for handlers that guarantees permission checks via the type system. Only route registration accepts handlers implementing this trait.

```rust
// Step 1: Define handler trait
pub trait AccessControlledHandler {
    fn required_permissions() -> &'static [&'static str];
}

// Step 2: Handlers implement trait
pub struct DeleteVetAdviceHandler;

impl AccessControlledHandler for DeleteVetAdviceHandler {
    fn required_permissions() -> &'static [&'static str] {
        &["animal.vet_advice.delete"]
    }
}

// Step 3: Route registration accepts trait-bound handlers
pub struct ApiRoute<H: AccessControlledHandler> {
    handler: H,
    // ... route metadata
}

// Route registration enforces AccessControlledHandler bound
```

#### Trade-offs

| Aspect | Evaluation |
|--------|-----------|
| **Compile-time Strength** | ⭐⭐⭐⭐⭐ — Type system enforces trait implementation; zero escape paths |
| **Runtime Overhead** | ✅ None (compile-time trait checking) |
| **Integration Friction** | ⭐ — Very High; requires refactoring ALL route registration & handler signatures |
| **Learning Curve** | ⭐ — Very High; developers need deep Rust trait + lifetime knowledge |
| **Backward Compatibility** | ❌ None — Breaking change; cannot coexist with existing code |
| **Maintenance Burden** | ⭐⭐ — High; centralized route registration becomes complex |
| **Code Clarity** | ⭐⭐ — Medium; trait bounds can obscure handler intent |

#### Risk (Disqualifying Factors)

1. **Rewrite Magnitude:** Affects all 580 handlers + entire route registration system
2. **Learning Curve:** Requres all backend devs to grok sealed traits + associated types
3. **Incompatible with Axum Ecosystem:** Axum routes are handler-agnostic; forcing them into a trait breaks composability
4. **Maintenance Debt:** Future async/routing changes in Axum may break the trait system

**Recommendation:** **Not recommended** for immediate adoption due to breaking nature and ecosystem incompatibility.

---

## 3. Recommendation: Hybrid Phased Approach

### 3.1 Recommended Strategy

**Phase 1 (Sprint 2-3): Method A - Newtype Tokens**
- ✅ Immediate adoption in **new code only**
- ✅ Zero disruption to existing handlers
- ✅ Compile-time verification for future deletions
- Effort: ~2 weeks (2 devs: macro framework + 5 POC refactors + tests)

**Phase 2 (Sprint 4-6): CI Grep Scanning**
- ✅ Identify all handlers without `require_permission!()` via CLI
- ✅ Automated reports in CI/CD
- Effort: ~1 week (1 dev: grep rule + CI integration)

**Phase 3 (Sprint 7+): Gradual Token Migration**
- ✅ Migrate sensitive handlers (delete, update, create) to token-based checks
- ✅ Legacy handlers coexist; no forced rewrite
- Effort: 2-3 sprints depending on handler volume

**Phase 4 (Optional, Future): Method B - Proc-Macro**
- ⚠️ Consider if token pattern proves insufficient
- ⚠️ Higher maintenance cost; only adopt if team has macro expertise

### 3.2 Acceptance Criteria

**Phase 1 Completion:**
- [ ] Newtype `PermissionToken<const NAME: &'static str>` type defined & documented
- [ ] 5+ permission token instances created (CRUD + custom actions)
- [ ] Middleware updated to inject tokens into handler context
- [ ] 2-3 delete handlers refactored as POC; tests pass
- [ ] **Zero-cost overhead verified** (binary size & runtime perf unchanged)

**Phase 2 Completion:**
- [ ] `backend/src/bin/check_missing_permissions.rs` CLI tool created
- [ ] CI rule in `.github/workflows/ci.yml` runs grep scan on PRs
- [ ] Report generated: list of handlers without `require_permission!()` call
- [ ] Sensitive operations (delete, update) flagged for audit

**Phase 3 Completion:**
- [ ] All delete handlers migrated to token-based checks (or documented rationale for exceptions)
- [ ] Permission documentation updated in `DESIGN.md` § 13
- [ ] Team training session on token pattern usage

---

## 4. Cost-Benefit Analysis

| Metric | Method A | Method B | Method C |
|--------|----------|----------|----------|
| **Impl. Cost (person-weeks)** | 2-3 | 4-6 | 8-10 |
| **Maintenance Burden (annual)** | Low | Medium | High |
| **Compile-time Coverage** | 95% (new code) | 100% | 100% |
| **Team Learning Curve** | 2-3 days | 1 week | 2+ weeks |
| **Backward Compatibility** | ✅ Full | ⚠️ Partial | ❌ None |
| **Ecosystem Alignment (Axum)** | ✅ High | ✅ High | ❌ Low |
| **Recommended?** | ✅ YES | ⚠️ Future | ❌ No |

---

## 5. Implementation Checklist (Method A + CI Scanning)

### Sprint 2 (Week 1-2)

- [ ] **Design Review:** Newtype pattern & token injection strategy with team
- [ ] **Implement:** `backend/src/middleware/permission_token.rs`
  - [ ] Define `PermissionToken<const NAME: &'static str>` generic
  - [ ] Implement `FromRequestParts` for automatic injection
  - [ ] Create permission token constants (e.g., `VET_ADVICE_DELETE`, `ANIMAL_CREATE`)
- [ ] **POC Refactors:** 3-5 delete handlers
  - [ ] `delete_vet_advice_record` 
  - [ ] `delete_equipment`
  - [ ] `delete_vet_patrol_report`
- [ ] **Tests:** Verify tokens are zero-cost; mock permission denied scenarios
- [ ] **Doc:** Update `DESIGN.md` § 13 with token usage examples

### Sprint 3 (Week 3-4)

- [ ] **Rollout:** Migrate remaining sensitive handlers (20-30 handlers)
- [ ] **CI Tool:** Implement `check_missing_permissions.rs` grep-based scanner
  - [ ] Detect public async fn without `require_permission!` call
  - [ ] Output: JSON report for CI integration
- [ ] **CI Rule:** Add to `.github/workflows/ci.yml`
  - [ ] Run scanner on every PR
  - [ ] Flag sensitive operations (delete, update, create)
  - [ ] Include report in PR comment

### Sprint 4+

- [ ] **Full Migration:** Remaining handlers (as time permits)
- [ ] **Monitoring:** Track handler coverage ratio in CI metrics
- [ ] **Post-Mortem:** Document lessons learned; consider Method B if gaps remain

---

## 6. Success Metrics

| Metric | Baseline (2026-04-14) | Target (2026-05-31) | Measurement |
|--------|-----|-----|-----|
| Permission check coverage | 52% (301/580) | >90% | `grep -r "require_permission" backend/src/handlers \| wc -l` |
| Delete operation coverage | ~20% (known gaps) | 100% | Manual audit of all delete handlers |
| Compile-time errors caught | 0 | >5 (in subsequent PRs) | Typos in permission string, missing token injection |
| Test skeleton quality | Unknown | 100% with `#[ignore]` + failing assertion | Code review checklist enforcement |
| JWT residual risk clarity | Ambiguous | Documented (5 min vs 6 hour) | § SEC-AUDIT-017 in security report |

---

## 7. Appendix: Glossary

- **Newtype Pattern:** Wrapping a type in a struct to provide type safety (e.g., `struct UserId(Uuid)` vs bare `Uuid`)
- **PhantomData:** Rust construct for compile-time type information without runtime overhead
- **Proc-Macro:** Compile-time plugin that transforms code before compilation
- **Sealed Trait:** Trait that cannot be implemented outside its crate (private marker trait)
- **FromRequestParts:** Axum extractor trait for automatic dependency injection into handlers

---

## 8. References

- [Axum Extractors](https://docs.rs/axum/latest/axum/extract/index.html)
- [Rust Sealed Traits](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
- [Proc-Macro Workshop](https://github.com/dtolnay/proc-macro-workshop)
- iPig DESIGN.md § 13. Permission Model (this codebase)

---

**Report Status:** ✅ COMPLETE — Ready for Technical Review & Planning

**Next Step:** Present findings to Tech Lead & team; finalize Phase 1 sprint task breakdowns.
