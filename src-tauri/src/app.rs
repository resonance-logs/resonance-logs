use crate::app;
use flexi_logger::{FlexiLoggerError, LoggerHandle};
use log::{error, info};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};

fn get_app_state() -> &'static Mutex<AppState> {
    static APP_STATE: OnceLock<Mutex<AppState>> = OnceLock::new();
    APP_STATE.get_or_init(|| Mutex::new(AppState::new()))
}

struct AppState {
    pub logger_handle: Option<LoggerHandle>,
}

impl AppState {
    pub(crate) fn new() -> Self {
        Self {
            logger_handle: None,
        }
    }

    // todo: maybe extract this logging logic out into app/logger.rs or smth
    pub fn init_logger(&mut self, path: PathBuf) {
        if self.logger_handle.clone().is_some() {
            error!("AppState logger already inited");
            return;
        }

        let mut logger = flexi_logger::Logger::try_with_str("trace, tao=off").unwrap()
            // 1. File logger: writes everything (trace and above) into a specified folder
            .use_utc()
            .write_mode(flexi_logger::WriteMode::BufferAndFlush)
            // .write_mode(flexi_logger::WriteMode::Async) // todo: figure out how to do async logs
            .log_to_file(
                flexi_logger::FileSpec::default()
                    .basename("bpsr_logs")
                    .directory(path),
            )
            .rotate(
                flexi_logger::Criterion::Size(5_000_000),
                flexi_logger::Naming::Timestamps,
                flexi_logger::Cleanup::KeepLogFiles(2),
            );

        #[cfg(debug_assertions)]
        {
            // 2. Duplicate logs at INFO level and above to stdout
            logger = logger.duplicate_to_stdout(flexi_logger::Duplicate::Info);
        }

        self.logger_handle = Some(logger.start().unwrap());
    }

    pub fn get_logger(&self) -> Option<LoggerHandle> {
        self.logger_handle.clone()
    }
}

pub fn init_app(app_handle: AppHandle) {
    std::panic::set_hook(Box::new(|info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "non-string panic payload".to_string()
        };
        error!("Panicked: {:?}, location: {:?}", payload, info.location());

        get_logger().unwrap().flush();
    }));

    let local_app_data_path = app_handle.path().app_local_data_dir().expect("error getting app local data dir");
    get_app_state().lock().unwrap().init_logger(local_app_data_path); // init logging
}

pub fn get_logger() -> Result<LoggerHandle, String> {
    if let Some(logger_handle) = get_app_state().lock().unwrap().get_logger() {
        return Ok(logger_handle);
    }
    Err("AppState logger not present".to_string())
}
