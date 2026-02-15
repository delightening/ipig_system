// GeoIP Service
// 使用 MaxMind GeoLite2-City 資料庫進行 IP 地理位置查詢

use maxminddb::{geoip2, Reader};
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;

/// GeoIP 查詢結果
#[derive(Debug, Clone, Default)]
pub struct GeoInfo {
    pub country: Option<String>,
    pub city: Option<String>,
    pub timezone: Option<String>,
}

/// GeoIP 服務
/// 使用 Arc 包裝，可安全在多執行緒間共享
#[derive(Clone)]
pub struct GeoIpService {
    reader: Option<Arc<Reader<Vec<u8>>>>,
}

impl GeoIpService {
    /// 從指定路徑載入 .mmdb 資料庫
    /// 如果載入失敗，服務仍可運作（降級模式，所有查詢回傳 None）
    pub fn new(db_path: &str) -> Self {
        let path = Path::new(db_path);
        if !path.exists() {
            tracing::warn!(
                "[GeoIP] 資料庫檔案不存在: {}，將以降級模式運行（不進行地理位置查詢）",
                db_path
            );
            return Self { reader: None };
        }

        match Reader::open_readfile(db_path) {
            Ok(reader) => {
                tracing::info!("[GeoIP] ✓ 已載入 GeoLite2-City 資料庫: {}", db_path);
                Self {
                    reader: Some(Arc::new(reader)),
                }
            }
            Err(e) => {
                tracing::error!(
                    "[GeoIP] 無法載入資料庫 {}: {}，將以降級模式運行",
                    db_path, e
                );
                Self { reader: None }
            }
        }
    }

    /// 查詢 IP 的地理位置資訊
    pub fn lookup(&self, ip_str: &str) -> Option<GeoInfo> {
        let reader = self.reader.as_ref()?;

        // 解析 IP 字串（移除可能的 CIDR 後綴，如 /32）
        let clean_ip = ip_str.split('/').next().unwrap_or(ip_str);
        let ip: IpAddr = clean_ip.parse().ok()?;

        // 查詢 GeoLite2-City 資料庫
        let city_result: geoip2::City = reader.lookup(ip).ok()?;

        let country = city_result
            .country
            .as_ref()
            .and_then(|c| c.names.as_ref())
            .and_then(|names| {
                // 優先使用中文名稱
                names.get("zh-CN")
                    .or_else(|| names.get("en"))
                    .map(|s| s.to_string())
            });

        let city = city_result
            .city
            .as_ref()
            .and_then(|c| c.names.as_ref())
            .and_then(|names| {
                names.get("zh-CN")
                    .or_else(|| names.get("en"))
                    .map(|s| s.to_string())
            });

        let timezone = city_result
            .location
            .as_ref()
            .and_then(|l| l.time_zone.map(|tz| tz.to_string()));

        Some(GeoInfo {
            country,
            city,
            timezone,
        })
    }

    /// 檢查 GeoIP 服務是否可用
    pub fn is_available(&self) -> bool {
        self.reader.is_some()
    }
}
