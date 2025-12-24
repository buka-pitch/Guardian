use guardian_common::{EventType, FileOperation, LogEvent, Severity};

/// Simple rule engine for evaluating events
pub struct RuleEngine {
    rules: Vec<Rule>,
}

/// A rule that can be evaluated against a LogEvent
struct Rule {
    name: String,
    matcher: Box<dyn Fn(&LogEvent) -> bool + Send + Sync>,
}

impl RuleEngine {
    /// Create a new rule engine with default rules
    pub fn new() -> Self {
        let mut engine = Self { rules: Vec::new() };
        engine.load_default_rules();
        engine
    }

    /// Load default security rules
    fn load_default_rules(&mut self) {
        // Rule 1: Critical file modifications
        self.add_rule(
            "critical_file_modification",
            Box::new(|event| {
                matches!(
                    &event.event_type,
                    EventType::FileIntegrity {
                        path,
                        operation: FileOperation::Modify | FileOperation::Delete,
                        ..
                    } if path.contains("/etc/passwd")
                        || path.contains("/etc/shadow")
                        || path.contains("/etc/sudoers")
                )
            }),
        );

        // Rule 2: High severity threshold
        self.add_rule(
            "high_severity_alert",
            Box::new(|event| event.severity >= Severity::High),
        );

        // Rule 3: Suspicious network activity
        self.add_rule(
            "suspicious_network",
            Box::new(|event| {
                matches!(
                    &event.event_type,
                    EventType::NetworkSocket { remote_addr, .. }
                    if remote_addr.as_ref().map_or(false, |addr| {
                        // Flag connections to non-standard ports
                        addr.contains(":4444") || addr.contains(":31337")
                    })
                )
            }),
        );

        // Rule 4: Excessive CPU usage
        self.add_rule(
            "high_cpu_usage",
            Box::new(|event| {
                matches!(
                    &event.event_type,
                    EventType::ProcessMonitor { cpu_usage, .. }
                    if *cpu_usage > 90.0
                )
            }),
        );
    }

    /// Add a custom rule
    pub fn add_rule(
        &mut self,
        name: impl Into<String>,
        matcher: Box<dyn Fn(&LogEvent) -> bool + Send + Sync>,
    ) {
        self.rules.push(Rule {
            name: name.into(),
            matcher,
        });
    }

    /// Evaluate an event against all rules
    /// Returns the name of the first matching rule, if any
    pub fn evaluate(&self, event: &LogEvent) -> Option<String> {
        for rule in &self.rules {
            if (rule.matcher)(event) {
                return Some(rule.name.clone());
            }
        }
        None
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use guardian_common::{EventType, FileOperation, Severity};

    #[test]
    fn test_critical_file_rule() {
        let engine = RuleEngine::new();

        let event = LogEvent::new(
            Severity::High,
            EventType::FileIntegrity {
                path: "/etc/passwd".to_string(),
                operation: FileOperation::Modify,
                hash: None,
            },
            "localhost".to_string(),
        );

        let result = engine.evaluate(&event);
        assert_eq!(result, Some("critical_file_modification".to_string()));
    }

    #[test]
    fn test_high_severity_rule() {
        let engine = RuleEngine::new();

        let event = LogEvent::new(
            Severity::Critical,
            EventType::SystemLog {
                source: "kernel".to_string(),
                level: "error".to_string(),
                message: "System panic".to_string(),
            },
            "localhost".to_string(),
        );

        let result = engine.evaluate(&event);
        assert!(result.is_some());
    }
}
