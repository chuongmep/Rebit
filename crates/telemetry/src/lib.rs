//! telemetry — observability, metrics, and crash reporting.
#![forbid(unsafe_code)]
#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    pub name: String,
    pub properties: Vec<(String, String)>,
}
#[derive(Debug, Default)]
pub struct TelemetrySession {
    events: Vec<TelemetryEvent>,
}
impl TelemetrySession {
    pub fn new() -> Self {
        Self { events: vec![] }
    }
    pub fn record(&mut self, event: TelemetryEvent) {
        self.events.push(event);
    }
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn record_increases_count() {
        let mut s = TelemetrySession::new();
        s.record(TelemetryEvent {
            name: "startup".into(),
            properties: vec![],
        });
        assert_eq!(s.event_count(), 1);
    }
}
