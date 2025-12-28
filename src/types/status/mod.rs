//! Status enums for streaming and cancellation states.
//!
//! These enums replace `Option<bool>` fields to make invalid states
//! unrepresentable at compile time. They support multiple serialization
//! formats for API compatibility.

use serde::{Deserialize, Serialize};


/// Status of streaming operations.
///
/// Replaces `Option<bool>` to make invalid states unrepresentable.
/// Supports multiple serialization formats for API compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StreamingStatus {
    /// Streaming has not started
    #[default]
    NotStarted,
    /// Streaming is currently in progress
    InProgress,
    /// Streaming has completed successfully
    Complete,
}

impl StreamingStatus {
    /// Returns true if streaming is currently active or was active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, StreamingStatus::InProgress | StreamingStatus::Complete)
    }

    /// Returns true if streaming is complete (either success or not started).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        matches!(self, StreamingStatus::Complete)
    }

    /// Creates StreamingStatus from boolean (legacy API compatibility).
    ///
    /// - `true` → Complete
    /// - `false` → NotStarted
    pub fn from_bool(is_streaming: bool) -> Self {
        if is_streaming {
            StreamingStatus::Complete
        } else {
            StreamingStatus::NotStarted
        }
    }

    /// Converts to boolean (legacy API compatibility).
    ///
    /// - NotStarted → false
    /// - InProgress → true
    /// - Complete → false (streaming finished)
    #[must_use]
    pub fn as_bool(&self) -> bool {
        matches!(self, StreamingStatus::InProgress)
    }
}



impl std::fmt::Display for StreamingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamingStatus::NotStarted => write!(f, "not_started"),
            StreamingStatus::InProgress => write!(f, "in_progress"),
            StreamingStatus::Complete => write!(f, "complete"),
        }
    }
}

/// Custom deserializer for StreamingStatus.
///
/// Handles multiple API formats:
/// - Boolean: true/false
/// - Integer: 0, 1, 2
/// - String: "not_started", "in_progress", "complete"
impl<'de> Deserialize<'de> for StreamingStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StreamingStatusVisitor;

        impl<'de> serde::de::Visitor<'de> for StreamingStatusVisitor {
            type Value = StreamingStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a boolean, integer, or string representing streaming status")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StreamingStatus::from_bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    0 => Ok(StreamingStatus::NotStarted),
                    1 => Ok(StreamingStatus::InProgress),
                    2 => Ok(StreamingStatus::Complete),
                    _ => Err(serde::de::Error::custom(format!(
                        "invalid streaming status: {} (expected 0, 1, or 2)",
                        value
                    ))),
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_i64(value as i64)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value.to_lowercase().as_str() {
                    "notstarted" | "not_started" => Ok(StreamingStatus::NotStarted),
                    "inprogress" | "in_progress" => Ok(StreamingStatus::InProgress),
                    "complete" => Ok(StreamingStatus::Complete),
                    _ => Err(serde::de::Error::custom(format!(
                        "invalid streaming status: {}",
                        value
                    ))),
                }
            }
        }

        deserializer.deserialize_any(StreamingStatusVisitor)
    }
}

/// Status of cancellation operations.
///
/// Replaces `Option<bool>` to make invalid states unrepresentable.
/// Supports multiple serialization formats for API compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CancellationStatus {
    /// No cancellation requested
    #[default]
    NotCancelled,
    /// Cancellation has been requested but not yet processed
    Requested,
    /// Cancellation completed successfully
    Completed,
    /// Cancellation failed (e.g., operation already finished)
    Failed,
}

impl CancellationStatus {
    /// Returns true if cancellation was requested or completed.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        matches!(
            self,
            CancellationStatus::Requested | CancellationStatus::Completed
        )
    }

    /// Returns true if cancellation is final (completed or failed).
    #[must_use]
    pub fn is_final(&self) -> bool {
        matches!(self, CancellationStatus::Completed | CancellationStatus::Failed)
    }

    /// Creates CancellationStatus from boolean (legacy API compatibility).
    ///
    /// - `true` → Requested
    /// - `false` → NotCancelled
    pub fn from_bool(is_cancelled: bool) -> Self {
        if is_cancelled {
            CancellationStatus::Requested
        } else {
            CancellationStatus::NotCancelled
        }
    }

    /// Converts to boolean (legacy API compatibility).
    ///
    /// - NotCancelled → false
    /// - Requested → true
    /// - Completed → true
    /// - Failed → false
    #[must_use]
    pub fn as_bool(&self) -> bool {
        matches!(
            self,
            CancellationStatus::Requested | CancellationStatus::Completed
        )
    }
}



impl std::fmt::Display for CancellationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CancellationStatus::NotCancelled => write!(f, "not_cancelled"),
            CancellationStatus::Requested => write!(f, "requested"),
            CancellationStatus::Completed => write!(f, "completed"),
            CancellationStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Custom deserializer for CancellationStatus.
///
/// Handles multiple API formats:
/// - Boolean: true/false
/// - Integer: 0, 1, 2, 3
/// - String: "not_cancelled", "requested", "completed", "failed"
impl<'de> Deserialize<'de> for CancellationStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CancellationStatusVisitor;

        impl<'de> serde::de::Visitor<'de> for CancellationStatusVisitor {
            type Value = CancellationStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "a boolean, integer, or string representing cancellation status",
                )
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(CancellationStatus::from_bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    0 => Ok(CancellationStatus::NotCancelled),
                    1 => Ok(CancellationStatus::Requested),
                    2 => Ok(CancellationStatus::Completed),
                    3 => Ok(CancellationStatus::Failed),
                    _ => Err(serde::de::Error::custom(format!(
                        "invalid cancellation status: {} (expected 0, 1, 2, or 3)",
                        value
                    ))),
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_i64(value as i64)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value.to_lowercase().as_str() {
                    "notcancelled" | "not_cancelled" => Ok(CancellationStatus::NotCancelled),
                    "requested" => Ok(CancellationStatus::Requested),
                    "completed" => Ok(CancellationStatus::Completed),
                    "failed" => Ok(CancellationStatus::Failed),
                    _ => Err(serde::de::Error::custom(format!(
                        "invalid cancellation status: {}",
                        value
                    ))),
                }
            }
        }

        deserializer.deserialize_any(CancellationStatusVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    // StreamingStatus tests
    #[test]
    fn test_streaming_status_from_bool() {
        assert_eq!(
            StreamingStatus::from_bool(true),
            StreamingStatus::Complete
        );
        assert_eq!(
            StreamingStatus::from_bool(false),
            StreamingStatus::NotStarted
        );
    }

    #[test]
    fn test_streaming_status_as_bool() {
        assert!(StreamingStatus::InProgress.as_bool());
        assert!(!StreamingStatus::NotStarted.as_bool());
        assert!(!StreamingStatus::Complete.as_bool());
    }

    #[test]
    fn test_streaming_status_is_active() {
        assert!(StreamingStatus::InProgress.is_active());
        assert!(StreamingStatus::Complete.is_active());
        assert!(!StreamingStatus::NotStarted.is_active());
    }

    #[test]
    fn test_streaming_status_is_complete() {
        assert!(StreamingStatus::Complete.is_complete());
        assert!(!StreamingStatus::InProgress.is_complete());
        assert!(!StreamingStatus::NotStarted.is_complete());
    }

    #[test]
    fn test_streaming_status_serialization() {
        let status = StreamingStatus::Complete;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json, "complete");
    }

    #[test]
    fn test_streaming_status_serialization_result() {
        let status = StreamingStatus::Complete;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json.as_str().unwrap(), "complete");
    }

    #[test]
    fn test_streaming_status_deserialization_bool() {
        let json = "true";
        let status: StreamingStatus = from_str(json).unwrap();
        assert_eq!(status, StreamingStatus::Complete);
    }

    #[test]
    fn test_streaming_status_deserialization_int() {
        let json = "1";
        let status: StreamingStatus = from_str(json).unwrap();
        assert_eq!(status, StreamingStatus::InProgress);

        let json = "0";
        let status: StreamingStatus = from_str(json).unwrap();
        assert_eq!(status, StreamingStatus::NotStarted);
    }

    #[test]
    fn test_streaming_status_deserialization_string() {
        let json = "\"in_progress\"";
        let status: StreamingStatus = from_str(json).unwrap();
        assert_eq!(status, StreamingStatus::InProgress);
    }

    #[test]
    fn test_streaming_status_roundtrip() {
        let original = StreamingStatus::InProgress;
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: StreamingStatus = from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }


    #[test]
    fn test_streaming_status_default() {
        assert_eq!(StreamingStatus::default(), StreamingStatus::NotStarted);
    }

    #[test]
    fn test_streaming_status_display() {
        assert_eq!(format!("{}", StreamingStatus::NotStarted), "not_started");
        assert_eq!(format!("{}", StreamingStatus::InProgress), "in_progress");
        assert_eq!(format!("{}", StreamingStatus::Complete), "complete");
    }

    // CancellationStatus tests
    #[test]
    fn test_cancellation_status_from_bool() {
        assert_eq!(
            CancellationStatus::from_bool(true),
            CancellationStatus::Requested
        );
        assert_eq!(
            CancellationStatus::from_bool(false),
            CancellationStatus::NotCancelled
        );
    }

    #[test]
    fn test_cancellation_status_as_bool() {
        assert!(CancellationStatus::Requested.as_bool());
        assert!(CancellationStatus::Completed.as_bool());
        assert!(!CancellationStatus::NotCancelled.as_bool());
        assert!(!CancellationStatus::Failed.as_bool());
    }

    #[test]
    fn test_cancellation_status_is_cancelled() {
        assert!(CancellationStatus::Requested.is_cancelled());
        assert!(CancellationStatus::Completed.is_cancelled());
        assert!(!CancellationStatus::NotCancelled.is_cancelled());
        assert!(!CancellationStatus::Failed.is_cancelled());
    }

    #[test]
    fn test_cancellation_status_is_final() {
        assert!(CancellationStatus::Completed.is_final());
        assert!(CancellationStatus::Failed.is_final());
        assert!(!CancellationStatus::NotCancelled.is_final());
        assert!(!CancellationStatus::Requested.is_final());
    }

    #[test]
    fn test_cancellation_status_serialization() {
        let status = CancellationStatus::Completed;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json, "completed");
    }

    #[test]
    fn test_cancellation_status_serialization_result() {
        let status = CancellationStatus::Completed;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json.as_str().unwrap(), "completed");
    }

    #[test]
    fn test_cancellation_status_deserialization_bool() {
        let json = "true";
        let status: CancellationStatus = from_str(json).unwrap();
        assert_eq!(status, CancellationStatus::Requested);
    }

    #[test]
    fn test_cancellation_status_deserialization_int() {
        let json = "1";
        let status: CancellationStatus = from_str(json).unwrap();
        assert_eq!(status, CancellationStatus::Requested);

        let json = "0";
        let status: CancellationStatus = from_str(json).unwrap();
        assert_eq!(status, CancellationStatus::NotCancelled);
    }

    #[test]
    fn test_cancellation_status_deserialization_string() {
        let json = "\"completed\"";
        let status: CancellationStatus = from_str(json).unwrap();
        assert_eq!(status, CancellationStatus::Completed);
    }

    #[test]
    fn test_cancellation_status_roundtrip() {
        let original = CancellationStatus::Requested;
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: CancellationStatus = from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }


    #[test]
    fn test_cancellation_status_default() {
        assert_eq!(
            CancellationStatus::default(),
            CancellationStatus::NotCancelled
        );
    }

    #[test]
    fn test_cancellation_status_display() {
        assert_eq!(
            format!("{}", CancellationStatus::NotCancelled),
            "not_cancelled"
        );
        assert_eq!(format!("{}", CancellationStatus::Requested), "requested");
        assert_eq!(format!("{}", CancellationStatus::Completed), "completed");
        assert_eq!(format!("{}", CancellationStatus::Failed), "failed");
    }
}
