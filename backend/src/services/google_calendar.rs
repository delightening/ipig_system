// Google Calendar API Client Service
// 使用 Service Account 認證與 Google Calendar 進行雙向同步

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::env;

use crate::{error::AppError, models::CalendarEvent, Result};

/// Google Calendar API 客戶端
pub struct GoogleCalendarClient {
    http_client: reqwest::Client,
    calendar_id: String,
}

/// Service Account JSON 金鑰結構
#[derive(Debug, Deserialize)]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
    token_uri: String,
}

/// Google OAuth2 Token Response
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: i64,
}

/// Google Calendar Events List Response
#[derive(Debug, Deserialize)]
struct EventsListResponse {
    items: Option<Vec<GoogleEvent>>,
    #[serde(rename = "nextPageToken")]
    #[allow(dead_code)]
    next_page_token: Option<String>,
}

/// Google Calendar Event (用於讀取)
#[derive(Debug, Deserialize)]
struct GoogleEvent {
    id: String,
    summary: Option<String>,
    start: Option<GoogleEventTime>,
    end: Option<GoogleEventTime>,
    description: Option<String>,
    location: Option<String>,
    #[serde(rename = "colorId")]
    color_id: Option<String>,
    #[serde(rename = "htmlLink")]
    html_link: Option<String>,
    #[allow(dead_code)]
    etag: Option<String>,
}

/// Google Calendar Event Time (用於讀取/寫入)
#[derive(Debug, Deserialize, Serialize, Clone)]
struct GoogleEventTime {
    #[serde(rename = "dateTime", skip_serializing_if = "Option::is_none")]
    date_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    #[serde(rename = "timeZone", skip_serializing_if = "Option::is_none")]
    time_zone: Option<String>,
}

/// 建立/更新事件的請求結構
#[derive(Debug, Serialize)]
struct CreateEventRequest {
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    start: GoogleEventTime,
    end: GoogleEventTime,
    #[serde(rename = "colorId", skip_serializing_if = "Option::is_none")]
    color_id: Option<String>,
}

/// 建立事件後的回應
#[derive(Debug, Deserialize)]
pub struct CreatedEventResponse {
    pub id: String,
    #[serde(rename = "htmlLink")]
    pub html_link: Option<String>,
    pub etag: Option<String>,
}

/// JWT Header
#[derive(Serialize)]
struct JwtHeader {
    alg: String,
    typ: String,
}

/// JWT Claims for Service Account
#[derive(Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    exp: i64,
    iat: i64,
}

/// 準備建立事件的資料
pub struct NewCalendarEvent {
    pub summary: String,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub all_day: bool,
    pub color_id: Option<String>,
}

impl GoogleCalendarClient {
    /// 建立新的 Google Calendar 客戶端
    pub fn new(calendar_id: &str) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            calendar_id: calendar_id.to_string(),
        }
    }

    /// 從 Google Calendar 獲取事件
    pub async fn fetch_events(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<CalendarEvent>> {
        let access_token = self.get_access_token().await?;

        let time_min = start_date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .to_rfc3339();
        let time_max = end_date
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc()
            .to_rfc3339();

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            urlencoding::encode(&self.calendar_id)
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&access_token)
            .query(&[
                ("timeMin", time_min.as_str()),
                ("timeMax", time_max.as_str()),
                ("singleEvents", "true"),
                ("orderBy", "startTime"),
                ("maxResults", "250"),
            ])
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to call Google Calendar API: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::Internal(format!(
                "Google Calendar API error: {}",
                error_text
            )));
        }

        let events_response: EventsListResponse = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse Google Calendar response: {}", e))
        })?;

        let events = events_response
            .items
            .unwrap_or_default()
            .into_iter()
            .filter_map(|event| self.convert_event(event))
            .collect();

        Ok(events)
    }

    /// 建立新事件到 Google Calendar
    pub async fn create_event(&self, event: NewCalendarEvent) -> Result<CreatedEventResponse> {
        let access_token = self.get_access_token().await?;

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            urlencoding::encode(&self.calendar_id)
        );

        let request_body = self.build_event_request(&event);

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create calendar event: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::Internal(format!(
                "Failed to create calendar event: {}",
                error_text
            )));
        }

        let created: CreatedEventResponse = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse create event response: {}", e))
        })?;

        Ok(created)
    }

    /// 更新已存在的事件
    pub async fn update_event(
        &self,
        event_id: &str,
        event: NewCalendarEvent,
    ) -> Result<CreatedEventResponse> {
        let access_token = self.get_access_token().await?;

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            urlencoding::encode(&self.calendar_id),
            urlencoding::encode(event_id)
        );

        let request_body = self.build_event_request(&event);

        let response = self
            .http_client
            .put(&url)
            .bearer_auth(&access_token)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update calendar event: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::Internal(format!(
                "Failed to update calendar event: {}",
                error_text
            )));
        }

        let updated: CreatedEventResponse = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse update event response: {}", e))
        })?;

        Ok(updated)
    }

    /// 刪除事件
    pub async fn delete_event(&self, event_id: &str) -> Result<()> {
        let access_token = self.get_access_token().await?;

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            urlencoding::encode(&self.calendar_id),
            urlencoding::encode(event_id)
        );

        let response = self
            .http_client
            .delete(&url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete calendar event: {}", e)))?;

        // 204 No Content 表示成功刪除，404 表示已經不存在（也算成功）
        if response.status().is_success() || response.status().as_u16() == 404 {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::Internal(format!(
                "Failed to delete calendar event: {}",
                error_text
            )))
        }
    }

    /// 取得單一事件 (用於衝突偵測)
    pub async fn get_event(&self, event_id: &str) -> Result<Option<CalendarEvent>> {
        let access_token = self.get_access_token().await?;

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            urlencoding::encode(&self.calendar_id),
            urlencoding::encode(event_id)
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get calendar event: {}", e)))?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::Internal(format!(
                "Failed to get calendar event: {}",
                error_text
            )));
        }

        let event: GoogleEvent = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse event response: {}", e))
        })?;

        Ok(self.convert_event(event))
    }

    /// 建立事件請求結構
    fn build_event_request(&self, event: &NewCalendarEvent) -> CreateEventRequest {
        let (start, end) = if event.all_day {
            // 全天事件使用 date 格式
            // Google Calendar 的結束日期是排他的，所以要加一天
            let end_date = event.end_date + Duration::days(1);
            (
                GoogleEventTime {
                    date: Some(event.start_date.format("%Y-%m-%d").to_string()),
                    date_time: None,
                    time_zone: None,
                },
                GoogleEventTime {
                    date: Some(end_date.format("%Y-%m-%d").to_string()),
                    date_time: None,
                    time_zone: None,
                },
            )
        } else {
            // 使用 dateTime 格式
            let start_dt = event
                .start_date
                .and_hms_opt(9, 0, 0)
                .unwrap()
                .and_utc()
                .to_rfc3339();
            let end_dt = event
                .end_date
                .and_hms_opt(18, 0, 0)
                .unwrap()
                .and_utc()
                .to_rfc3339();
            (
                GoogleEventTime {
                    date: None,
                    date_time: Some(start_dt),
                    time_zone: Some("Asia/Taipei".to_string()),
                },
                GoogleEventTime {
                    date: None,
                    date_time: Some(end_dt),
                    time_zone: Some("Asia/Taipei".to_string()),
                },
            )
        };

        CreateEventRequest {
            summary: event.summary.clone(),
            description: event.description.clone(),
            start,
            end,
            color_id: event.color_id.clone(),
        }
    }

    /// 取得 OAuth2 Access Token (使用 Service Account)
    async fn get_access_token(&self) -> Result<String> {
        let key_json = if let Ok(credentials_path) = env::var("GOOGLE_APPLICATION_CREDENTIALS") {
            std::fs::read_to_string(&credentials_path).map_err(|e| {
                AppError::Internal(format!(
                    "Failed to read service account key from {}: {}",
                    credentials_path, e
                ))
            })?
        } else if let Ok(key_str) = env::var("GOOGLE_CALENDAR_SERVICE_ACCOUNT_KEY") {
            key_str
        } else {
            return Err(AppError::Internal(
                "Neither GOOGLE_APPLICATION_CREDENTIALS nor GOOGLE_CALENDAR_SERVICE_ACCOUNT_KEY environment variable is set".to_string(),
            ));
        };

        let service_account: ServiceAccountKey = serde_json::from_str(&key_json)
            .map_err(|e| AppError::Internal(format!("Invalid service account key: {}", e)))?;

        let now = Utc::now().timestamp();
        let exp = now + 3600;

        let header = JwtHeader {
            alg: "RS256".to_string(),
            typ: "JWT".to_string(),
        };

        // 使用完整的 calendar scope (讀寫權限)
        let claims = JwtClaims {
            iss: service_account.client_email.clone(),
            scope: "https://www.googleapis.com/auth/calendar".to_string(),
            aud: service_account.token_uri.clone(),
            exp,
            iat: now,
        };

        let jwt = self.create_jwt(&header, &claims, &service_account.private_key)?;

        let response = self
            .http_client
            .post(&service_account.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get access token: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::Internal(format!(
                "Failed to get access token: {}",
                error_text
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse token response: {}", e)))?;

        Ok(token_response.access_token)
    }

    /// 建立 JWT (使用 RS256 簽名)
    fn create_jwt(
        &self,
        _header: &JwtHeader,
        claims: &JwtClaims,
        private_key_pem: &str,
    ) -> Result<String> {
        let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
            .map_err(|e| AppError::Internal(format!("Invalid RSA private key: {}", e)))?;

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            claims,
            &encoding_key,
        )
        .map_err(|e| AppError::Internal(format!("Failed to sign JWT: {}", e)))?;

        Ok(token)
    }

    /// 轉換 Google Event 為 CalendarEvent
    fn convert_event(&self, event: GoogleEvent) -> Option<CalendarEvent> {
        let summary = event.summary.unwrap_or_else(|| "(無標題)".to_string());

        let (start, all_day) = if let Some(ref start_time) = event.start {
            if let Some(ref dt) = start_time.date_time {
                (DateTime::parse_from_rfc3339(dt).ok()?.with_timezone(&Utc), false)
            } else if let Some(ref d) = start_time.date {
                let date = NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()?;
                (date.and_hms_opt(0, 0, 0)?.and_utc(), true)
            } else {
                return None;
            }
        } else {
            return None;
        };

        let end = if let Some(ref end_time) = event.end {
            if let Some(ref dt) = end_time.date_time {
                DateTime::parse_from_rfc3339(dt).ok()?.with_timezone(&Utc)
            } else if let Some(ref d) = end_time.date {
                let date = NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()?;
                // 全天事件的結束日期是排他的，需要減一天
                (date - Duration::days(1)).and_hms_opt(23, 59, 59)?.and_utc()
            } else {
                start + Duration::hours(1)
            }
        } else {
            start + Duration::hours(1)
        };

        Some(CalendarEvent {
            id: event.id,
            summary,
            start,
            end,
            all_day,
            description: event.description,
            location: event.location,
            color_id: event.color_id,
            html_link: event.html_link,
        })
    }
}
