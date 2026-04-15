# Walkthrough: RTK Benefit Evaluation

## Date: 2026-04-15

## Task
Evaluate whether the [rtk-ai/rtk](https://github.com/rtk-ai/rtk) repository provides benefits for the iPig System project.

## What is RTK?
RTK (Rust Token Killer) is a single-binary CLI proxy written in Rust that intercepts terminal command outputs and compresses them before they reach an LLM's context window. It achieves 60-90% token reduction on common development commands with zero external dependencies.

## Decision
**Recommended to install** — minimal effort, meaningful token/cost savings for a project of this size.

## Key Findings

### Benefits for iPig System

| Area | Impact | Reason |
|------|--------|--------|
| Cargo build errors | **High** | Rust compiler output is verbose; 47K LOC backend means frequent noisy builds |
| Cargo test output | **High** | 142+ unit tests + integration tests produce compressible output |
| Git operations | **Medium** | 200+ files across backend/frontend; git diff/status can consume significant context |
| npm/vite output | **Medium** | 69K LOC TypeScript frontend; ESLint + TypeScript compiler output is verbose |
| Context preservation | **High** | More room for code understanding instead of compiler messages |
| Cost savings | **Medium-High** | 60-90% reduction on command output tokens |

### Non-Benefits
- Database queries (run through SQLx, not CLI)
- Application runtime (dev-time tool only)
- Frontend HMR (already minimal output)

### Integration
RTK supports Claude Code natively via hooks-based auto-rewrite:
```bash
cargo install --git https://github.com/rtk-ai/rtk
rtk setup claude-code
```

### Risk Assessment
- **Zero risk** — transparent proxy, no codebase changes required
- Can be removed at any time without impact
- Single Rust binary, no dependency chain concerns
