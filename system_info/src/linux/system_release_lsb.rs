use crate::{system_os::Type, Info, SystemMatcher, SystemVersion};
use log::{debug, trace};
use std::process::Command;

pub fn get() -> Option<Info> {
    let release = retrieve()?;
    let version = match release.version.as_deref() {
        Some("rolling") => SystemVersion::Rolling(None),
        Some(version) => SystemVersion::from_string(version.to_owned()),
        None => SystemVersion::Unknown,
    };


    let system_type = match release.distributor_id.as_ref().map(String::as_ref){
        Some("Alpaquita") => Type::Alpaquita,
        Some("Amazon") | Some("AmazonAMI") => Type::Amazon,
        Some("Arch") => Type::Arch,
        Some("Artix") => Type::Artix,
        Some("cachyos") => Type::CachyOS,
        Some("CentOS") => Type::CentOS,
        Some("Debian") => Type::Debian,
        Some("EndeavourOS") => Type::EndeavourOS,
        Some("Fedora") | Some("Fedora Linux") => Type::Fedora,
        Some("Garuda") => Type::Garuda,
        Some("Gentoo") => Type::Gentoo,
        Some("Kali") => Type::Kali,
        Some("Linuxmint") => Type::Mint,
        Some("MaboxLinux") => Type::Mabox,
        Some("ManjaroLinux") => Type::Manjaro,
        Some("Mariner") => Type::Mariner,
        Some("NixOS") => Type::NixOS,
        Some("NobaraLinux") => Type::Nobara,
        Some("Uos") => Type::Uos,
        Some("OpenCloudOS") => Type::OpenCloudOS,
        Some("openEuler") => Type::OpenEuler,
        Some("openSUSE") => Type::OpenSUSE,
        Some("OracleServer") => Type::OracleLinux,
        Some("Pop") => Type::Pop,
        Some("Raspbian") => Type::Raspbian,
        Some("RedHatEnterprise") | Some("RedHatEnterpriseServer") => Type::RedHatEnterprise,
        Some("Solus") => Type::Solus,
        Some("SUSE") => Type::SUSE,
        Some("Ubuntu") => Type::Ubuntu,
        Some("UltramarineLinux") => Type::Ultramarine,
        Some("VoidLinux") => Type::Void,
        _ => Type::Linux,
    };

    Some(Info{
        system_type,
        version,
        ..Default::default()
    })
}

fn retrieve() -> Option<LsbRelease>{
    match Command::new ("system_lsb_release").arg("-a").output(){
        Ok(output)=>{
            trace!("system_lsb_release command returned: {:?}",output);
            Some(parse(&String::from_utf8_lossy(&output.stdout)))
        }
        Err(error)=>{
            debug!("Failed to execute system_lsb_release command: {:?}",error);
            None
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

    trace!("Parsed lsb_release output: {:?} {:?}",
        distributor_id,
        version,
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