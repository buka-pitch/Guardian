use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Severity levels for security events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Types of events the Guardian daemon can collect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventType {
    /// File system integrity events
    FileIntegrity {
        path: String,
        operation: FileOperation,
        hash: Option<String>,
    },
    /// Network socket events
    NetworkSocket {
        local_addr: String,
        remote_addr: Option<String>,
        protocol: String,
        state: String,
    },
    /// System log events
    SystemLog {
        source: String,
        level: String,
        message: String,
    },
    /// Process monitoring events
    ProcessMonitor {
        pid: u32,
        name: String,
        cpu_usage: f32,
        memory_usage: u64,
    },
}

/// File operations for integrity monitoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileOperation {
    Create,
    Modify,
    Delete,
    Rename,
    Chmod,
}

/// Core log event structure shared between daemon and frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    /// Unique identifier for this event
    pub id: Uuid,
    
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    
    /// Severity level of the event
    pub severity: Severity,
    
    /// The actual event data
    #[serde(flatten)]
    pub event_type: EventType,
    
    /// Hostname where the event originated
    pub hostname: String,
    
    /// Optional tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Whether this event triggered any rules
    #[serde(default)]
    pub rule_triggered: bool,
    
    /// Optional rule name that was triggered
    pub rule_name: Option<String>,
}

impl LogEvent {
    /// Create a new log event
    pub fn new(severity: Severity, event_type: EventType, hostname: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            severity,
            event_type,
            hostname,
            tags: Vec::new(),
            rule_triggered: false,
            rule_name: None,
        }
    }
    
    /// Add a tag to this event
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// Mark this event as having triggered a rule
    pub fn with_rule(mut self, rule_name: impl Into<String>) -> Self {
        self.rule_triggered = true;
        self.rule_name = Some(rule_name.into());
        self
    }
    
    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_event_serialization() {
        let event = LogEvent::new(
            Severity::High,
            EventType::FileIntegrity {
                path: "/etc/passwd".to_string(),
                operation: FileOperation::Modify,
                hash: Some("abc123".to_string()),
            },
            "localhost".to_string(),
        );

        let json = event.to_json().unwrap();
        let deserialized = LogEvent::from_json(&json).unwrap();

        assert_eq!(event.severity, deserialized.severity);
        assert_eq!(event.hostname, deserialized.hostname);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Info);
    }
}
