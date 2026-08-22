//! R30-3a unit tests — pure logic only (DB-dependent paths covered by integration tests).

use super::{compute_next_attempt, ChannelRegistry, OutboxEvent, OutboxStatus};
use chrono::Utc;
use uuid::Uuid;

fn dummy_event(channel: &str) -> OutboxEvent {
    OutboxEvent {
        id: Uuid::new_v4(),
        channel: channel.to_string(),
        payload: serde_json::json!({}),
        status: "PENDING".into(),
        attempt_count: 0,
        next_attempt_at: Utc::now(),
        last_error: None,
        enqueued_by: None,
        enqueued_at: Utc::now(),
        started_at: None,
        done_at: None,
        source_entity: None,
        source_entity_id: None,
    }
}

#[test]
fn retry_table_attempt_1_returns_failed_with_10s_backoff() {
    let (status, next) = compute_next_attempt(1);
    assert_eq!(status, OutboxStatus::Failed);
    let delta = next - Utc::now();
    assert!(delta.num_seconds() >= 9 && delta.num_seconds() <= 11);
}

#[test]
fn retry_table_attempt_2_returns_failed_with_1m_backoff() {
    let (status, next) = compute_next_attempt(2);
    assert_eq!(status, OutboxStatus::Failed);
    let delta = next - Utc::now();
    assert!(delta.num_seconds() >= 59 && delta.num_seconds() <= 61);
}

#[test]
fn retry_table_attempt_3_returns_failed_with_10m_backoff() {
    let (status, next) = compute_next_attempt(3);
    assert_eq!(status, OutboxStatus::Failed);
    let secs = (next - Utc::now()).num_seconds();
    assert!((599..=601).contains(&secs), "expected ~10min, got {secs}s");
}

#[test]
fn retry_table_attempt_4_returns_failed_with_1h_backoff() {
    let (status, next) = compute_next_attempt(4);
    assert_eq!(status, OutboxStatus::Failed);
    let mins = (next - Utc::now()).num_minutes();
    assert!(mins == 59 || mins == 60, "expected ~60min, got {mins}min");
}

#[test]
fn retry_table_attempt_5_returns_failed_with_6h_backoff() {
    let (status, next) = compute_next_attempt(5);
    assert_eq!(status, OutboxStatus::Failed);
    // num_hours floors; allow [5, 6] window since some ms have passed since now()
    let hours = (next - Utc::now()).num_hours();
    assert!(hours == 5 || hours == 6, "expected ~6h, got {hours}h");
}

#[test]
fn retry_table_attempt_6_returns_dead() {
    let (status, _) = compute_next_attempt(6);
    assert_eq!(status, OutboxStatus::Dead);
}

#[test]
fn retry_table_higher_attempts_stay_dead() {
    for n in [7, 10, 100, i32::MAX] {
        let (status, _) = compute_next_attempt(n);
        assert_eq!(status, OutboxStatus::Dead, "attempt {n} should be DEAD");
    }
}

#[test]
fn outbox_status_as_str_roundtrip() {
    assert_eq!(OutboxStatus::Pending.as_str(), "PENDING");
    assert_eq!(OutboxStatus::Sending.as_str(), "SENDING");
    assert_eq!(OutboxStatus::Done.as_str(), "DONE");
    assert_eq!(OutboxStatus::Failed.as_str(), "FAILED");
    assert_eq!(OutboxStatus::Dead.as_str(), "DEAD");
}

#[tokio::test]
async fn channel_registry_unknown_channel_errors() {
    let registry = ChannelRegistry::new();
    let event = dummy_event("nonexistent");
    let result = registry.send(&event).await;
    assert!(result.is_err());
    let msg = format!(
        "{:?}",
        result.expect_err("registry should fail on unknown channel")
    );
    assert!(
        msg.contains("nonexistent"),
        "error should name the channel: {msg}"
    );
}

#[test]
fn channel_registry_register_returns_self_for_chaining() {
    use super::ChannelAdapter;
    use async_trait::async_trait;

    struct DummyAdapter;
    #[async_trait]
    impl ChannelAdapter for DummyAdapter {
        fn channel(&self) -> &'static str {
            "dummy"
        }
        async fn send(&self, _event: &OutboxEvent) -> crate::Result<()> {
            Ok(())
        }
    }

    let registry = ChannelRegistry::new().register(DummyAdapter);
    assert_eq!(registry.registered_channels(), vec!["dummy"]);
}

#[test]
#[should_panic(expected = "duplicate ChannelAdapter registration for channel 'dup'")]
fn channel_registry_panics_on_duplicate_registration() {
    use super::ChannelAdapter;
    use async_trait::async_trait;

    struct DupAdapter;
    #[async_trait]
    impl ChannelAdapter for DupAdapter {
        fn channel(&self) -> &'static str {
            "dup"
        }
        async fn send(&self, _event: &OutboxEvent) -> crate::Result<()> {
            Ok(())
        }
    }

    let _ = ChannelRegistry::new()
        .register(DupAdapter)
        .register(DupAdapter); // 期望 panic
}
