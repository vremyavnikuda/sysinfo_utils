//! Extension traits for convenient operations on system information types.
//!
//! This module provides extension traits that add convenient methods for
//! querying and inspecting system information objects.

use crate::{BitDepth, Info, SystemVersion, Type};

/// Extension trait for [`SystemVersion`] providing convenient query methods.
///
/// This trait adds helper methods to check the variant type and extract
/// version components from a `SystemVersion` instance.
///
/// # Examples
///
/// ```
/// use system_info_lib::{SystemVersion, SystemVersionExt};
///
/// let version = SystemVersion::semantic(5, 15, 0);
/// assert!(version.is_semantic());
/// assert_eq!(version.major(), Some(5));
/// assert_eq!(version.minor(), Some(15));
/// assert_eq!(version.patch(), Some(0));
/// ```
pub trait SystemVersionExt {
    /// Returns `true` if this is a semantic version.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{SystemVersion, SystemVersionExt};
    ///
    /// let semantic = SystemVersion::semantic(1, 2, 3);
    /// assert!(semantic.is_semantic());
    ///
    /// let rolling = SystemVersion::rolling(None::<String>);
    /// assert!(!rolling.is_semantic());
    /// ```
    fn is_semantic(&self) -> bool;

    /// Returns `true` if this is a rolling release version.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{SystemVersion, SystemVersionExt};
    ///
    /// let rolling = SystemVersion::rolling(Some("2024.01"));
    /// assert!(rolling.is_rolling());
    ///
    /// let semantic = SystemVersion::semantic(1, 2, 3);
    /// assert!(!semantic.is_rolling());
    /// ```
    fn is_rolling(&self) -> bool;

    /// Returns `true` if this is a custom version.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{SystemVersion, SystemVersionExt};
    ///
    /// let custom = SystemVersion::custom("NT 10.0");
    /// assert!(custom.is_custom());
    ///
    /// let semantic = SystemVersion::semantic(1, 2, 3);
    /// assert!(!custom.is_semantic());
    /// ```
    fn is_custom(&self) -> bool;

    /// Returns `true` if this is an unknown version.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{SystemVersion, SystemVersionExt};
    ///
    /// let unknown = SystemVersion::Unknown;
    /// assert!(unknown.is_unknown());
    ///
    /// let semantic = SystemVersion::semantic(1, 2, 3);
    /// assert!(!semantic.is_unknown());
    /// ```
    fn is_unknown(&self) -> bool;

    /// Returns the major version number if this is a semantic version.
    ///
    /// # Returns
    ///
    /// * `Some(u64)` - The major version number for semantic versions
    /// * `None` - For all other version types
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{SystemVersion, SystemVersionExt};
    ///
    /// let version = SystemVersion::semantic(5, 15, 0);
    /// assert_eq!(version.major(), Some(5));
    ///
    /// let rolling = SystemVersion::rolling(None::<String>);
    /// assert_eq!(rolling.major(), None);
    /// ```
    fn major(&self) -> Option<u64>;

    /// Returns the minor version number if this is a semantic version.
    ///
    /// # Returns
    ///
    /// * `Some(u64)` - The minor version number for semantic versions
    /// * `None` - For all other version types
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{SystemVersion, SystemVersionExt};
    ///
    /// let version = SystemVersion::semantic(5, 15, 0);
    /// assert_eq!(version.minor(), Some(15));
    ///
    /// let custom = SystemVersion::custom("1.0");
    /// assert_eq!(custom.minor(), None);
    /// ```
    fn minor(&self) -> Option<u64>;

    /// Returns the patch version number if this is a semantic version.
    ///
    /// # Returns
    ///
    /// * `Some(u64)` - The patch version number for semantic versions
    /// * `None` - For all other version types
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{SystemVersion, SystemVersionExt};
    ///
    /// let version = SystemVersion::semantic(5, 15, 23);
    /// assert_eq!(version.patch(), Some(23));
    ///
    /// let unknown = SystemVersion::Unknown;
    /// assert_eq!(unknown.patch(), None);
    /// ```
    fn patch(&self) -> Option<u64>;

    /// Returns the codename if this is a rolling release with a codename.
    ///
    /// # Returns
    ///
    /// * `Some(&str)` - The codename for rolling releases that have one
    /// * `None` - For rolling releases without codename or other version types
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{SystemVersion, SystemVersionExt};
    ///
    /// let rolling = SystemVersion::rolling(Some("2024.01"));
    /// assert_eq!(rolling.codename(), Some("2024.01"));
    ///
    /// let rolling_no_name = SystemVersion::rolling(None::<String>);
    /// assert_eq!(rolling_no_name.codename(), None);
    /// ```
    fn codename(&self) -> Option<&str>;
}

impl SystemVersionExt for SystemVersion {
    fn is_semantic(&self) -> bool {
        matches!(self, SystemVersion::Semantic(_, _, _))
    }

    fn is_rolling(&self) -> bool {
        matches!(self, SystemVersion::Rolling(_))
    }

    fn is_custom(&self) -> bool {
        matches!(self, SystemVersion::Custom(_))
    }

    fn is_unknown(&self) -> bool {
        matches!(self, SystemVersion::Unknown)
    }

    fn major(&self) -> Option<u64> {
        match self {
            SystemVersion::Semantic(major, _, _) => Some(*major),
            _ => None,
        }
    }

    fn minor(&self) -> Option<u64> {
        match self {
            SystemVersion::Semantic(_, minor, _) => Some(*minor),
            _ => None,
        }
    }

    fn patch(&self) -> Option<u64> {
        match self {
            SystemVersion::Semantic(_, _, patch) => Some(*patch),
            _ => None,
        }
    }

    fn codename(&self) -> Option<&str> {
        match self {
            SystemVersion::Rolling(Some(name)) => Some(name.as_str()),
            _ => None,
        }
    }
}

/// Extension trait for [`Info`] providing convenient query methods.
///
/// This trait adds helper methods to easily check system properties without
/// manual pattern matching or comparisons.
///
/// # Examples
///
/// ```
/// use system_info_lib::{Info, Type, BitDepth, InfoExt};
///
/// let info = Info::builder()
///     .system_type(Type::Linux)
///     .bit_depth(BitDepth::X64)
///     .build();
///
/// assert!(info.is_linux());
/// assert!(info.is_64bit());
/// assert!(!info.is_windows());
/// ```
pub trait InfoExt {
    /// Returns `true` if the operating system is Windows.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, Type, InfoExt};
    ///
    /// let info = Info::builder().system_type(Type::Windows).build();
    /// assert!(info.is_windows());
    ///
    /// let info = Info::builder().system_type(Type::Linux).build();
    /// assert!(!info.is_windows());
    /// ```
    fn is_windows(&self) -> bool;

    /// Returns `true` if the operating system is any Linux distribution.
    ///
    /// This includes all Linux-based systems like Ubuntu, Fedora, Arch, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, Type, InfoExt};
    ///
    /// let info = Info::builder().system_type(Type::Linux).build();
    /// assert!(info.is_linux());
    ///
    /// let info = Info::builder().system_type(Type::Ubuntu).build();
    /// assert!(info.is_linux());
    ///
    /// let info = Info::builder().system_type(Type::Windows).build();
    /// assert!(!info.is_linux());
    /// ```
    fn is_linux(&self) -> bool;

    /// Returns `true` if the operating system is macOS.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, Type, InfoExt};
    ///
    /// let info = Info::builder().system_type(Type::Macos).build();
    /// assert!(info.is_macos());
    ///
    /// let info = Info::builder().system_type(Type::Linux).build();
    /// assert!(!info.is_macos());
    /// ```
    fn is_macos(&self) -> bool;

    /// Returns `true` if the operating system is a BSD variant.
    ///
    /// This includes FreeBSD, OpenBSD, NetBSD, DragonFly, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, Type, InfoExt};
    ///
    /// let info = Info::builder().system_type(Type::FreeBSD).build();
    /// assert!(info.is_bsd());
    ///
    /// let info = Info::builder().system_type(Type::OpenBSD).build();
    /// assert!(info.is_bsd());
    ///
    /// let info = Info::builder().system_type(Type::Linux).build();
    /// assert!(!info.is_bsd());
    /// ```
    fn is_bsd(&self) -> bool;

    /// Returns `true` if the system has a 64-bit architecture.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, BitDepth, InfoExt};
    ///
    /// let info = Info::builder().bit_depth(BitDepth::X64).build();
    /// assert!(info.is_64bit());
    ///
    /// let info = Info::builder().bit_depth(BitDepth::X32).build();
    /// assert!(!info.is_64bit());
    /// ```
    fn is_64bit(&self) -> bool;

    /// Returns `true` if the system has a 32-bit architecture.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, BitDepth, InfoExt};
    ///
    /// let info = Info::builder().bit_depth(BitDepth::X32).build();
    /// assert!(info.is_32bit());
    ///
    /// let info = Info::builder().bit_depth(BitDepth::X64).build();
    /// assert!(!info.is_32bit());
    /// ```
    fn is_32bit(&self) -> bool;

    /// Returns `true` if the system architecture is unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, BitDepth, InfoExt};
    ///
    /// let info = Info::builder().bit_depth(BitDepth::Unknown).build();
    /// assert!(info.has_unknown_bit_depth());
    ///
    /// let info = Info::builder().bit_depth(BitDepth::X64).build();
    /// assert!(!info.has_unknown_bit_depth());
    /// ```
    fn has_unknown_bit_depth(&self) -> bool;

    /// Returns `true` if the system type is unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, Type, InfoExt};
    ///
    /// let info = Info::builder().system_type(Type::Unknown).build();
    /// assert!(info.is_unknown_system());
    ///
    /// let info = Info::builder().system_type(Type::Linux).build();
    /// assert!(!info.is_unknown_system());
    /// ```
    fn is_unknown_system(&self) -> bool;

    /// Returns `true` if kernel version information is available.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, InfoExt};
    ///
    /// let info = Info::builder().kernel_version("5.15.0").build();
    /// assert!(info.has_kernel_version());
    ///
    /// let info = Info::builder().build();
    /// assert!(!info.has_kernel_version());
    /// ```
    fn has_kernel_version(&self) -> bool;
}

impl InfoExt for Info {
    fn is_windows(&self) -> bool {
        self.system_type() == Type::Windows
    }

    fn is_linux(&self) -> bool {
        matches!(
            self.system_type(),
            Type::Linux
                | Type::Ubuntu
                | Type::Debian
                | Type::Fedora
                | Type::Arch
                | Type::CentOS
                | Type::Alpine
                | Type::Gentoo
                | Type::Manjaro
                | Type::openSUSE
                | Type::SUSE
                | Type::Redhat
                | Type::RedHatEnterprise
                | Type::AlmaLinux
                | Type::RockyLinux
                | Type::OracleLinux
                | Type::Amazon
                | Type::Kali
                | Type::Pop
                | Type::Mint
                | Type::EndeavourOS
                | Type::Garuda
                | Type::Artix
                | Type::CachyOS
                | Type::Mabox
                | Type::Nobara
                | Type::Ultramarine
                | Type::NixOS
                | Type::Void
                | Type::Solus
                | Type::Mariner
                | Type::Raspbian
                | Type::Alpaquita
                | Type::ChromeOS
                | Type::ClearLinux
                | Type::OpenWrt
                | Type::Silverblue
                | Type::OpenCloudOS
                | Type::openEuler
                | Type::Uos
        )
    }

    fn is_macos(&self) -> bool {
        self.system_type() == Type::Macos
    }

    fn is_bsd(&self) -> bool {
        matches!(
            self.system_type(),
            Type::FreeBSD
                | Type::OpenBSD
                | Type::NetBSD
                | Type::DragonFly
                | Type::HardenedBSD
                | Type::MidnightBSD
        )
    }

    fn is_64bit(&self) -> bool {
        self.bit_depth() == BitDepth::X64
    }

    fn is_32bit(&self) -> bool {
        self.bit_depth() == BitDepth::X32
    }

    fn has_unknown_bit_depth(&self) -> bool {
        self.bit_depth() == BitDepth::Unknown
    }

    fn is_unknown_system(&self) -> bool {
        self.system_type() == Type::Unknown
    }

    fn has_kernel_version(&self) -> bool {
        self.kernel_version().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod system_version_ext {
        use super::*;

        #[test]
        fn test_is_semantic() {
            let version = SystemVersion::semantic(1, 2, 3);
            assert!(version.is_semantic());
            assert!(!version.is_rolling());
            assert!(!version.is_custom());
            assert!(!version.is_unknown());
        }

        #[test]
        fn test_is_rolling() {
            let version = SystemVersion::rolling(Some("2024"));
            assert!(version.is_rolling());
            assert!(!version.is_semantic());
            assert!(!version.is_custom());
            assert!(!version.is_unknown());
        }

        #[test]
        fn test_is_custom() {
            let version = SystemVersion::custom("NT 10.0");
            assert!(version.is_custom());
            assert!(!version.is_semantic());
            assert!(!version.is_rolling());
            assert!(!version.is_unknown());
        }

        #[test]
        fn test_is_unknown() {
            let version = SystemVersion::Unknown;
            assert!(version.is_unknown());
            assert!(!version.is_semantic());
            assert!(!version.is_rolling());
            assert!(!version.is_custom());
        }

        #[test]
        fn test_major_minor_patch() {
            let version = SystemVersion::semantic(5, 15, 23);
            assert_eq!(version.major(), Some(5));
            assert_eq!(version.minor(), Some(15));
            assert_eq!(version.patch(), Some(23));
        }

        #[test]
        fn test_major_minor_patch_non_semantic() {
            let rolling = SystemVersion::rolling(None::<String>);
            assert_eq!(rolling.major(), None);
            assert_eq!(rolling.minor(), None);
            assert_eq!(rolling.patch(), None);

            let unknown = SystemVersion::Unknown;
            assert_eq!(unknown.major(), None);
            assert_eq!(unknown.minor(), None);
            assert_eq!(unknown.patch(), None);
        }

        #[test]
        fn test_codename() {
            let rolling = SystemVersion::rolling(Some("2024.01"));
            assert_eq!(rolling.codename(), Some("2024.01"));

            let rolling_no_name = SystemVersion::rolling(None::<String>);
            assert_eq!(rolling_no_name.codename(), None);

            let semantic = SystemVersion::semantic(1, 2, 3);
            assert_eq!(semantic.codename(), None);
        }

        #[test]
        fn test_zero_version() {
            let version = SystemVersion::semantic(0, 0, 0);
            assert!(version.is_semantic());
            assert_eq!(version.major(), Some(0));
            assert_eq!(version.minor(), Some(0));
            assert_eq!(version.patch(), Some(0));
        }
    }

    mod info_ext {
        use super::*;

        #[test]
        fn test_is_windows() {
            let info = Info::builder().system_type(Type::Windows).build();
            assert!(info.is_windows());
            assert!(!info.is_linux());
            assert!(!info.is_macos());
            assert!(!info.is_bsd());
        }

        #[test]
        fn test_is_linux() {
            let info = Info::builder().system_type(Type::Linux).build();
            assert!(info.is_linux());
            assert!(!info.is_windows());

            let ubuntu = Info::builder().system_type(Type::Ubuntu).build();
            assert!(ubuntu.is_linux());

            let arch = Info::builder().system_type(Type::Arch).build();
            assert!(arch.is_linux());
        }

        #[test]
        fn test_is_macos() {
            let info = Info::builder().system_type(Type::Macos).build();
            assert!(info.is_macos());
            assert!(!info.is_linux());
            assert!(!info.is_windows());
        }

        #[test]
        fn test_is_bsd() {
            let freebsd = Info::builder().system_type(Type::FreeBSD).build();
            assert!(freebsd.is_bsd());

            let openbsd = Info::builder().system_type(Type::OpenBSD).build();
            assert!(openbsd.is_bsd());

            let netbsd = Info::builder().system_type(Type::NetBSD).build();
            assert!(netbsd.is_bsd());

            let linux = Info::builder().system_type(Type::Linux).build();
            assert!(!linux.is_bsd());
        }

        #[test]
        fn test_bit_depth_checks() {
            let info_64 = Info::builder().bit_depth(BitDepth::X64).build();
            assert!(info_64.is_64bit());
            assert!(!info_64.is_32bit());
            assert!(!info_64.has_unknown_bit_depth());

            let info_32 = Info::builder().bit_depth(BitDepth::X32).build();
            assert!(info_32.is_32bit());
            assert!(!info_32.is_64bit());
            assert!(!info_32.has_unknown_bit_depth());

            let info_unknown = Info::builder().bit_depth(BitDepth::Unknown).build();
            assert!(info_unknown.has_unknown_bit_depth());
            assert!(!info_unknown.is_32bit());
            assert!(!info_unknown.is_64bit());
        }

        #[test]
        fn test_is_unknown_system() {
            let unknown = Info::builder().system_type(Type::Unknown).build();
            assert!(unknown.is_unknown_system());

            let linux = Info::builder().system_type(Type::Linux).build();
            assert!(!linux.is_unknown_system());
        }

        #[test]
        fn test_has_kernel_version() {
            let with_kernel = Info::builder().kernel_version("5.15.0").build();
            assert!(with_kernel.has_kernel_version());

            let without_kernel = Info::builder().build();
            assert!(!without_kernel.has_kernel_version());
        }

        #[test]
        fn test_all_linux_distributions() {
            let distros = [
                Type::Ubuntu,
                Type::Debian,
                Type::Fedora,
                Type::Arch,
                Type::Manjaro,
                Type::Pop,
                Type::Mint,
                Type::ChromeOS,
                Type::ClearLinux,
                Type::OpenWrt,
                Type::Silverblue,
            ];

            for distro in distros {
                let info = Info::builder().system_type(distro).build();
                assert!(info.is_linux(), "{:?} should be detected as Linux", distro);
            }
        }

        #[test]
        fn test_combined_checks() {
            let linux_64 = Info::builder()
                .system_type(Type::Ubuntu)
                .bit_depth(BitDepth::X64)
                .kernel_version("5.15.0")
                .build();

            assert!(linux_64.is_linux());
            assert!(linux_64.is_64bit());
            assert!(linux_64.has_kernel_version());
            assert!(!linux_64.is_windows());
            assert!(!linux_64.is_32bit());
        }
    }
}
