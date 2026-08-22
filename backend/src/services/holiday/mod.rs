//! 台灣國定假日服務：抓取政府開放資料 + per-year 記憶體快取 + 失敗 fallback。
//!
//! 用途：員工通知 email 寄送時間窗需判斷某日是否為國定假日（見
//! `services/notification/send_window.rs`）。
//!
//! 設計：
//! - URL 模板可由 system_settings `holiday_calendar_url` 覆寫（`{year}` 替換為西元年），
//!   預設 `constants::HOLIDAY_CALENDAR_URL_TEMPLATE_DEFAULT`。
//! - per-year lazy load，成功才快取（避免暫時性網路失敗被永久快取為空）。
//! - **Fallback（抓取 / 解析失敗）**：該年視為「無國定假日」→ 時間窗退化為只看週六日，
//!   並記 `warn`。理由：寧可假日偶爾多寄一封，也不要因 API 故障把所有員工通知卡住。
//! - 抓取行為以 `HolidayFetcher` trait 抽象，單元測試可注入 fake。
//!
//! ⚠️ 實際政府端點格式須於有對外網路的環境驗證；建置沙箱因 egress 政策（403）無法 live 驗證。

mod parse;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};

use crate::constants::{HOLIDAY_CALENDAR_URL_TEMPLATE_DEFAULT, SETTING_HOLIDAY_CALENDAR_URL};
use crate::services::SystemSettingsService;

/// 假日年度資料抓取抽象（便於測試注入 fake）。
#[async_trait]
pub trait HolidayFetcher: Send + Sync {
    /// 抓取指定西元年的假日原始 JSON。
    async fn fetch_year(&self, year: i32) -> anyhow::Result<String>;
}

/// 以 HTTP 抓取政府開放資料的實作。
pub struct HttpHolidayFetcher {
    client: reqwest::Client,
    url_template: String,
}

impl HttpHolidayFetcher {
    fn new(url_template: String) -> anyhow::Result<Self> {
        // 啟動時即驗證模板，讓錯誤的 holiday_calendar_url 走 main.rs 的警告路徑，
        // 而非執行期每次抓取都靜默 fallback 成「無假日」才被發現。
        if !url_template.contains("{year}") {
            anyhow::bail!("holiday_calendar_url 缺少 {{year}} 佔位符: {url_template}");
        }
        if !(url_template.starts_with("http://") || url_template.starts_with("https://")) {
            anyhow::bail!("holiday_calendar_url 非有效 http(s) URL: {url_template}");
        }
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self {
            client,
            url_template,
        })
    }
}

#[async_trait]
impl HolidayFetcher for HttpHolidayFetcher {
    async fn fetch_year(&self, year: i32) -> anyhow::Result<String> {
        let url = self.url_template.replace("{year}", &year.to_string());
        let resp = self.client.get(&url).send().await?.error_for_status()?;
        Ok(resp.text().await?)
    }
}

/// 假日服務：持有 fetcher 與 per-year 快取。
pub struct HolidayService {
    fetcher: Arc<dyn HolidayFetcher>,
    cache: RwLock<HashMap<i32, Arc<HashSet<NaiveDate>>>>,
}

impl HolidayService {
    pub fn new(fetcher: Arc<dyn HolidayFetcher>) -> Self {
        Self {
            fetcher,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// 由 system_settings 解析 URL 模板（fallback 至預設常數）建構 HTTP 版服務。
    pub async fn from_settings(pool: &sqlx::PgPool) -> anyhow::Result<Self> {
        let url = SystemSettingsService::new(pool.clone())
            .get_setting_string(SETTING_HOLIDAY_CALENDAR_URL)
            .await
            .unwrap_or_else(|| HOLIDAY_CALENDAR_URL_TEMPLATE_DEFAULT.to_string());
        let fetcher = Arc::new(HttpHolidayFetcher::new(url)?);
        Ok(Self::new(fetcher))
    }

    /// 強制重新抓取並覆寫指定年份的快取（啟動暖機 / 每日刷新用）。
    ///
    /// **只有成功抓取 + 解析時才更新快取**；抓取或解析失敗 → 記 warn 並**保留原有快取**
    /// （避免外部 API 暫時故障時把好資料清空 → 退化為僅週末判斷）。
    pub async fn refresh_year(&self, year: i32) {
        match self.fetcher.fetch_year(year).await {
            Ok(body) => match parse::parse_holiday_year(&body) {
                Ok(set) => {
                    if let Ok(mut c) = self.cache.write() {
                        c.insert(year, Arc::new(set));
                    }
                }
                Err(e) => {
                    tracing::warn!(year, error = %e, "假日資料解析失敗，保留原快取（退化為僅週末判斷）");
                }
            },
            Err(e) => {
                tracing::warn!(year, error = %e, "假日資料抓取失敗，保留原快取（退化為僅週末判斷）");
            }
        }
    }

    /// 只讀快取（熱路徑用）：未暖機的年份回 None，呼叫端 fallback 為「無假日」。
    fn cached_year(&self, year: i32) -> Option<Arc<HashSet<NaiveDate>>> {
        self.cache.read().ok().and_then(|c| c.get(&year).cloned())
    }

    /// 蒐集日期區間內（inclusive）已暖機的放假日期，供「下一個寄送窗口」掃描使用。
    ///
    /// **刻意不在此發 HTTP**：通知分派是熱路徑（可能同步等待），抓取一律交由背景
    /// `refresh_year` 完成，避免 API 緩慢 / 斷線時阻塞業務流程造成網關逾時。
    /// 未暖機的年份視為「無假日」（fallback 僅週末判斷）。
    pub fn collect_holidays(&self, start: NaiveDate, end: NaiveDate) -> HashSet<NaiveDate> {
        let mut all = HashSet::new();
        for year in start.year()..=end.year() {
            if let Some(set) = self.cached_year(year) {
                for d in set.iter() {
                    if *d >= start && *d <= end {
                        all.insert(*d);
                    }
                }
            }
        }
        all
    }
}

/// 程序級全域單例。通知 chokepoint 與 scheduler 透過 `global()` 取用，避免將
/// `HolidayService` 逐層 thread 進只持有 `db` 的通知服務方法（與該層既有的
/// `load_config()` 全域風格一致）。未初始化時 `global()` 回 `None`，呼叫端 fallback
/// 為「無假日資訊」（僅週末判斷）。
static GLOBAL: OnceLock<Arc<HolidayService>> = OnceLock::new();

/// 初始化全域 HolidayService（main.rs 啟動時呼叫一次）。重複呼叫忽略。
pub fn init_global(service: Arc<HolidayService>) {
    let _ = GLOBAL.set(service);
}

/// 取得全域 HolidayService（未初始化回 None）。
pub fn global() -> Option<Arc<HolidayService>> {
    GLOBAL.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use std::sync::atomic::AtomicBool;

    struct FakeFetcher {
        body: String,
        calls: AtomicUsize,
        fail: AtomicBool,
    }

    impl FakeFetcher {
        fn ok(body: &str) -> Arc<Self> {
            Arc::new(Self {
                body: body.to_string(),
                calls: AtomicUsize::new(0),
                fail: AtomicBool::new(false),
            })
        }
    }

    #[async_trait]
    impl HolidayFetcher for FakeFetcher {
        async fn fetch_year(&self, _year: i32) -> anyhow::Result<String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail.load(Ordering::SeqCst) {
                anyhow::bail!("simulated fetch failure");
            }
            Ok(self.body.clone())
        }
    }

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    #[test]
    fn http_fetcher_validates_url_template() {
        // 缺 {year} 佔位符 → 啟動即錯
        assert!(HttpHolidayFetcher::new("https://example.com/data.json".to_string()).is_err());
        // 非 http(s) → 錯
        assert!(HttpHolidayFetcher::new("ftp://example.com/{year}.json".to_string()).is_err());
        // 合法 → ok
        assert!(HttpHolidayFetcher::new("https://example.com/{year}.json".to_string()).is_ok());
    }

    #[tokio::test]
    async fn refresh_loads_year_into_cache() {
        let fetcher = FakeFetcher::ok(r#"[{"date":"20260101","isHoliday":true}]"#);
        let svc = HolidayService::new(fetcher.clone());

        // 暖機前熱路徑只讀快取 → 空（fallback）
        assert!(svc.cached_year(2026).is_none());
        svc.refresh_year(2026).await;
        assert!(svc
            .cached_year(2026)
            .expect("warmed")
            .contains(&ymd(2026, 1, 1)));
    }

    #[tokio::test]
    async fn collect_holidays_does_not_fetch_on_hot_path() {
        let fetcher = FakeFetcher::ok(r#"[{"date":"20260101","isHoliday":true}]"#);
        let svc = HolidayService::new(fetcher.clone());
        // 未暖機 → 熱路徑回空且**不發 HTTP**
        assert!(svc
            .collect_holidays(ymd(2026, 1, 1), ymd(2026, 1, 31))
            .is_empty());
        assert_eq!(fetcher.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn refresh_failure_preserves_previous_cache() {
        let fetcher = FakeFetcher::ok(r#"[{"date":"20260101","isHoliday":true}]"#);
        let svc = HolidayService::new(fetcher.clone());
        svc.refresh_year(2026).await; // 成功暖機

        // 模擬 API 故障後再刷新 → 應保留原有好資料，不清空
        fetcher.fail.store(true, Ordering::SeqCst);
        svc.refresh_year(2026).await;
        assert!(svc
            .cached_year(2026)
            .expect("preserved")
            .contains(&ymd(2026, 1, 1)));
    }

    #[tokio::test]
    async fn collect_holidays_filters_range() {
        let fetcher = FakeFetcher::ok(
            r#"[{"date":"20260101","isHoliday":true},{"date":"20260601","isHoliday":true}]"#,
        );
        let svc = HolidayService::new(fetcher);
        svc.refresh_year(2026).await;
        let got = svc.collect_holidays(ymd(2026, 5, 1), ymd(2026, 7, 1));
        assert!(got.contains(&ymd(2026, 6, 1)));
        assert!(!got.contains(&ymd(2026, 1, 1)));
    }
}
