use std::fmt::Display;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[non_exhaustive]

pub enum Type {
    AIX,
    AlmaLinux,
    Alpaquita,
    Alpine,
    Amazon,
    Android,
    Arch,
    Artix,
    CachyOS,
    CentOS,
    Debian,
    DragonFly,
    Emscripten,
    EndeavourOS,
    Fedora,
    FreeBSD,
    Garuda,
    Gentoo,
    HardenedBSD,
    Illumos,
    Kali,
    Linux,
    Mabox,
    Macos,
    Manjaro,
    Mariner,
    MidnightBSD,
    Mint,
    NetBSD,
    NixOS,
    Nobara,
    OpenBSD,
    OpenCloudOS,
    openEuler,
    openSUSE,
    OracleLinux,
    Pop,
    Raspbian,
    Redhat,
    RedHatEnterprise,
    Redox,
    RockyLinux,
    Solus,
    SUSE,
    Ubuntu,
    Ultramarine,
    Uos,
    Void,
    Unknown,
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
    /// The string is the name of the OS type, without any additional information.
    ///
    /// # Examples
    ///
    ///
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Type::AIX => write!(f, "AIX"),
            Type::AlmaLinux => write!(f, "AlmaLinux"),
            Type::Alpaquita => write!(f, "Alpaquita"),
            Type::Alpine => write!(f, "Alpine"),
            Type::Amazon => write!(f, "Amazon"),
            Type::Android => write!(f, "Android"),
            Type::Arch => write!(f, "Arch"),
            Type::Artix => write!(f, "Artix"),
            Type::CachyOS => write!(f, "CachyOS"),
            Type::CentOS => write!(f, "CentOS"),
            Type::Debian => write!(f, "Debian"),
            Type::DragonFly => write!(f, "DragonFly"),
            Type::Emscripten => write!(f, "Emscripten"),
            Type::EndeavourOS => write!(f, "EndeavourOS"),
            Type::Fedora => write!(f, "Fedora"),
            Type::FreeBSD => write!(f, "FreeBSD"),
            Type::Garuda => write!(f, "Garuda"),
            Type::Gentoo => write!(f, "Gentoo"),
            Type::HardenedBSD => write!(f, "HardenedBSD"),
            Type::Illumos => write!(f, "Illumos"),
            Type::Kali => write!(f, "Kali"),
            Type::Linux => write!(f, "Linux"),
            Type::Mabox => write!(f, "Mabox"),
            Type::Macos => write!(f, "Macos"),
            Type::Manjaro => write!(f, "Manjaro"),
            Type::Mariner => write!(f, "Mariner"),
            Type::MidnightBSD => write!(f, "MidnightBSD"),
            Type::Mint => write!(f, "Mint"),
            Type::NetBSD => write!(f, "NetBSD"),
            Type::NixOS => write!(f, "NixOS"),
            Type::Nobara => write!(f, "Nobara"),
            Type::OpenBSD => write!(f, "OpenBSD"),
            Type::OpenCloudOS => write!(f, "OpenCloudOS"),
            Type::openEuler => write!(f, "openEuler"),
            Type::openSUSE => write!(f, "OpenSUSE"),
            Type::OracleLinux => write!(f, "OracleLinux"),
            Type::Pop => write!(f, "Pop"),
            Type::Raspbian => write!(f, "Raspbian"),
            Type::Redhat => write!(f, "Redhat"),
            Type::RedHatEnterprise => write!(f, "RedHatEnterprise"),
            Type::Redox => write!(f, "Redox"),
            Type::RockyLinux => write!(f, "RockyLinux"),
            Type::Solus => write!(f, "Solus"),
            Type::SUSE => write!(f, "SUSE"),
            Type::Ubuntu => write!(f, "Ubuntu"),
            Type::Ultramarine => write!(f, "Ultramarine"),
            Type::Uos => write!(f, "Uos"),
            Type::Void => write!(f, "Void"),
            Type::Unknown => write!(f, "Unknown"),
            Type::Windows => write!(f, "Windows"),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::system_info::Info;

    /// Tests that `Info::default()` is correct.
    ///
    /// A default `Info` should have an `Unknown` system type, an `Unknown`
    /// version, and `None` for all other fields.
    #[test]
    fn test_info_default() {
        let info = Info::default();
        assert_eq!(info.system_type, Type::Unknown);
        assert_eq!(info.version, Version::unknown());
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
        assert_eq!(info.version, Version::unknown());
        assert_eq!(info.edition, None);
        assert_eq!(info.codename, None);
        assert_eq!(info.bit_depth, BitDepth::Unknown);
        assert_eq!(info.architecture, None);
    }

    /// Test that `Info::write_type` creates an `Info` with the specified system type
    /// and all other fields set to their default values.
    #[test]
    fn test_info_write_type() {
        let info = Info::write_type(Type::Linux);
        assert_eq!(info.system_type, Type::Linux);
        assert_eq!(info.version, Version::unknown());
        assert_eq!(info.edition, None);
        assert_eq!(info.codename, None);
    /// Tests that `Info` implements `Display` correctly.
    ///
    /// Verifies that the output of `Display` for `Info` is in the
    /// correct format, including the system type, version number,
    /// edition, codename, bit depth, and architecture.
        assert_eq!(info.bit_depth, BitDepth::Unknown);
        assert_eq!(info.architecture, None);
    }

    /// Tests that `Info` implements `Display` correctly.
    ///
    /// Verifies that the output of `Display` for `Info` is in the
    /// correct format, including the system type, version number,
    /// edition, codename, bit depth, and architecture.
    #[test]
    fn test_info_display() {
        let info = Info {
            system_type: Type::Linux,
            version: Version::new(1, 0, 0),
            edition: Some("Pro".to_string()),
            codename: Some("Focal".to_string()),
            bit_depth: BitDepth::X64,
            architecture: Some("x86_64".to_string()),
        };
        let display = format!("{}", info);
        assert_eq!(display, "Linux Pro (Focal) 1.0.0, 64-bit, x86_64");
    }
}