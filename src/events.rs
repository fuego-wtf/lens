use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Events emitted during lens execution.
///
/// The framework provides 5 core event types. Lenses emit custom message types
/// via the `Data` variant, where `key` identifies the message type and `value`
/// contains the payload. Framework-owned renderers map `key` via `lens.output.yaml`.
///
/// # Example
///
/// ```ignore
/// // Lens emitting structured player output
/// tx.send(LensEvent::data("spotify", "player", json!({
///     "track": "Song Name",
///     "artist": "Artist",
///     "progress": 0.45
/// }))).await;
///
/// // Figma lens emitting phase data
/// tx.send(LensEvent::data("figma", "component_preview", json!({
///     "node_id": "123:456",
///     "thumbnail_url": "...",
///     "name": "Button"
/// }))).await;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LensEvent {
    /// Lens started execution
    Started {
        lens: String,
        task: String,
        #[serde(with = "system_time_serde")]
        timestamp: SystemTime,
    },

    /// Progress update (optional percent 0-100)
    Progress {
        lens: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        percent: Option<f32>,
        #[serde(with = "system_time_serde")]
        timestamp: SystemTime,
    },

    /// Generic data event for lens-specific message types.
    ///
    /// The `key` field identifies the message type (e.g., "player", "question",
    /// "component_preview"). The framework maps this key to render blocks defined
    /// in `lens.output.yaml`.
    Data {
        lens: String,
        /// Message type identifier - maps to an output definition
        key: String,
        /// JSON payload for the message type
        value: serde_json::Value,
        #[serde(with = "system_time_serde")]
        timestamp: SystemTime,
    },

    /// Task completed successfully
    Completed {
        lens: String,
        #[serde(with = "duration_serde")]
        duration: Duration,
        #[serde(with = "system_time_serde")]
        timestamp: SystemTime,
    },

    /// Task failed with error
    Failed {
        lens: String,
        error: String,
        recoverable: bool,
        #[serde(with = "system_time_serde")]
        timestamp: SystemTime,
    },

    /// Checkpoint for user review (MVP: informational, future: bidirectional)
    ///
    /// Emitted when a pipeline phase completes and the user may want to review
    /// the results before continuing. The frontend can display this data and
    /// optionally pause for user input.
    Checkpoint {
        lens: String,
        /// Phase that just completed
        phase: String,
        /// Phase output data for review
        data: serde_json::Value,
        /// Human-readable message about what was completed
        message: String,
        #[serde(with = "system_time_serde")]
        timestamp: SystemTime,
    },
}

impl LensEvent {
    /// Create a Started event
    pub fn started(lens: impl Into<String>, task: impl Into<String>) -> Self {
        Self::Started {
            lens: lens.into(),
            task: task.into(),
            timestamp: SystemTime::now(),
        }
    }

    /// Create a Progress event without percent
    pub fn progress(lens: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Progress {
            lens: lens.into(),
            message: message.into(),
            percent: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Create a Progress event with percent
    pub fn progress_with_percent(
        lens: impl Into<String>,
        message: impl Into<String>,
        percent: f32,
    ) -> Self {
        Self::Progress {
            lens: lens.into(),
            message: message.into(),
            percent: Some(percent.clamp(0.0, 100.0)),
            timestamp: SystemTime::now(),
        }
    }

    /// Create a Data event with a custom message type.
    ///
    /// # Arguments
    /// * `lens` - Lens identifier
    /// * `key` - Message type (maps to output definition in lens.output.yaml)
    /// * `value` - JSON payload for framework rendering
    pub fn data(
        lens: impl Into<String>,
        key: impl Into<String>,
        value: serde_json::Value,
    ) -> Self {
        Self::Data {
            lens: lens.into(),
            key: key.into(),
            value,
            timestamp: SystemTime::now(),
        }
    }

    /// Create a Completed event
    pub fn completed(lens: impl Into<String>, duration: Duration) -> Self {
        Self::Completed {
            lens: lens.into(),
            duration,
            timestamp: SystemTime::now(),
        }
    }

    /// Create a Failed event
    pub fn failed(lens: impl Into<String>, error: impl Into<String>, recoverable: bool) -> Self {
        Self::Failed {
            lens: lens.into(),
            error: error.into(),
            recoverable,
            timestamp: SystemTime::now(),
        }
    }

    /// Create a Checkpoint event for user review
    ///
    /// # Arguments
    /// * `lens` - Lens identifier
    /// * `phase` - Phase that completed (e.g., "deduplication", "validation")
    /// * `data` - Phase output data for review
    /// * `message` - Human-readable completion message
    pub fn checkpoint(
        lens: impl Into<String>,
        phase: impl Into<String>,
        data: serde_json::Value,
        message: impl Into<String>,
    ) -> Self {
        Self::Checkpoint {
            lens: lens.into(),
            phase: phase.into(),
            data,
            message: message.into(),
            timestamp: SystemTime::now(),
        }
    }

    /// Get the lens name from this event
    pub fn lens(&self) -> &str {
        match self {
            Self::Started { lens, .. } => lens,
            Self::Progress { lens, .. } => lens,
            Self::Data { lens, .. } => lens,
            Self::Completed { lens, .. } => lens,
            Self::Failed { lens, .. } => lens,
            Self::Checkpoint { lens, .. } => lens,
        }
    }

    /// Get the timestamp from this event
    pub fn timestamp(&self) -> SystemTime {
        match self {
            Self::Started { timestamp, .. } => *timestamp,
            Self::Progress { timestamp, .. } => *timestamp,
            Self::Data { timestamp, .. } => *timestamp,
            Self::Completed { timestamp, .. } => *timestamp,
            Self::Failed { timestamp, .. } => *timestamp,
            Self::Checkpoint { timestamp, .. } => *timestamp,
        }
    }

    /// Get the event type as a string (for testing assertions)
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::Started { .. } => "Started",
            Self::Progress { .. } => "Progress",
            Self::Data { .. } => "Data",
            Self::Completed { .. } => "Completed",
            Self::Failed { .. } => "Failed",
            Self::Checkpoint { .. } => "Checkpoint",
        }
    }
}

// Serde helpers for SystemTime and Duration
mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap();
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_started_event() {
        let event = LensEvent::started("figma", "decompose");

        match event {
            LensEvent::Started { lens, task, .. } => {
                assert_eq!(lens, "figma");
                assert_eq!(task, "decompose");
            }
            _ => panic!("Expected Started event"),
        }
    }

    #[test]
    fn test_progress_event_without_percent() {
        let event = LensEvent::progress("nolimit", "Validating commands...");

        match event {
            LensEvent::Progress { lens, message, percent, .. } => {
                assert_eq!(lens, "nolimit");
                assert_eq!(message, "Validating commands...");
                assert!(percent.is_none());
            }
            _ => panic!("Expected Progress event"),
        }
    }

    #[test]
    fn test_progress_event_with_percent() {
        let event = LensEvent::progress_with_percent("figma", "Processing...", 45.5);

        match event {
            LensEvent::Progress { lens, message, percent, .. } => {
                assert_eq!(lens, "figma");
                assert_eq!(message, "Processing...");
                assert_eq!(percent, Some(45.5));
            }
            _ => panic!("Expected Progress event"),
        }
    }

    #[test]
    fn test_progress_percent_clamped_to_100() {
        let event = LensEvent::progress_with_percent("test", "msg", 150.0);

        match event {
            LensEvent::Progress { percent, .. } => {
                assert_eq!(percent, Some(100.0));
            }
            _ => panic!("Expected Progress event"),
        }
    }

    #[test]
    fn test_progress_percent_clamped_to_0() {
        let event = LensEvent::progress_with_percent("test", "msg", -10.0);

        match event {
            LensEvent::Progress { percent, .. } => {
                assert_eq!(percent, Some(0.0));
            }
            _ => panic!("Expected Progress event"),
        }
    }

    #[test]
    fn test_data_event() {
        let value = json!({"track": "Song", "artist": "Artist"});
        let event = LensEvent::data("spotify", "player", value.clone());

        match event {
            LensEvent::Data { lens, key, value: v, .. } => {
                assert_eq!(lens, "spotify");
                assert_eq!(key, "player");
                assert_eq!(v, value);
            }
            _ => panic!("Expected Data event"),
        }
    }

    #[test]
    fn test_completed_event() {
        let duration = Duration::from_secs(5);
        let event = LensEvent::completed("figma", duration);

        match event {
            LensEvent::Completed { lens, duration: d, .. } => {
                assert_eq!(lens, "figma");
                assert_eq!(d, duration);
            }
            _ => panic!("Expected Completed event"),
        }
    }

    #[test]
    fn test_failed_event_recoverable() {
        let event = LensEvent::failed("nolimit", "Rate limit exceeded", true);

        match event {
            LensEvent::Failed { lens, error, recoverable, .. } => {
                assert_eq!(lens, "nolimit");
                assert_eq!(error, "Rate limit exceeded");
                assert!(recoverable);
            }
            _ => panic!("Expected Failed event"),
        }
    }

    #[test]
    fn test_failed_event_non_recoverable() {
        let event = LensEvent::failed("figma", "Auth failed", false);

        match event {
            LensEvent::Failed { lens, error, recoverable, .. } => {
                assert_eq!(lens, "figma");
                assert_eq!(error, "Auth failed");
                assert!(!recoverable);
            }
            _ => panic!("Expected Failed event"),
        }
    }

    #[test]
    fn test_lens_accessor() {
        let events = vec![
            LensEvent::started("a", "task"),
            LensEvent::progress("b", "msg"),
            LensEvent::data("c", "key", json!({})),
            LensEvent::completed("d", Duration::from_secs(1)),
            LensEvent::failed("e", "err", false),
        ];

        let lenses: Vec<&str> = events.iter().map(|e| e.lens()).collect();
        assert_eq!(lenses, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn test_timestamp_accessor() {
        let before = SystemTime::now();
        let event = LensEvent::started("test", "task");
        let after = SystemTime::now();

        let timestamp = event.timestamp();
        assert!(timestamp >= before);
        assert!(timestamp <= after);
    }

    #[test]
    fn test_started_event_serialization() {
        let event = LensEvent::started("figma", "decompose");
        let serialized = serde_json::to_string(&event).unwrap();

        assert!(serialized.contains("\"type\":\"started\""));
        assert!(serialized.contains("\"lens\":\"figma\""));
        assert!(serialized.contains("\"task\":\"decompose\""));
    }

    #[test]
    fn test_progress_event_serialization() {
        let event = LensEvent::progress_with_percent("test", "Working", 50.0);
        let serialized = serde_json::to_string(&event).unwrap();

        assert!(serialized.contains("\"type\":\"progress\""));
        assert!(serialized.contains("\"percent\":50.0"));
    }

    #[test]
    fn test_data_event_serialization() {
        let event = LensEvent::data("lens", "msg_type", json!({"key": "value"}));
        let serialized = serde_json::to_string(&event).unwrap();

        assert!(serialized.contains("\"type\":\"data\""));
        assert!(serialized.contains("\"key\":\"msg_type\""));
        assert!(serialized.contains("\"value\":{"));
    }

    #[test]
    fn test_completed_event_serialization() {
        let event = LensEvent::completed("test", Duration::from_millis(1500));
        let serialized = serde_json::to_string(&event).unwrap();

        assert!(serialized.contains("\"type\":\"completed\""));
        assert!(serialized.contains("\"duration\":1500"));
    }

    #[test]
    fn test_failed_event_serialization() {
        let event = LensEvent::failed("test", "error msg", true);
        let serialized = serde_json::to_string(&event).unwrap();

        assert!(serialized.contains("\"type\":\"failed\""));
        assert!(serialized.contains("\"recoverable\":true"));
    }

    #[test]
    fn test_checkpoint_event() {
        let data = json!({"components": 5, "deduplicated": 3});
        let event = LensEvent::checkpoint(
            "figma",
            "deduplication",
            data.clone(),
            "Review: 5 components, 3 unique",
        );

        match event {
            LensEvent::Checkpoint { lens, phase, data: d, message, .. } => {
                assert_eq!(lens, "figma");
                assert_eq!(phase, "deduplication");
                assert_eq!(d, data);
                assert_eq!(message, "Review: 5 components, 3 unique");
            }
            _ => panic!("Expected Checkpoint event"),
        }
    }

    #[test]
    fn test_checkpoint_event_serialization() {
        let event = LensEvent::checkpoint(
            "figma",
            "validation",
            json!({"passed": true}),
            "Validation complete",
        );
        let serialized = serde_json::to_string(&event).unwrap();

        assert!(serialized.contains("\"type\":\"checkpoint\""));
        assert!(serialized.contains("\"phase\":\"validation\""));
        assert!(serialized.contains("\"message\":\"Validation complete\""));
    }

    #[test]
    fn test_event_deserialization_roundtrip() {
        let original = LensEvent::data("spotify", "player", json!({"track": "Test"}));
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: LensEvent = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            LensEvent::Data { lens, key, value, .. } => {
                assert_eq!(lens, "spotify");
                assert_eq!(key, "player");
                assert_eq!(value["track"], "Test");
            }
            _ => panic!("Expected Data event"),
        }
    }

    #[test]
    fn test_event_clone() {
        let event = LensEvent::progress("test", "message");
        let cloned = event.clone();

        assert_eq!(event.lens(), cloned.lens());
    }

    #[test]
    fn test_event_debug() {
        let event = LensEvent::started("test", "task");
        let debug_str = format!("{:?}", event);

        assert!(debug_str.contains("Started"));
        assert!(debug_str.contains("test"));
    }
}
