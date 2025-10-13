//src/system_info.rs
use std::fmt::Display;
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[non_exhaustive]
/// Represents the different types of operating systems.
///
/// Covers a wide range of operating systems from various distributions
/// of Linux, BSD variants, macOS, Windows, and other specialized systems.
pub enum Type {
    /// IBM AIX - Enterprise Unix operating system from IBM
    AIX,
    /// Linux distribution based on Red Hat Enterprise Linux
    AlmaLinux,
    /// Lightweight Linux distribution focused on simplicity
    Alpaquita,
    /// Minimalist Linux distribution oriented towards security
    Alpine,
    /// Cloud-based Linux distribution by Amazon Web Services
    Amazon,
    /// Mobile operating system for smartphones and tablets
    Android,
    /// Independent Linux distribution with a rolling-release model
    Arch,
    /// Independent version of Arch Linux without systemd
    Artix,
    /// High-performance Linux distribution based on Arch
    CachyOS,
    /// Commercial Linux distribution based on Red Hat
    CentOS,
    /// Browser-based operating system by Google
    ChromeOS,
    /// Performance-oriented Linux distribution by Intel
    ClearLinux,
    /// One of the oldest and most stable Linux distributions
    Debian,
    /// Unix-like operating system, a fork of BSD
    DragonFly,
    /// Compiler and runtime for web technologies
    Emscripten,
    /// Arch-based Linux distribution with aesthetic design
    EndeavourOS,
    /// Flagship Linux distribution from Red Hat
    Fedora,
    /// Unix-like operating system with open-source code
    FreeBSD,
    /// Gaming Linux distribution based on Arch
    Garuda,
    /// Linux distribution with a high degree of customization
    Gentoo,
    /// A secure version of BSD with enhanced protection
    HardenedBSD,
    /// Open-source operating system based on Solaris
    Illumos,
    /// Linux distribution for penetration testing and cybersecurity
    Kali,
    /// A general term for Unix-like operating systems
    Linux,
    /// Linux distribution with a minimalist interface
    Mabox,
    /// Operating system from Apple for desktop computers
    Macos,
    /// Popular Linux distribution based on Arch
    Manjaro,
    /// Linux distribution by Microsoft for microservices
    Mariner,
    /// BSD-like operating system
    MidnightBSD,
    /// Popular Linux distribution based on Ubuntu
    Mint,
    /// Portable BSD operating system
    NetBSD,
    /// Functional Linux distribution with declarative configuration
    NixOS,
    /// Gaming Linux distribution based on Fedora
    Nobara,
    /// Another BSD operating system with a focus on security
    OpenBSD,
    /// Open cloud operating system from Tencent
    OpenCloudOS,
    /// Open operating system from Huawei
    openEuler,
    /// Linux distribution by SUSE
    openSUSE,
    /// Embedded Linux distribution for routers
    OpenWrt,
    /// Enterprise Linux distribution by Oracle
    OracleLinux,
    /// Linux distribution by System76
    Pop,
    /// Linux distribution for Raspberry Pi
    Raspbian,
    /// Commercial Linux distribution
    Redhat,
    /// Enterprise version of Red Hat Linux
    RedHatEnterprise,
    /// Experimental operating system written in Rust
    Redox,
    /// Linux distribution compatible with Red Hat Enterprise
    RockyLinux,
    /// Independent Linux distribution
    Solus,
    /// Immutable Fedora variant with OSTree
    Silverblue,
    /// Enterprise Linux distribution by SUSE
    SUSE,
    /// Popular Linux distribution by Canonical
    Ubuntu,
    /// Linux distribution with Asian localization
    Ultramarine,
    /// Operating system from Chinese company Deepin
    Uos,
    /// Independent Linux distribution
    Void,
    /// Used when the system type cannot be determined
    Unknown,
    /// Operating system by Microsoft
    Windows,
}

impl Default for Type {
    /// Returns the default `Type`, which is `Type::Unknown`.
    ///
    /// This is used to provide a default value for the `Type` enum
    /// when one is not explicitly specified.
    fn default() -> Self {
        Type::Unknown
    }
}

impl Display for Type {
    /// Formats the OS type into a string.
    ///
    /// The string is the name_gpu of the OS type, without any additional information.
    ///
    /// # Examples
    ///
    ///
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Type::AIX => write!(f, "AIX"),
            Type::AlmaLinux => write!(f, "AlmaLinux"),
            Type::Alpaquita => write!(f, "Alpaquita Linux"),
            Type::Alpine => write!(f, "Alpine Linux"),
            Type::Amazon => write!(f, "Amazon Linux AMI"),
            Type::Android => write!(f, "Android"),
            Type::Arch => write!(f, "Arch Linux"),
            Type::Artix => write!(f, "Artix Linux"),
            Type::CachyOS => write!(f, "CachyOS Linux"),
            Type::CentOS => write!(f, "CentOS"),
            Type::ChromeOS => write!(f, "Chrome OS"),
            Type::ClearLinux => write!(f, "Clear Linux"),
            Type::Debian => write!(f, "Debian"),
            Type::DragonFly => write!(f, "DragonFly BSD"),
            Type::Emscripten => write!(f, "Emscripten"),
            Type::EndeavourOS => write!(f, "EndeavourOS"),
            Type::Fedora => write!(f, "Fedora"),
            Type::FreeBSD => write!(f, "FreeBSD"),
            Type::Garuda => write!(f, "Garuda Linux"),
            Type::Gentoo => write!(f, "Gentoo Linux"),
            Type::HardenedBSD => write!(f, "HardenedBSD"),
            Type::Illumos => write!(f, "Illumos"),
            Type::Kali => write!(f, "Kali Linux"),
            Type::Linux => write!(f, "Linux"),
            Type::Mabox => write!(f, "Mabox"),
            Type::Macos => write!(f, "Mac OS"),
            Type::Manjaro => write!(f, "Manjaro"),
            Type::Mariner => write!(f, "Mariner"),
            Type::MidnightBSD => write!(f, "Midnight BSD"),
            Type::Mint => write!(f, "Linux Mint"),
            Type::NetBSD => write!(f, "NetBSD"),
            Type::NixOS => write!(f, "NixOS"),
            Type::Nobara => write!(f, "Nobara Linux"),
            Type::OpenBSD => write!(f, "OpenBSD"),
            Type::OpenCloudOS => write!(f, "OpenCloudOS"),
            Type::openEuler => write!(f, "EulerOS"),
            Type::openSUSE => write!(f, "openSUSE"),
            Type::OpenWrt => write!(f, "OpenWrt"),
            Type::OracleLinux => write!(f, "Oracle Linux"),
            Type::Pop => write!(f, "Pop!_OS"),
            Type::Raspbian => write!(f, "Raspberry Pi OS"),
            Type::Redhat => write!(f, "Red Hat Linux"),
            Type::RedHatEnterprise => write!(f, "Red Hat Enterprise Linux"),
            Type::Redox => write!(f, "Redox"),
            Type::RockyLinux => write!(f, "Rocky Linux"),
            Type::Solus => write!(f, "Solus"),
            Type::Silverblue => write!(f, "Fedora Silverblue"),
            Type::SUSE => write!(f, "SUSE Linux Enterprise Server"),
            Type::Ubuntu => write!(f, "Ubuntu"),
            Type::Ultramarine => write!(f, "Ultramarine Linux"),
            Type::Uos => write!(f, "UOS"),
            Type::Void => write!(f, "Void Linux"),
            Type::Unknown => write!(f, "Unknown"),
            Type::Windows => write!(f, "Windows"),
        }
    }
}

#[cfg(test)]
mod system_os_tests {
    use super::*;
    use crate::bit_depth::BitDepth;
    use crate::system_info::Info;
    use crate::system_version::SystemVersion;
    use pretty_assertions::assert_eq;

    /// Tests that `Info::default()` is correct.
    ///
    /// A default `Info` should have an `Unknown` system type, an `Unknown`
    /// version, and `None` for all other fields.
    #[test]
    fn test_info_default() {
        let info = Info::default();
        assert_eq!(info.system_type, Type::Unknown);
        assert_eq!(info.version, SystemVersion::Unknown);
        assert_eq!(info.edition, None);
        assert_eq!(info.codename, None);
        assert_eq!(info.bit_depth, BitDepth::Unknown);
        assert_eq!(info.architecture, None);
    }

    /// Tests that `Info::unknown()` returns an `Info` with all fields set to
    /// their default values.
    ///
    /// Verifies that the output of `unknown()` is an `Info` with an
    /// `Unknown` system type, an `Unknown` version, and `None` for all
    /// other fields.
    #[test]
    fn test_info_unknown() {
        let info = Info::unknown();
        assert_eq!(info.system_type, Type::Unknown);
        assert_eq!(info.version, SystemVersion::Unknown);
        assert_eq!(info.edition, None);
        assert_eq!(info.codename, None);
        assert_eq!(info.bit_depth, BitDepth::Unknown);
        assert_eq!(info.architecture, None);
    }

    /// Test that `Info::write_type` creates an `Info` with the specified system type
    /// and all other fields set to their default values.
    #[test]
    fn test_info_write_type() {
        let info = Info::with_type(Type::Linux);
        assert_eq!(info.system_type, Type::Linux);
        assert_eq!(info.version, SystemVersion::Unknown);
        assert_eq!(info.edition, None);
        assert_eq!(info.codename, None);
        assert_eq!(info.bit_depth, BitDepth::Unknown);
        assert_eq!(info.architecture, None);
    }

    /// Tests that `Info` implements `Display` correctly.
    ///
    /// Verifies that the output of `Display` for `Info` is in the
    /// correct format, including the system type, version number,
    /// edition, codename, a bit of depth, and architecture.
    #[test]
    fn test_info_display() {
        let info = Info {
            system_type: Type::Linux,
            version: SystemVersion::Semantic(1, 1, 1),
            edition: Some("Pro".to_string()),
            codename: Some("Focal".to_string()),
            bit_depth: BitDepth::X64,
            architecture: Some("x86_64".to_string()),
        };
        let display = format!("{}", info);
        assert_eq!(display, "Linux Pro (Focal) 1.1.1, 64-bit, x86_64");
    }

    #[test]
    fn default() {
        assert_eq!(Type::Unknown, Type::default());
    }

    #[test]
    fn display() {
        let data = [
            (Type::AIX, "AIX"),
            (Type::AlmaLinux, "AlmaLinux"),
            (Type::Alpaquita, "Alpaquita Linux"),
            (Type::Alpine, "Alpine Linux"),
            (Type::Amazon, "Amazon Linux AMI"),
            (Type::Android, "Android"),
            (Type::Arch, "Arch Linux"),
            (Type::Artix, "Artix Linux"),
            (Type::CachyOS, "CachyOS Linux"),
            (Type::CentOS, "CentOS"),
            (Type::ChromeOS, "Chrome OS"),
            (Type::ClearLinux, "Clear Linux"),
            (Type::Debian, "Debian"),
            (Type::DragonFly, "DragonFly BSD"),
            (Type::Emscripten, "Emscripten"),
            (Type::EndeavourOS, "EndeavourOS"),
            (Type::Fedora, "Fedora"),
            (Type::FreeBSD, "FreeBSD"),
            (Type::Garuda, "Garuda Linux"),
            (Type::Gentoo, "Gentoo Linux"),
            (Type::HardenedBSD, "HardenedBSD"),
            (Type::Illumos, "Illumos"),
            (Type::Kali, "Kali Linux"),
            (Type::Linux, "Linux"),
            (Type::Mabox, "Mabox"),
            (Type::Macos, "Mac OS"),
            (Type::Manjaro, "Manjaro"),
            (Type::Mariner, "Mariner"),
            (Type::MidnightBSD, "Midnight BSD"),
            (Type::Mint, "Linux Mint"),
            (Type::NetBSD, "NetBSD"),
            (Type::NixOS, "NixOS"),
            (Type::Nobara, "Nobara Linux"),
            (Type::OpenCloudOS, "OpenCloudOS"),
            (Type::OpenBSD, "OpenBSD"),
            (Type::openEuler, "EulerOS"),
            (Type::openSUSE, "openSUSE"),
            (Type::OpenWrt, "OpenWrt"),
            (Type::OracleLinux, "Oracle Linux"),
            (Type::Pop, "Pop!_OS"),
            (Type::Raspbian, "Raspberry Pi OS"),
            (Type::Redhat, "Red Hat Linux"),
            (Type::RedHatEnterprise, "Red Hat Enterprise Linux"),
            (Type::Redox, "Redox"),
            (Type::RockyLinux, "Rocky Linux"),
            (Type::Solus, "Solus"),
            (Type::Silverblue, "Fedora Silverblue"),
            (Type::SUSE, "SUSE Linux Enterprise Server"),
            (Type::Ubuntu, "Ubuntu"),
            (Type::Ultramarine, "Ultramarine Linux"),
            (Type::Unknown, "Unknown"),
            (Type::Uos, "UOS"),
            (Type::Void, "Void Linux"),
            (Type::Windows, "Windows"),
        ];

        for (t, expected) in &data {
            assert_eq!(&t.to_string(), expected);
        }
    }
}
