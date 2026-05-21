pub mod renderer;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ProviderState {
    pub status: String,
    pub time: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessState {
    pub query: String,
    pub resolved_location: Option<String>,
    pub cache_status: String,
    pub global_progress: u32,
    pub step_parsing: String,
    pub step_geocoding: String,
    pub step_race: String,
    pub step_blending: String,
    pub providers: HashMap<String, ProviderState>,
    pub last_log: String,
}

impl ProcessState {
    pub fn new(query: String) -> Self {
        let mut providers = HashMap::new();
        providers.insert("Open-Meteo".to_string(), ProviderState { status: "pending".to_string(), time: None, error: None });
        providers.insert("MET Norway".to_string(), ProviderState { status: "pending".to_string(), time: None, error: None });
        providers.insert("wttr.in".to_string(), ProviderState { status: "pending".to_string(), time: None, error: None });

        Self {
            query,
            resolved_location: None,
            cache_status: "searching".to_string(),
            global_progress: 0,
            step_parsing: "completed".to_string(),
            step_geocoding: "pending".to_string(),
            step_race: "pending".to_string(),
            step_blending: "pending".to_string(),
            providers,
            last_log: "Süreç başlatılıyor...".to_string(),
        }
    }
}

pub type SharedState = Arc<Mutex<ProcessState>>;

pub async fn conditional_sleep(state: Option<&SharedState>, ms: u64) {
    if state.is_some() {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    }
}

pub struct TuiLoggingLayer {
    pub state: SharedState,
}

impl<S> tracing_subscriber::Layer<S> for TuiLoggingLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        struct MsgVisitor {
            msg: String,
        }
        impl tracing::field::Visit for MsgVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.msg = format!("{:?}", value);
                    if self.msg.starts_with('"') && self.msg.ends_with('"') {
                        self.msg = self.msg[1..self.msg.len() - 1].to_string();
                    }
                }
            }
        }

        let mut visitor = MsgVisitor { msg: String::new() };
        event.record(&mut visitor);

        if !visitor.msg.is_empty() {
            if let Ok(mut guard) = self.state.lock() {
                guard.last_log = visitor.msg;
            }
        }
    }
}

