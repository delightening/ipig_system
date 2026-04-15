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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(500));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert!((config.backoff_factor - 2.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_with_retry_succeeds_first_try() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_factor: 2.0,
        };
        let result: std::result::Result<i32, String> =
            with_retry(&config, "test_op", || async { Ok(42) }).await;
        assert_eq!(result.expect("should succeed on first try"), 42);
    }

    #[tokio::test]
    async fn test_with_retry_succeeds_after_failures() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_factor: 2.0,
        };
        let attempt = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_clone = attempt.clone();

        let result: std::result::Result<&str, String> =
            with_retry(&config, "flaky_op", || {
                let a = attempt_clone.clone();
                async move {
                    let n = a.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if n < 2 {
                        Err(format!("fail #{}", n))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert_eq!(result.expect("should succeed after retries"), "success");
        assert_eq!(attempt.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_with_retry_exhausts_retries() {
        let config = RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_factor: 2.0,
        };
        let attempt = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_clone = attempt.clone();

        let result: std::result::Result<(), String> =
            with_retry(&config, "always_fail", || {
                let a = attempt_clone.clone();
                async move {
                    a.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Err("permanent failure".to_string())
                }
            })
            .await;

        assert_eq!(result.expect_err("should fail after exhausting retries"), "permanent failure");
        // max_retries=2 → 1 initial + 2 retries = 3 attempts
        assert_eq!(attempt.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_with_retry_zero_retries() {
        let config = RetryConfig {
            max_retries: 0,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_factor: 2.0,
        };

        let result: std::result::Result<(), String> =
            with_retry(&config, "no_retry", || async {
                Err("fail".to_string())
            })
            .await;

        assert_eq!(result.expect_err("should fail with zero retries"), "fail");
    }
}
