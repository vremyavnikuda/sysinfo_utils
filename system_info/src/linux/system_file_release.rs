use crate::system_os::Type;
use crate::{BitDepth, Info, SystemMatcher, SystemVersion};
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
            Ok(val) => val,
            Err(e) => {
                warn!("Unable to open {:?} file: {:?}", &path, e);
                continue;
            }
        };

        let mut file_content = String::new();
        if let Err(e) = file.read_to_string(&mut file_content) {
            warn!("Unable to read {:?} file: {:?}", &path, e);
            continue;
        }

        let os_type = (release_info.type_var)(&file_content);

        if os_type.is_none() {
            continue;
        }

        let version = (release_info.version)(&file_content);

        return Some(Info {
            system_type: os_type.unwrap(),
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
                    "openEuler" => Some(Type::OpenEuler),
                    "ol" => Some(Type::OracleLinux),
                    "opensuse" => Some(Type::OpenSUSE),
                    "opensuse-leap" => Some(Type::OpenSUSE),
                    "opensuse-microos" => Some(Type::OpenSUSE),
                    "opensuse-tumbleweed" => Some(Type::OpenSUSE),
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
