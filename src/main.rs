use std::{
    fs::{self, File},
    io::Read,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};
use std::{io::Write, time::Duration};

use serde::{Deserialize, Serialize};
// use std::{fs::File, io::Read};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestConfig {
    pub target_file: String,
}

impl TestConfig {
    pub fn load_from_file(file_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: TestConfig = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_level = match std::env::var("WORKFLOW_LOG") {
        Ok(level) => level,
        Err(_) => "info".to_string(),
    };

    env_logger::Builder::new().parse_filters(&log_level).init();

    let running = Arc::new(AtomicBool::new(true));
    let sleeping = Arc::new(AtomicBool::new(false));

    let r = running.clone();
    let s = sleeping.clone();
    ctrlc::set_handler(move || {
        log::info!("Received Ctrl-C from console.");
        r.store(false, Ordering::SeqCst);
        if s.load(Ordering::SeqCst) {
            log::warn!("Quit from sleeping...");
            process::exit(0);
        }
    })
    .expect("Error setting Ctrl-C handler");
    let config_file = std::env::var("CONFIG_FILE").expect("CONFIG_FILE must be set");
    let config = TestConfig::load_from_file(&config_file)?;
    let file_name = config.target_file;
    log::info!("task started with file:{}", file_name);
    let task = tokio::spawn(async move {
        match repeat_test(&file_name, running, sleeping).await {
            Ok(()) => println!("Test passed"),
            Err(e) => println!("Test failed: {}", e),
        }
    });

    let _ = task.await;
    Ok(())
}

async fn repeat_test(
    file_name: &str,
    running: Arc<AtomicBool>,
    sleeping: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file_path = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file_name)?;

    while running.load(Ordering::SeqCst) {
        let now = SystemTime::now();
        let body = reqwest::get("https://www.rust-lang.org")
            .await?
            .text()
            .await?;

        let elapsed = now.elapsed()?;
        writeln!(file_path, "{},{}", elapsed.as_millis(), body.len())?;
        if !running.load(Ordering::SeqCst) {
            break;
        }
        sleeping.store(true, Ordering::SeqCst);
        log::info!("Sleeping...");
        tokio::time::sleep(Duration::from_secs(30)).await;
        sleeping.store(false, Ordering::SeqCst);
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn it_works() {
        let req = reqwest::get("https://www.rust-lang.org");
        assert!(aw!(req).is_ok());
    }
}
