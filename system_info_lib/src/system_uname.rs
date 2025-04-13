//src/system_info.rs
use log::error;
use std::process::Command;

/// Executes the `uname` command with the given argument.
///
/// # Arguments
///
/// * `arg` - The argument to pass to `uname`.
///
/// # Returns
///
/// * `Option<String>` - The output of the `uname` command, if successful.
pub fn uname(arg: &str) -> Option<String> {
    Command::new("uname")
        .arg(arg)
        .output()
        .map_err(|e| {
            error!("Failed to invoke 'uname {}': {:?}", arg, e);
        })
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim_end().to_owned())
            } else {
                log::error!("'uname' invocation error: {:?}", out);
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that calling `uname` with an empty argument fails.
    #[test]
    fn test_uname() {
        assert!(uname("").is_none());
    }

    /// Tests that calling `uname` with a non-empty argument returns a non-empty result.
    ///
    /// This test ensures that when `uname` is called with the `-s` argument,
    /// it successfully returns a string that is not empty.
    #[test]
    fn uname_nonempty() {
        let val = uname("-s").expect("uname failed");
        assert!(!val.is_empty())
    }
}
