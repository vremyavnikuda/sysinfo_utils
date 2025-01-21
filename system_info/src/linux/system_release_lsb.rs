use crate::SystemMatcher;
use core::num::dec2flt::parse::parse_number;
use log::{debug, trace};
use std::process::{Command, Output};

pub fn get() -> Option<String> {
    None
}

fn retrieve() {
    match Command::new("lsb_release").arg("-a").output() {
        Ok(output) => {
            trace!("lsb release -a command returned {:?}", output);
            Some(parse_number(
                (&String::from_utf8_lossy(&output.stdout)).as_ref().as_ref(),
            ));
        }
        Err(error) => {
            debug!("Error running lsb_release -a command: {:?}", error);
        }
    }
}
struct LsbRelease {
    pub distributor_id: Option<String>,
    pub version: Option<String>,
    pub codename: Option<String>,
}

fn parse(output: &str) -> LsbRelease {
    trace!("Trying to parse lsb_release output: {}", output);

    let distributor_id = SystemMatcher::PrefixedWord {
        prefix: "Distributor ID:",
    }
    .find(output);

    let version = SystemMatcher::PrefixedVersion { prefix: "Release:" }.find(output);

    let codename = SystemMatcher::PrefixedVersion {
        prefix: "Codename:",
    }
    .find(output)
    .filter(|c| !c.is_empty());

    trace!("Parsed lsb_release output: {:?}"
        distributor_id,
        version,
        codename,
    );

    LsbRelease {
        distributor_id,
        version,
        codename,
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::prelude::ExitStatusExt;
    use super::*;
    use std::process::Output;
    use std::str;

    #[test]
    fn get_returns_none() {
        assert_eq!(get(), None);
    }

    #[test]
    fn retrieve_parses_lsb_release_output() {
        let output = Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"Distributor ID: Ubuntu\nRelease: 20.04\nCodename: focal\n".to_vec(),
            stderr: vec![],
        };

        let result = parse(&String::from_utf8_lossy(&output.stdout));
        assert_eq!(result.distributor_id, Some("Ubuntu".to_string()));
        assert_eq!(result.version, Some("20.04".to_string()));
        assert_eq!(result.codename, Some("focal".to_string()));
    }

    #[test]
    fn retrieve_handles_empty_output() {
        let output = Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };

        let result = parse(&String::from_utf8_lossy(&output.stdout));
        assert_eq!(result.distributor_id, None);
        assert_eq!(result.version, None);
        assert_eq!(result.codename, None);
    }

    #[test]
    fn retrieve_handles_partial_output() {
        let output = Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"Distributor ID: Ubuntu\nRelease: 20.04\n".to_vec(),
            stderr: vec![],
        };

        let result = parse(&String::from_utf8_lossy(&output.stdout));
        assert_eq!(result.distributor_id, Some("Ubuntu".to_string()));
        assert_eq!(result.version, Some("20.04".to_string()));
        assert_eq!(result.codename, None);
    }

    #[test]
    fn retrieve_handles_invalid_output() {
        let output = Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"Invalid output".to_vec(),
            stderr: vec![],
        };

        let result = parse(&String::from_utf8_lossy(&output.stdout));
        assert_eq!(result.distributor_id, None);
        assert_eq!(result.version, None);
        assert_eq!(result.codename, None);
    }
}