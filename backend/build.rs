//! R30-22 follow-up: 將 GIT_SHA / BUILD_TIME / RUSTC_VERSION_RUNTIME 注入為
//! `cargo:rustc-env`，讓 `handlers/version.rs` 透過 `option_env!` 在 runtime 取得。
//!
//! 注入優先順序（皆 fallback 到合理預設）：
//!   1. **環境變數**（CI / Dockerfile 注入）— 最高優先
//!   2. **`git` 命令**（local dev 環境，當 build host 可存取 .git/）
//!   3. **預設值**（"unknown" 或 build.rs 啟動時的 system time）
//!
//! 本檔不引入額外 build-dependency；只用 `std::process::Command` + `std::env`。

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // 任何環境變數變動或 .git/HEAD 移動時都重跑 build.rs
    println!("cargo:rerun-if-env-changed=GIT_SHA");
    println!("cargo:rerun-if-env-changed=BUILD_TIME");
    println!("cargo:rerun-if-env-changed=RUSTC_VERSION_RUNTIME");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");

    // GIT_SHA：env > `git rev-parse --short=12 HEAD` > "unknown"
    let git_sha = std::env::var("GIT_SHA")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            Command::new("git")
                .args(["rev-parse", "--short=12", "HEAD"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_SHA={git_sha}");

    // BUILD_TIME：env > 當前 UTC ISO 8601 (YYYY-MM-DDTHH:MM:SSZ)
    let build_time = std::env::var("BUILD_TIME")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            // 不引入 chrono build-dep，自己用 unix timestamp 算 UTC ISO 8601
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format_iso8601_utc(secs)
        });
    println!("cargo:rustc-env=BUILD_TIME={build_time}");

    // RUSTC_VERSION_RUNTIME：env > `rustc --version` > "unknown"
    let rustc_version = std::env::var("RUSTC_VERSION_RUNTIME")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            Command::new("rustc")
                .arg("--version")
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=RUSTC_VERSION_RUNTIME={rustc_version}");
}

/// 把 unix epoch seconds 格式化成 ISO 8601 UTC（"YYYY-MM-DDTHH:MM:SSZ"）。
/// 用途：build.rs fallback when BUILD_TIME env 未注入。
/// 不依賴 chrono；用 days-since-epoch 算 calendar。
fn format_iso8601_utc(secs: u64) -> String {
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;

    // Civil from days (Howard Hinnant algorithm)
    let z = days as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m_civ = if mp < 10 { mp + 3 } else { mp.wrapping_sub(9) };
    let year = if m_civ <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, m_civ, d, h, m, s
    )
}
