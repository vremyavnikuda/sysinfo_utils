//! Integration tests for system_info_lib public API.
//!
//! These tests verify the library works correctly when used as a dependency,
//! testing the public API surface and ensuring all exported types and functions
//! work as expected.

use system_info_lib::prelude::*;

#[allow(unused_imports)]
use std::assert_eq;

#[test]
fn test_get_returns_valid_info() {
    let info = get();

    assert_ne!(info.system_type(), Type::Unknown);
    assert_ne!(info.bit_depth(), BitDepth::Unknown);
}

#[test]
fn test_info_builder_creates_valid_info() {
    let info = Info::builder()
        .system_type(Type::Linux)
        .version(SystemVersion::semantic(5, 15, 0))
        .edition("Ubuntu")
        .codename("Jammy")
        .bit_depth(BitDepth::X64)
        .architecture("x86_64")
        .build();

    assert_eq!(info.system_type(), Type::Linux);
    assert_eq!(info.version(), &SystemVersion::semantic(5, 15, 0));
    assert_eq!(info.edition(), Some("Ubuntu"));
    assert_eq!(info.codename(), Some("Jammy"));
    assert_eq!(info.bit_depth(), BitDepth::X64);
    assert_eq!(info.architecture(), Some("x86_64"));
}

#[test]
fn test_info_builder_with_minimal_fields() {
    let info = Info::builder().system_type(Type::Arch).build();

    assert_eq!(info.system_type(), Type::Arch);
    assert_eq!(info.version(), &SystemVersion::Unknown);
    assert_eq!(info.edition(), None);
    assert_eq!(info.codename(), None);
    assert_eq!(info.bit_depth(), BitDepth::Unknown);
    assert_eq!(info.architecture(), None);
}

#[test]
fn test_system_version_constructors() {
    let semantic = SystemVersion::semantic(1, 2, 3);
    let rolling = SystemVersion::rolling(Some("focal"));
    let custom = SystemVersion::custom("NT 10.0");

    assert_eq!(semantic, SystemVersion::Semantic(1, 2, 3));
    assert_eq!(rolling, SystemVersion::Rolling(Some("focal".to_string())));
    assert_eq!(custom, SystemVersion::Custom("NT 10.0".to_string()));
}

#[test]
fn test_bit_depth_display() {
    assert_eq!(BitDepth::Unknown.to_string(), "unknown bit depth");
    assert_eq!(BitDepth::X32.to_string(), "32-bit");
    assert_eq!(BitDepth::X64.to_string(), "64-bit");
}

#[test]
fn test_type_display() {
    assert_eq!(Type::Linux.to_string(), "Linux");
    assert_eq!(Type::Arch.to_string(), "Arch Linux");
    assert_eq!(Type::Ubuntu.to_string(), "Ubuntu");
    assert_eq!(Type::Windows.to_string(), "Windows");
    assert_eq!(Type::Macos.to_string(), "Mac OS");
}

#[test]
fn test_info_display_full() {
    let info = Info::builder()
        .system_type(Type::Ubuntu)
        .version(SystemVersion::semantic(20, 4, 0))
        .edition("LTS")
        .codename("Focal Fossa")
        .bit_depth(BitDepth::X64)
        .architecture("x86_64")
        .build();

    let display = info.to_string();
    assert!(display.contains("Ubuntu"));
    assert!(display.contains("LTS"));
    assert!(display.contains("Focal Fossa"));
    assert!(display.contains("20.4.0"));
    assert!(display.contains("64-bit"));
    assert!(display.contains("x86_64"));
}

#[test]
fn test_info_display_minimal() {
    let info = Info::builder()
        .system_type(Type::Linux)
        .version(SystemVersion::Unknown)
        .bit_depth(BitDepth::Unknown)
        .build();

    let display = info.to_string();
    assert!(display.contains("Linux"));
}

#[test]
fn test_info_clone_and_equality() {
    let info1 = Info::builder()
        .system_type(Type::Fedora)
        .version(SystemVersion::semantic(39, 0, 0))
        .bit_depth(BitDepth::X64)
        .build();

    let info2 = info1.clone();

    assert_eq!(info1, info2);
    assert_eq!(info1.system_type(), info2.system_type());
    assert_eq!(info1.version(), info2.version());
}

#[test]
fn test_prelude_imports() {
    let _ = get();
    let _ = Info::builder().build();
    let _ = SystemVersion::semantic(1, 0, 0);
    let _ = BitDepth::X64;
    let _ = Type::Linux;
}

#[test]
fn test_system_version_equality() {
    let v1 = SystemVersion::semantic(5, 15, 0);
    let v2 = SystemVersion::Semantic(5, 15, 0);
    assert_eq!(v1, v2);

    let v3 = SystemVersion::rolling(Some("latest"));
    let v4 = SystemVersion::Rolling(Some("latest".to_string()));
    assert_eq!(v3, v4);
}

#[test]
fn test_info_ordering() {
    let info1 = Info::builder()
        .system_type(Type::Arch)
        .version(SystemVersion::Unknown)
        .build();

    let info2 = Info::builder()
        .system_type(Type::Ubuntu)
        .version(SystemVersion::semantic(20, 4, 0))
        .build();

    assert_ne!(info1, info2);
}

#[test]
fn test_new_distro_types() {
    let clear_linux = Type::ClearLinux;
    let chrome_os = Type::ChromeOS;
    let openwrt = Type::OpenWrt;
    let silverblue = Type::Silverblue;

    assert_eq!(clear_linux.to_string(), "Clear Linux");
    assert_eq!(chrome_os.to_string(), "Chrome OS");
    assert_eq!(openwrt.to_string(), "OpenWrt");
    assert_eq!(silverblue.to_string(), "Fedora Silverblue");
}

#[test]
fn test_info_unknown() {
    let info = Info::unknown();

    assert_eq!(info.system_type(), Type::Unknown);
    assert_eq!(info.version(), &SystemVersion::Unknown);
    assert_eq!(info.edition(), None);
    assert_eq!(info.codename(), None);
    assert_eq!(info.bit_depth(), BitDepth::Unknown);
    assert_eq!(info.architecture(), None);
}

#[test]
fn test_info_with_type() {
    let info = Info::with_type(Type::Linux);

    assert_eq!(info.system_type(), Type::Linux);
    assert_eq!(info.version(), &SystemVersion::Unknown);
    assert_eq!(info.edition(), None);
    assert_eq!(info.codename(), None);
    assert_eq!(info.bit_depth(), BitDepth::Unknown);
    assert_eq!(info.architecture(), None);
}

#[test]
fn test_info_default() {
    let info = Info::default();

    assert_eq!(info.system_type(), Type::Unknown);
    assert_eq!(info.version(), &SystemVersion::Unknown);
    assert_eq!(info.edition(), None);
    assert_eq!(info.codename(), None);
    assert_eq!(info.bit_depth(), BitDepth::Unknown);
    assert_eq!(info.architecture(), None);
}

#[test]
fn test_system_version_unknown() {
    let version = SystemVersion::Unknown;
    assert_eq!(version.to_string(), "Unknown");
}

#[test]
fn test_system_version_semantic_display() {
    let version = SystemVersion::semantic(5, 15, 23);
    assert_eq!(version.to_string(), "5.15.23");
}

#[test]
fn test_system_version_rolling_with_codename() {
    let version = SystemVersion::rolling(Some("latest"));
    assert_eq!(version.to_string(), "Rolling (latest)");
}

#[test]
fn test_system_version_rolling_without_codename() {
    let version = SystemVersion::rolling(None::<String>);
    assert_eq!(version.to_string(), "Rolling");
}

#[test]
fn test_system_version_custom() {
    let version = SystemVersion::custom("NT 10.0.19041");
    assert_eq!(version.to_string(), "NT 10.0.19041");
}

#[test]
fn test_system_version_default() {
    let version = SystemVersion::default();
    assert_eq!(version, SystemVersion::Unknown);
}

#[test]
fn test_type_default() {
    let t = Type::default();
    assert_eq!(t, Type::Unknown);
}

#[test]
fn test_bit_depth_ordering() {
    assert!(BitDepth::Unknown < BitDepth::X32);
    assert!(BitDepth::X32 < BitDepth::X64);
    assert!(BitDepth::X64 > BitDepth::X32);
}

#[test]
fn test_info_all_getters() {
    let info = Info::builder()
        .system_type(Type::Debian)
        .version(SystemVersion::semantic(11, 0, 0))
        .edition("Stable")
        .codename("Bullseye")
        .bit_depth(BitDepth::X64)
        .architecture("amd64")
        .build();

    assert_eq!(info.system_type(), Type::Debian);
    assert_eq!(info.version(), &SystemVersion::Semantic(11, 0, 0));
    assert_eq!(info.edition(), Some("Stable"));
    assert_eq!(info.codename(), Some("Bullseye"));
    assert_eq!(info.bit_depth(), BitDepth::X64);
    assert_eq!(info.architecture(), Some("amd64"));
}

#[test]
fn test_info_partial_getters() {
    let info = Info::builder()
        .system_type(Type::Arch)
        .version(SystemVersion::rolling(None::<String>))
        .bit_depth(BitDepth::X64)
        .build();

    assert_eq!(info.system_type(), Type::Arch);
    assert_eq!(info.version(), &SystemVersion::Rolling(None));
    assert_eq!(info.edition(), None);
    assert_eq!(info.codename(), None);
    assert_eq!(info.bit_depth(), BitDepth::X64);
    assert_eq!(info.architecture(), None);
}

#[test]
fn test_info_builder_default() {
    let builder = InfoBuilder::default();
    let info = builder.build();

    assert_eq!(info.system_type(), Type::Unknown);
    assert_eq!(info.version(), &SystemVersion::Unknown);
    assert_eq!(info.bit_depth(), BitDepth::Unknown);
}

#[test]
fn test_info_hash() {
    use std::collections::HashSet;

    let info1 = Info::builder()
        .system_type(Type::Linux)
        .version(SystemVersion::semantic(5, 15, 0))
        .build();

    let info2 = Info::builder()
        .system_type(Type::Linux)
        .version(SystemVersion::semantic(5, 15, 0))
        .build();

    let mut set = HashSet::new();
    set.insert(info1.clone());
    set.insert(info2.clone());

    assert_eq!(set.len(), 1);
}

#[test]
fn test_info_debug() {
    let info = Info::builder()
        .system_type(Type::Linux)
        .version(SystemVersion::semantic(5, 15, 0))
        .build();

    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("Info"));
    assert!(debug_str.contains("Linux"));
}

#[test]
fn test_system_version_debug() {
    let version = SystemVersion::semantic(5, 15, 0);
    let debug_str = format!("{:?}", version);
    assert!(debug_str.contains("Semantic"));
}

#[test]
fn test_type_debug() {
    let t = Type::Linux;
    let debug_str = format!("{:?}", t);
    assert!(debug_str.contains("Linux"));
}

#[test]
fn test_bit_depth_debug() {
    let depth = BitDepth::X64;
    let debug_str = format!("{:?}", depth);
    assert!(debug_str.contains("X64"));
}

#[test]
fn test_info_builder_new() {
    let builder = InfoBuilder::new();
    let info = builder.build();

    assert_eq!(info.system_type(), Type::Unknown);
    assert_eq!(info.version(), &SystemVersion::Unknown);
}

#[test]
fn test_bit_depth_copy() {
    let depth1 = BitDepth::X64;
    let depth2 = depth1;

    assert_eq!(depth1, depth2);
    assert_eq!(depth1, BitDepth::X64);
}

#[test]
fn test_bit_depth_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(BitDepth::X32);
    set.insert(BitDepth::X64);
    set.insert(BitDepth::X64);

    assert_eq!(set.len(), 2);
}

#[test]
fn test_system_version_ordering() {
    let v1 = SystemVersion::Unknown;
    let v2 = SystemVersion::Semantic(1, 0, 0);
    let v3 = SystemVersion::Semantic(2, 0, 0);

    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v3 > v2);
}

#[test]
fn test_system_version_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(SystemVersion::semantic(1, 0, 0));
    set.insert(SystemVersion::semantic(1, 0, 0));
    set.insert(SystemVersion::semantic(2, 0, 0));

    assert_eq!(set.len(), 2);
}

#[test]
fn test_type_ordering() {
    let t1 = Type::AIX;
    let t2 = Type::Linux;
    let t3 = Type::Windows;

    assert_ne!(t1, t2);
    assert_ne!(t2, t3);
}

#[test]
fn test_type_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(Type::Linux);
    set.insert(Type::Linux);
    set.insert(Type::Windows);

    assert_eq!(set.len(), 2);
}

#[test]
fn test_type_copy() {
    let t1 = Type::Linux;
    let t2 = t1;

    assert_eq!(t1, t2);
    assert_eq!(t1, Type::Linux);
}

#[test]
fn test_info_builder_chaining_ownership() {
    let info = InfoBuilder::new()
        .system_type(Type::Linux)
        .version(SystemVersion::semantic(5, 15, 0))
        .edition("Ubuntu")
        .codename("Focal")
        .bit_depth(BitDepth::X64)
        .architecture("x86_64")
        .build();

    assert_eq!(info.system_type(), Type::Linux);
}

#[test]
fn test_info_display_with_only_type() {
    let info = Info::with_type(Type::Linux);
    let display = info.to_string();

    assert!(display.contains("Linux"));
    assert!(display.contains("Unknown"));
}

#[test]
fn test_system_version_clone() {
    let v1 = SystemVersion::semantic(5, 15, 0);
    let v2 = v1.clone();

    assert_eq!(v1, v2);
}

#[test]
fn test_info_partial_eq() {
    let info1 = Info::builder()
        .system_type(Type::Linux)
        .version(SystemVersion::semantic(5, 15, 0))
        .build();

    let info2 = Info::builder()
        .system_type(Type::Linux)
        .version(SystemVersion::semantic(5, 15, 0))
        .build();

    let info3 = Info::builder()
        .system_type(Type::Windows)
        .version(SystemVersion::semantic(10, 0, 0))
        .build();

    assert_eq!(info1, info2);
    assert_ne!(info1, info3);
}

#[test]
fn test_builder_with_string_types() {
    let edition = String::from("Pro");
    let codename = String::from("Jammy");
    let arch = String::from("amd64");

    let info = Info::builder()
        .system_type(Type::Ubuntu)
        .edition(edition)
        .codename(codename)
        .architecture(arch)
        .build();

    assert_eq!(info.edition(), Some("Pro"));
    assert_eq!(info.codename(), Some("Jammy"));
    assert_eq!(info.architecture(), Some("amd64"));
}

#[test]
fn test_builder_with_str_slices() {
    let edition: &str = "Pro";
    let codename: &str = "Jammy";
    let arch: &str = "amd64";

    let info = Info::builder()
        .system_type(Type::Ubuntu)
        .edition(edition)
        .codename(codename)
        .architecture(arch)
        .build();

    assert_eq!(info.edition(), Some("Pro"));
    assert_eq!(info.codename(), Some("Jammy"));
    assert_eq!(info.architecture(), Some("amd64"));
}

#[test]
fn test_info_with_all_none_optionals() {
    let info = Info::builder()
        .system_type(Type::Linux)
        .version(SystemVersion::Unknown)
        .bit_depth(BitDepth::Unknown)
        .build();

    assert_eq!(info.edition(), None);
    assert_eq!(info.codename(), None);
    assert_eq!(info.architecture(), None);
}

#[test]
fn test_system_version_semantic_zero() {
    let version = SystemVersion::semantic(0, 0, 0);
    assert_eq!(version.to_string(), "0.0.0");
}

#[test]
fn test_system_version_semantic_large() {
    let version = SystemVersion::semantic(100, 200, 300);
    assert_eq!(version.to_string(), "100.200.300");
}

#[test]
fn test_system_version_custom_empty() {
    let version = SystemVersion::custom("");
    assert_eq!(version.to_string(), "");
}

#[test]
fn test_system_version_rolling_str() {
    let version = SystemVersion::rolling(Some("testing"));
    assert_eq!(version, SystemVersion::Rolling(Some("testing".to_string())));
}

#[test]
fn test_builder_multiple_chains() {
    let builder = InfoBuilder::new();
    let builder = builder.system_type(Type::Linux);
    let builder = builder.version(SystemVersion::semantic(5, 15, 0));
    let info = builder.build();

    assert_eq!(info.system_type(), Type::Linux);
    assert_eq!(info.version(), &SystemVersion::Semantic(5, 15, 0));
}
