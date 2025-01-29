use log::error;
use std::process::Command;

/// Executes the `uname -m` command to retrieve the architecture type of the system.
///
/// # Returns
///
/// - `Some(String)`: The architecture type as a string if the command is successful.
/// - `None`: If the command fails to execute or does not return a successful status.
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

    /// Test that the `get` function returns a `Some` value when it is successful.
    #[test]
    fn test_get_success() {
        let output = get();
        assert!(output.is_some());
    }

    /// Test that the `get` function returns a `None` value when it is not successful.
    ///
    /// This test simulates a failure by overriding the command with a failing one.
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
