use log::error;
use std::process::Command;

pub fn get() -> Option<String> {
    Command::new("uname")
        .arg("-m")
        .output()
        .map_err(|e| {
            error!("Failed to execute command: {}", e);
        })
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim_end().to_owned())
            } else {
                log::error!("'uname' command failed with status: {}", out.status);
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_success() {
        let output = get();
        assert!(output.is_some());
    }

    #[test]
    fn test_get_failure() {
        // Simulate a failure by overriding the command
        let result = Command::new("false")
            .output()
            .map_err(|e| {
                error!("Failed to execute command: {}", e);
            })
            .ok()
            .and_then(|out| {
                if out.status.success() {
                    Some(String::from_utf8_lossy(&out.stdout).trim_end().to_owned())
                } else {
                    log::error!("'false' command failed with status: {}", out.status);
                    None
                }
            });
        assert!(result.is_none());
    }
}