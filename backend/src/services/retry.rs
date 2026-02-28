use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
        }
    }
}

/// Retry an async operation with exponential backoff.
///
/// ```ignore
/// let result = with_retry(&RetryConfig::default(), "send_email", || async {
///     mailer.send(email.clone()).await
/// }).await;
/// ```
pub async fn with_retry<F, Fut, T, E>(
    config: &RetryConfig,
    operation_name: &str,
    f: F,
) -> std::result::Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = config.initial_delay;

    for attempt in 0..=config.max_retries {
        match f().await {
            Ok(val) => {
                if attempt > 0 {
                    tracing::info!(
                        "[Retry] {} 成功（第 {} 次重試）",
                        operation_name,
                        attempt,
                    );
                }
                return Ok(val);
            }
            Err(err) => {
                if attempt == config.max_retries {
                    tracing::error!(
                        "[Retry] {} 在 {} 次嘗試後失敗: {}",
                        operation_name,
                        config.max_retries + 1,
                        err,
                    );
                    return Err(err);
                }

                tracing::warn!(
                    "[Retry] {} 第 {} 次失敗: {}，{}ms 後重試",
                    operation_name,
                    attempt + 1,
                    err,
                    delay.as_millis(),
                );

                sleep(delay).await;
                delay = Duration::from_secs_f64(
                    (delay.as_secs_f64() * config.backoff_factor).min(config.max_delay.as_secs_f64()),
                );
            }
        }
    }

    unreachable!()
}
