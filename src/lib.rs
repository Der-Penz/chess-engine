pub mod game;
pub mod uci;

#[cfg(feature = "log_to_file")]
use log::info;
use log::LevelFilter;
use std::io::Write;

pub fn init_logging() {
    #[cfg(not(feature = "log_to_file"))]
    {
        env_logger::builder()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .filter(None, LevelFilter::Info)
            .init();
    }

    #[cfg(feature = "log_to_file")]
    {
        let log_file_path = std::env::var("LOG_FILE").unwrap_or("logs.log".to_string());
        let log_file = std::fs::File::create(log_file_path).unwrap();
        env_logger::Builder::from_default_env()
            .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .filter(None, LevelFilter::Info)
            .init();
        info!("Log to file enabled");
    }
}
