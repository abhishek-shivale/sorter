use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::config::Config;
use crate::error::SorterError;
use crate::{files, paths};

const POLL_INTERVAL: Duration = Duration::from_secs(60);

pub fn start() -> Result<(), SorterError> {
    let config = Config::load(&paths::config_path())?;

    let domain = launchd_domain()?;
    if is_loaded(&domain)? {
        return Err(SorterError::AlreadyRunning);
    }

    let dir = paths::app_dir();
    fs::create_dir_all(&dir)?;

    let plist_path = paths::launch_agent_path();
    if let Some(parent) = plist_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &plist_path,
        render_plist(
            &std::env::current_exe()?,
            &dir.join("sorter.log"),
            &dirs::home_dir().expect("Home directory not found"),
        ),
    )?;

    let status = Command::new("launchctl")
        .args(["bootstrap", &domain])
        .arg(&plist_path)
        .status()?;
    if !status.success() {
        let _ = fs::remove_file(&plist_path);
        return Err(SorterError::StartFailed);
    }

    println!("sorter started, watching {}", config.url);
    Ok(())
}

pub fn run() -> Result<(), SorterError> {
    loop {
        let result = Config::load(&paths::config_path())
            .and_then(|config| files::sort_dir(Path::new(&config.url), &config.ignore));
        if let Err(problem) = result {
            eprintln!("{problem}");
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

pub fn stop() -> Result<(), SorterError> {
    let domain = launchd_domain()?;
    let plist_path = paths::launch_agent_path();

    if !is_loaded(&domain)? {
        let _ = fs::remove_file(&plist_path);
        return Err(SorterError::NotRunning);
    }

    let status = Command::new("launchctl")
        .args(["bootout", &service_target(&domain)])
        .status()?;
    if !status.success() {
        return Err(SorterError::StopFailed);
    }

    let _ = fs::remove_file(&plist_path);
    println!("sorter stopped");
    Ok(())
}

fn launchd_domain() -> Result<String, SorterError> {
    let output = Command::new("id").arg("-u").output()?;
    Ok(format!("gui/{}", String::from_utf8_lossy(&output.stdout).trim()))
}

fn service_target(domain: &str) -> String {
    format!("{domain}/{}", paths::LAUNCH_LABEL)
}

fn is_loaded(domain: &str) -> Result<bool, SorterError> {
    let output = Command::new("launchctl")
        .args(["print", &service_target(domain)])
        .output()?;
    Ok(output.status.success())
}

fn render_plist(exe: &Path, log: &Path, home: &Path) -> String {
    let exe = xml_escape(exe);
    let log = xml_escape(log);
    let home = xml_escape(home);
    let label = paths::LAUNCH_LABEL;

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>run</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>HOME</key>
        <string>{home}</string>
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log}</string>
    <key>StandardErrorPath</key>
    <string>{log}</string>
</dict>
</plist>
"#
    )
}

fn xml_escape(path: &Path) -> String {
    path.to_string_lossy()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_runs_daemon_and_escapes_paths() {
        let plist = render_plist(
            Path::new("/Apps/R&D/sorter"),
            Path::new("/logs/sorter.log"),
            Path::new("/Users/me"),
        );

        assert!(plist.contains("<string>/Apps/R&amp;D/sorter</string>"));
        assert!(plist.contains("<string>run</string>"));
        assert!(plist.contains("<string>/Users/me</string>"));
        assert!(plist.contains("<key>RunAtLoad</key>\n    <true/>"));
        assert!(plist.contains("<key>KeepAlive</key>\n    <true/>"));
    }
}
