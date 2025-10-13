//src/linux/system_file_release.rs
use crate::system_matcher::SystemMatcher;
use crate::system_os::Type;
use crate::{BitDepth, Info, SystemVersion};
use log::{trace, warn};
use std::fs::File;
use std::io::Read;
use std::path::Path;

//TODO: add more distributions
pub fn get() -> Option<Info> {
    retrieve_release_info(&DISTRIBUTIONS, "/")
}

/// Iterate over the given list of `ReleaseInfo` and check if the `path`
/// exists. If it does, read the file and call the `system` and `version`
/// closures to retrieve the system type and version. If they are defined,
/// return an `Info` instance with the given system type and version.
/// Otherwise, return `None`.
///
/// The `root` parameter is prepended to the `path` of each `ReleaseInfo`
/// item.
///
/// If any error occurs while opening or reading the file, log a warning
/// and continue with the next item.
///
/// If no item matches, return `None`.
fn retrieve_release_info(distributions: &[ReleaseInfo], root: &str) -> Option<Info> {
    for release_info in distributions {
        let path = Path::new(root).join(release_info.path);

        if !path.exists() {
            trace!("Path '{}' doesn't exist", release_info.path);
            continue;
        }

        let mut file = match File::open(&path) {
            Ok(value) => value,
            Err(error) => {
                warn!("Unable to open {:?} file: {:?}", &path, error);
                continue;
            }
        };

        let mut file_content = String::new();
        if let Err(error) = file.read_to_string(&mut file_content) {
            warn!("Unable to read {:?} file: {:?}", &path, error);
            continue;
        }

        let system_type = (release_info.type_var)(&file_content);

        let Some(system_type) = system_type else {
            continue;
        };

        let version = (release_info.version)(&file_content);

        return Some(Info {
            system_type,
            version: version.unwrap_or(SystemVersion::Unknown),
            bit_depth: BitDepth::Unknown,
            ..Default::default()
        });
    }
    None
}

#[derive(Clone)]
struct ReleaseInfo<'a> {
    path: &'a str,

    type_var: for<'b> fn(&'b str) -> Option<Type>,

    version: for<'b> fn(&'b str) -> Option<SystemVersion>,
}

static DISTRIBUTIONS: [ReleaseInfo; 6] = [
    // Keep this first; most modern distributions have this file.
    ReleaseInfo {
        path: "etc/os-release",
        type_var: |release| {
            SystemMatcher::KeyValue { key: "ID" }
                .find(release)
                .and_then(|id| match id.as_str() {
                    // os-release information collected from
                    // https://github.com/chef/os_release
                    "almalinux" => Some(Type::AlmaLinux),
                    "alpaquita" => Some(Type::Alpaquita),
                    "alpine" => Some(Type::Alpine),
                    "amzn" => Some(Type::Amazon),
                    //"antergos" => Antergos
                    //"aosc" => AOSC
                    "arch" => Some(Type::Arch),
                    "archarm" => Some(Type::Arch),
                    "artix" => Some(Type::Artix),
                    "cachyos" => Some(Type::CachyOS),
                    "centos" => Some(Type::CentOS),
                    //"clear-linux-os" => ClearLinuxOS
                    //"clearos" => ClearOS
                    //"coreos"
                    //"cumulus-linux" => Cumulus
                    "debian" => Some(Type::Debian),
                    //"devuan" => Devuan
                    //"elementary" => Elementary
                    "fedora" => Some(Type::Fedora),
                    //"gentoo" => Gentoo
                    //"ios_xr" => ios_xr
                    "kali" => Some(Type::Kali),
                    //"mageia" => Mageia
                    //"manjaro" => Manjaro
                    "linuxmint" => Some(Type::Mint),
                    "mariner" => Some(Type::Mariner),
                    //"nexus" => Nexus
                    "nixos" => Some(Type::NixOS),
                    "nobara" => Some(Type::Nobara),
                    "Uos" => Some(Type::Uos),
                    "opencloudos" => Some(Type::OpenCloudOS),
                    "openEuler" => Some(Type::openEuler),
                    "ol" => Some(Type::OracleLinux),
                    "opensuse" => Some(Type::openSUSE),
                    "opensuse-leap" => Some(Type::openSUSE),
                    "opensuse-microos" => Some(Type::openSUSE),
                    "opensuse-tumbleweed" => Some(Type::openSUSE),
                    //"rancheros" => RancherOS
                    //"raspbian" => Raspbian
                    // note XBian also uses "raspbian"
                    "rhel" => Some(Type::RedHatEnterprise),
                    "rocky" => Some(Type::RockyLinux),
                    //"sabayon" => Sabayon
                    //"scientific" => Scientific
                    //"slackware" => Slackware
                    "sled" => Some(Type::SUSE), // SUSE desktop
                    "sles" => Some(Type::SUSE),
                    "sles_sap" => Some(Type::SUSE), // SUSE SAP
                    "ubuntu" => Some(Type::Ubuntu),
                    "ultramarine" => Some(Type::Ultramarine),
                    //"virtuozzo" => Virtuozzo
                    "void" => Some(Type::Void),
                    //"XCP-ng" => xcp-ng
                    //"xenenterprise" => xcp-ng
                    //"xenserver" => xcp-ng
                    _ => None,
                })
        },
        version: |release| {
            SystemMatcher::KeyValue { key: "VERSION_ID" }
                .find(release)
                .map(SystemVersion::from_string)
        },
    },
    // Older distributions must have their specific release file parsed.
    ReleaseInfo {
        path: "etc/mariner-release",
        type_var: |_| Some(Type::Mariner),
        version: |release| {
            SystemMatcher::PrefixedVersion {
                prefix: "CBL-Mariner",
            }
            .find(release)
            .map(SystemVersion::from_string)
        },
    },
    ReleaseInfo {
        path: "etc/centos-release",
        type_var: |_| Some(Type::CentOS),
        version: |release| {
            SystemMatcher::PrefixedVersion { prefix: "release" }
                .find(release)
                .map(SystemVersion::from_string)
        },
    },
    ReleaseInfo {
        path: "etc/fedora-release",
        type_var: |_| Some(Type::Fedora),
        version: |release| {
            SystemMatcher::PrefixedVersion { prefix: "release" }
                .find(release)
                .map(SystemVersion::from_string)
        },
    },
    ReleaseInfo {
        path: "etc/alpine-release",
        type_var: |_| Some(Type::Alpine),
        version: |release| {
            SystemMatcher::AllTrimmed
                .find(release)
                .map(SystemVersion::from_string)
        },
    },
    ReleaseInfo {
        path: "etc/redhat-release",
        type_var: |_| Some(Type::RedHatEnterprise),
        version: |release| {
            SystemMatcher::PrefixedVersion { prefix: "release" }
                .find(release)
                .map(SystemVersion::from_string)
        },
    },
];

#[cfg(test)]
mod system_file_release_test {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn retrieve_release_info_finds_valid_distribution() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("os-release");
        let mut file = fs::File::create(&temp_path).unwrap();
        writeln!(
            file,
            "ID=ubuntu\nVERSION_ID=\"20.04\"\nPRETTY_NAME=\"Ubuntu 20.04 LTS\""
        )
        .unwrap();

        let custom_distributions = [ReleaseInfo {
            path: "os-release",
            type_var: |content| {
                SystemMatcher::KeyValue { key: "ID" }
                    .find(content)
                    .and_then(|id| match id.as_str() {
                        "ubuntu" => Some(Type::Ubuntu),
                        _ => None,
                    })
            },
            version: |content| {
                SystemMatcher::KeyValue { key: "VERSION_ID" }
                    .find(content)
                    .map(SystemVersion::from_string)
            },
        }];

        let result =
            retrieve_release_info(&custom_distributions, temp_dir.path().to_str().unwrap());
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.system_type, Type::Ubuntu);
        assert_eq!(
            info.version,
            SystemVersion::from_string("20.04".to_string())
        );
    }

    #[test]
    fn retrieve_release_info_handles_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        let custom_distributions = [ReleaseInfo {
            path: "os-release",
            type_var: |_| Some(Type::Ubuntu),
            version: |_| Some(SystemVersion::from_string("20.04".to_string())),
        }];

        let result =
            retrieve_release_info(&custom_distributions, temp_dir.path().to_str().unwrap());
        assert!(result.is_none());
    }

    #[test]
    fn retrieve_release_info_handles_invalid_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("os-release");
        let mut file = fs::File::create(&temp_path).unwrap();
        writeln!(file, "INVALID_CONTENT").unwrap();

        let custom_distributions = [ReleaseInfo {
            path: "os-release",
            type_var: |content| {
                SystemMatcher::KeyValue { key: "ID" }
                    .find(content)
                    .and_then(|id| match id.as_str() {
                        "ubuntu" => Some(Type::Ubuntu),
                        _ => None,
                    })
            },
            version: |content| {
                SystemMatcher::KeyValue { key: "VERSION_ID" }
                    .find(content)
                    .map(SystemVersion::from_string)
            },
        }];

        let result =
            retrieve_release_info(&custom_distributions, temp_dir.path().to_str().unwrap());
        assert!(result.is_none());
    }

    #[test]
    fn retrieve_release_info_handles_partial_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("os-release");
        let mut file = fs::File::create(&temp_path).unwrap();
        writeln!(file, "ID=ubuntu").unwrap();

        let custom_distributions = [ReleaseInfo {
            path: "os-release",
            type_var: |content| {
                SystemMatcher::KeyValue { key: "ID" }
                    .find(content)
                    .and_then(|id| match id.as_str() {
                        "ubuntu" => Some(Type::Ubuntu),
                        _ => None,
                    })
            },
            version: |content| {
                SystemMatcher::KeyValue { key: "VERSION_ID" }
                    .find(content)
                    .map(SystemVersion::from_string)
            },
        }];

        let result =
            retrieve_release_info(&custom_distributions, temp_dir.path().to_str().unwrap());
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.system_type, Type::Ubuntu);
        assert_eq!(info.version, SystemVersion::Unknown);
    }
}
