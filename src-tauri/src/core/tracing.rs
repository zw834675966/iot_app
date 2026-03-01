use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

use crate::core::config::RuntimeConfig;
use crate::core::error::{AppError, AppResult};

static TRACING_GUARDS: OnceLock<TracingGuards> = OnceLock::new();
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct TraceContext {
    pub request_id: Option<String>,
}

struct TracingGuards {
    _console: WorkerGuard,
    _file: WorkerGuard,
}

pub fn init_tracing(runtime_config: &RuntimeConfig) -> Result<(), String> {
    if TRACING_GUARDS.get().is_some() {
        return Ok(());
    }

    let log_dir = resolve_log_directory(&runtime_config.logging.directory);
    std::fs::create_dir_all(&log_dir)
        .map_err(|err| format!("create log directory failed ({}): {err}", log_dir.display()))?;

    let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("pure-admin-thin")
        .filename_suffix("log")
        .build(&log_dir)
        .map_err(|err| format!("build rolling log appender failed: {err}"))?;
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);
    let (console_writer, console_guard) = tracing_appender::non_blocking(std::io::stdout());

    let env_filter = build_env_filter(&runtime_config.logging.level)?;
    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(console_writer)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(file_writer)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_current_span(true)
        .with_span_list(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    if let Err(err) = tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .try_init()
    {
        let already_initialized = err
            .to_string()
            .contains("a global default trace dispatcher has already been set");
        if already_initialized {
            return Ok(());
        }
        return Err(format!("initialize tracing subscriber failed: {err}"));
    }

    let _ = TRACING_GUARDS.set(TracingGuards {
        _console: console_guard,
        _file: file_guard,
    });

    tracing::info!(
        level = %runtime_config.logging.level,
        directory = %log_dir.display(),
        "tracing initialized"
    );
    Ok(())
}

pub fn execute_traced_command<T>(
    command: &'static str,
    trace: Option<TraceContext>,
    handler: impl FnOnce() -> AppResult<T>,
) -> AppResult<T> {
    let request_id = resolve_request_id(trace.as_ref());
    let span = tracing::info_span!("tauri_request", command, request_id = %request_id);
    let _span_guard = span.enter();

    tracing::info!("request started");
    let result = handler();
    match &result {
        Ok(_) => tracing::info!("request completed"),
        Err(AppError::Validation(message)) => {
            tracing::warn!(error = %message, "request failed");
        }
        Err(AppError::Database(message)) => {
            tracing::error!(error = %message, "request failed");
        }
    }

    result
}

fn resolve_request_id(trace: Option<&TraceContext>) -> String {
    if let Some(request_id) = trace
        .and_then(|context| context.request_id.as_ref())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        return request_id.to_string();
    }

    REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed).to_string()
}

fn build_env_filter(level: &str) -> Result<EnvFilter, String> {
    if let Ok(filter_from_env) = EnvFilter::try_from_default_env() {
        return Ok(filter_from_env);
    }

    parse_env_filter(level)
}

fn parse_env_filter(level: &str) -> Result<EnvFilter, String> {
    EnvFilter::try_new(level).map_err(|err| format!("invalid logging.level '{}': {err}", level))
}

fn resolve_log_directory(directory: &str) -> PathBuf {
    PathBuf::from(directory)
}

#[cfg(test)]
mod tests {
    use super::{TraceContext, parse_env_filter, resolve_request_id};

    #[test]
    fn accepts_valid_env_filter_level() {
        let _filter = parse_env_filter("info").expect("valid filter");
    }

    #[test]
    fn rejects_invalid_env_filter_level() {
        let err = parse_env_filter("!!!invalid!!!").expect_err("invalid filter should fail");
        assert!(err.contains("invalid logging.level"));
    }

    #[test]
    fn uses_frontend_request_id_when_present() {
        let trace = TraceContext {
            request_id: Some("fe-abc-001".to_string()),
        };
        let request_id = resolve_request_id(Some(&trace));
        assert_eq!(request_id, "fe-abc-001");
    }

    #[test]
    fn generates_request_id_when_frontend_value_absent() {
        let request_id = resolve_request_id(None);
        assert!(!request_id.is_empty());
    }
}
