use std::collections::HashMap;
use std::path::PathBuf;

pub struct LoggingConfig {
    pub log_dir: PathBuf,
}

pub struct Config {
    pub unit_dirs: Vec<PathBuf>,
    pub notification_sockets_dir: PathBuf,
}

pub fn load_config() -> (LoggingConfig, Result<Config, String>) {
    let mut settings = HashMap::new();

    std::env::vars().for_each(|(key, value)| {
        let mut new_key: Vec<String> = key.split('_').map(|part| part.to_lowercase()).collect();
        //drop prefix
        if *new_key[0] == *"rustysd" {
            new_key.remove(0);
            let new_key = new_key.join(".");
            settings.insert(new_key, value);
        }
    });

    let log_dir = settings.get("logging.dir").map(|dir|  Some(PathBuf::from(dir)));

    let notification_sockets_dir = settings.get("notifications.dir").map(|dir|  Some(PathBuf::from(dir)));

    let unit_dirs = settings.get("notifications.dir").map(|dir|  vec![PathBuf::from(dir)]);

    let config = Config {
        unit_dirs: unit_dirs.unwrap_or_else(|| vec![PathBuf::from("./test_units")]),

        notification_sockets_dir: notification_sockets_dir
            .unwrap_or_else(|| Some(PathBuf::from("./notifications")))
            .unwrap_or_else(|| PathBuf::from("./notifications")),
    };

    let conf =Ok(config);

    (
        LoggingConfig {
            log_dir: log_dir
                .unwrap_or_else(|| Some(PathBuf::from("./logs")))
                .unwrap_or_else(|| PathBuf::from("./logs")),
        },
        conf,
    )
}

