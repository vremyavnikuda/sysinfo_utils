//src/system_info.rs
use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Operating system version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Represents a rolling release version of the system.
///
/// The `Rolling` variant contains an `Option<String>` which may hold
/// additional information about the rolling release version.
///
/// # Examples
///
/// ```
/// use system_info_lib::SystemVersion::Rolling;
///
/// let version = Rolling(Some(String::from("2023.10")));
/// let no_version = Rolling(None);
/// ```    
pub enum SystemVersion {
    /// Represents an unknown version of the system.
    Unknown,
    /// Represents a semantic version of the system with major, minor, and patch numbers.
    Semantic(u64, u64, u64),
    /// Represents a rolling release version of the system with an optional codename.
    Rolling(Option<String>),
    /// Represents a custom version of the system as a string.
    Custom(String),
}

impl SystemVersion {
    /// Creates a `SystemVersion` instance from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - A string that represents the version. It can be a semantic version
    ///   (e.g., "1.2.3"), a custom version string, or an empty string.
    ///
    /// # Returns
    ///
    /// * `SystemVersion::Unknown` if the string is empty.
    /// * `SystemVersion::Semantic` if the string is a valid semantic version.
    /// * `SystemVersion::Custom` for any other non-empty string.
    pub fn from_string<S: Into<String> + AsRef<str>>(s: S) -> Self {
        if s.as_ref().is_empty() {
            Self::Unknown
        } else if let Some((major, minor, patch)) = parse_version(s.as_ref()) {
            Self::Semantic(major, minor, patch)
        } else {
            Self::Custom(s.into())
        }
    }

    /// Creates a semantic version with major, minor, and patch numbers.
    ///
    /// This is a convenience constructor for creating `SystemVersion::Semantic`
    /// instances without directly using the enum variant.
    ///
    /// # Arguments
    ///
    /// * `major` - The major version number
    /// * `minor` - The minor version number
    /// * `patch` - The patch version number
    ///
    /// # Returns
    ///
    /// A `SystemVersion::Semantic` instance with the specified version numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::SystemVersion;
    ///
    /// let version = SystemVersion::semantic(5, 15, 0);
    /// assert_eq!(version.to_string(), "5.15.0");
    ///
    /// // Equivalent to:
    /// let version2 = SystemVersion::Semantic(5, 15, 0);
    /// assert_eq!(version, version2);
    /// ```
    pub fn semantic(major: u64, minor: u64, patch: u64) -> Self {
        Self::Semantic(major, minor, patch)
    }

    /// Creates a rolling release version with an optional codename.
    ///
    /// Rolling releases are distributions that continuously update rather than
    /// having fixed version numbers (e.g., Arch Linux, Gentoo).
    ///
    /// # Arguments
    ///
    /// * `codename` - Optional codename or identifier for the rolling release
    ///
    /// # Returns
    ///
    /// A `SystemVersion::Rolling` instance with the specified codename.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::SystemVersion;
    ///
    /// // Rolling release with codename
    /// let version = SystemVersion::rolling(Some("2024.01"));
    /// assert_eq!(version.to_string(), "Rolling (2024.01)");
    ///
    /// // Rolling release without codename
    /// let version = SystemVersion::rolling(None::<String>);
    /// assert_eq!(version.to_string(), "Rolling");
    ///
    /// // Works with &str
    /// let version = SystemVersion::rolling(Some("focal"));
    /// assert_eq!(version.to_string(), "Rolling (focal)");
    /// ```
    pub fn rolling<S: Into<String>>(codename: Option<S>) -> Self {
        Self::Rolling(codename.map(|s| s.into()))
    }

    /// Creates a custom version with an arbitrary version string.
    ///
    /// Use this for version formats that don't fit semantic or rolling
    /// release patterns (e.g., "NT 10.0", "2023.12-LTS").
    ///
    /// # Arguments
    ///
    /// * `version` - A custom version string
    ///
    /// # Returns
    ///
    /// A `SystemVersion::Custom` instance with the specified version string.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::SystemVersion;
    ///
    /// let version = SystemVersion::custom("NT 10.0");
    /// assert_eq!(version.to_string(), "NT 10.0");
    ///
    /// // Works with both &str and String
    /// let version = SystemVersion::custom(String::from("2023.12-LTS"));
    /// assert_eq!(version.to_string(), "2023.12-LTS");
    /// ```
    pub fn custom(version: impl Into<String>) -> Self {
        Self::Custom(version.into())
    }
}

impl Default for SystemVersion {
    /// Returns the default system version.
    ///
    /// This implementation of `Default` for `SystemVersion`
    /// returns `SystemVersion::Unknown`, representing an unknown version.
    fn default() -> Self {
        SystemVersion::Unknown
    }
}

impl Display for SystemVersion {
    /// Formats the version as a string.
    ///
    /// # Examples
    ///
    /// * `SystemVersion::Unknown` is formatted as `"Unknown"`.
    /// * `SystemVersion::Semantic(1, 2, 3)` is formatted as `"1.2.3"`.
    /// * `SystemVersion::Rolling(Some("focal".to_string()))` is formatted as `"Rolling (focal)"`.
    /// * `SystemVersion::Rolling(None)` is formatted as `"Rolling"`.
    /// * `SystemVersion::Custom("custom_version".to_string())` is formatted as `"custom_version"`.
    ///
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SystemVersion::Unknown => write!(f, "Unknown"),
            SystemVersion::Semantic(major, minor, patch) => {
                write!(f, "{}.{}.{}", major, minor, patch)
            }
            SystemVersion::Rolling(ref codename) => {
                if let Some(codename) = codename {
                    write!(f, "Rolling ({})", codename)
                } else {
                    write!(f, "Rolling")
                }
            }
            SystemVersion::Custom(ref version) => write!(f, "{}", version),
        }
    }
}

/// Parses a string as a semantic version.
///
/// A semantic version is a version in the format `major.minor.patch` where
/// `major`, `minor`, and `patch` are all non-negative integers. The version
/// is parsed left-to-right; if any part of the version is invalid, `None` is
/// returned.
///
/// # Examples
///
/// * `"1.2.3"` is parsed as `(1, 2, 3)`.
/// * `"1.2"` is not parsed at all and returns `None`.
/// * `"1.2.3.4"` is not parsed at all and returns `None`.
/// * `"1.2.3-alpha"` is not parsed at all and returns `None`.
///
/// # Return
///
/// * `Some(major, minor, patch)` if the version is parsed successfully.
/// * `None` if the version is not parsed successfully.
fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let mut parts = s.trim().split_terminator('.').fuse();

    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;

    if parts.next().is_some() {
        return None;
    }

    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that an empty string is parsed as `SystemVersion::Unknown`.
    ///
    /// This test case ensures that `SystemVersion::from_string` returns `SystemVersion::Unknown`
    /// when given an empty string.
    #[test]
    fn test_from_string_empty() {
        let version = SystemVersion::from_string("");
        assert_eq!(version, SystemVersion::Unknown);
    }

    /// Tests that a semantic version is parsed successfully.
    ///
    /// This test case ensures that `SystemVersion::from_string` returns a
    /// `SystemVersion::Semantic` instance when given a valid semantic version
    /// string.
    #[test]
    fn test_from_string_semantic() {
        let version = SystemVersion::from_string("1.2.3");
        assert_eq!(version, SystemVersion::Semantic(1, 2, 3));
    }

    /// Tests that a custom version is parsed successfully.
    ///
    /// This test case ensures that `SystemVersion::from_string` returns a
    /// `SystemVersion::Custom` instance when given a string that is not a valid
    /// semantic version.
    #[test]
    fn test_from_string_custom() {
        let version = SystemVersion::from_string("custom_version");
        assert_eq!(version, SystemVersion::Custom("custom_version".to_string()));
    }

    /// Tests that `SystemVersion::Unknown` is formatted as `"Unknown"`.
    ///
    /// This test case ensures that `Display` for `SystemVersion` returns the
    /// string `"Unknown"` when given `SystemVersion::Unknown`.
    #[test]
    fn test_display_unknown() {
        let version = SystemVersion::Unknown;
        assert_eq!(version.to_string(), "Unknown");
    }

    /// Tests that a semantic version is formatted as `"X.Y.Z"`.
    ///
    /// This test case ensures that `Display` for `SystemVersion` returns the
    /// string `"X.Y.Z"` when given a `SystemVersion::Semantic` instance.
    #[test]
    fn test_display_semantic() {
        let version = SystemVersion::Semantic(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    /// Tests that a rolling version with a codename is formatted as
    /// `"Rolling (codename)"`.
    ///
    /// This test case ensures that `Display` for `SystemVersion` returns the
    /// string `"Rolling (codename)"` when given a `SystemVersion::Rolling`
    /// instance with a codename.
    #[test]
    fn test_display_rolling_with_codename() {
        let version = SystemVersion::Rolling(Some("codename".to_string()));
        assert_eq!(version.to_string(), "Rolling (codename)");
    }

    /// Tests that a rolling version without a codename is formatted as `"Rolling"`.
    ///
    /// This test case ensures that `Display` for `SystemVersion` returns the
    /// string `"Rolling"` when given a `SystemVersion::Rolling` instance
    /// without a codename.
    #[test]
    fn test_display_rolling_without_codename() {
        let version = SystemVersion::Rolling(None);
        assert_eq!(version.to_string(), "Rolling");
    }
    /// Tests that a custom version is formatted correctly.
    ///
    /// This test case ensures that `Display` for `SystemVersion` returns the
    /// string representation of the custom version when given a
    /// `SystemVersion::Custom` instance.
    #[test]
    fn test_display_custom() {
        let version = SystemVersion::Custom("custom_version".to_string());
        assert_eq!(version.to_string(), "custom_version");
    }

    /// Tests that a valid semantic version string is parsed correctly.
    ///
    /// This test case ensures that `parse_version` returns `Some((1, 2, 3))`
    /// when given the string `"1.2.3"`, indicating that the version is parsed
    /// into its respective major, minor, and patch components.
    #[test]
    fn test_parse_version_valid() {
        let parsed = parse_version("1.2.3");
        assert_eq!(parsed, Some((1, 2, 3)));
    }

    /// Tests that an invalid semantic version string is not parsed.
    ///
    /// This test case ensures that `parse_version` returns `None` when given an
    /// invalid semantic version string, such as `"1.2"` (which is missing the
    /// patch component).
    #[test]
    fn test_parse_version_invalid() {
        let parsed = parse_version("1.2");
        assert_eq!(parsed, None);
    }

    /// Tests the `semantic()` constructor.
    ///
    /// This test ensures that `SystemVersion::semantic()` creates a valid
    /// semantic version and that it's equivalent to using the enum variant directly.
    #[test]
    fn test_constructor_semantic() {
        let version = SystemVersion::semantic(5, 15, 0);
        assert_eq!(version, SystemVersion::Semantic(5, 15, 0));
        assert_eq!(version.to_string(), "5.15.0");

        let version2 = SystemVersion::semantic(1, 2, 3);
        assert_eq!(version2.to_string(), "1.2.3");
    }

    /// Tests the `rolling()` constructor with a codename.
    ///
    /// This test ensures that `SystemVersion::rolling()` correctly creates
    /// rolling release versions with codenames.
    #[test]
    fn test_constructor_rolling_with_codename() {
        let version = SystemVersion::rolling(Some("2024.01"));
        assert_eq!(version, SystemVersion::Rolling(Some("2024.01".to_string())));
        assert_eq!(version.to_string(), "Rolling (2024.01)");

        let version2 = SystemVersion::rolling(Some(String::from("focal")));
        assert_eq!(version2.to_string(), "Rolling (focal)");
    }

    /// Tests the `rolling()` constructor without a codename.
    ///
    /// This test ensures that `SystemVersion::rolling()` correctly creates
    /// rolling release versions without codenames.
    #[test]
    fn test_constructor_rolling_without_codename() {
        let version = SystemVersion::rolling(None::<String>);
        assert_eq!(version, SystemVersion::Rolling(None));
        assert_eq!(version.to_string(), "Rolling");
    }

    /// Tests the `custom()` constructor.
    ///
    /// This test ensures that `SystemVersion::custom()` creates custom versions
    /// correctly and works with both &str and String.
    #[test]
    fn test_constructor_custom() {
        let version = SystemVersion::custom("NT 10.0");
        assert_eq!(version, SystemVersion::Custom("NT 10.0".to_string()));
        assert_eq!(version.to_string(), "NT 10.0");

        let version2 = SystemVersion::custom(String::from("2023.12-LTS"));
        assert_eq!(version2.to_string(), "2023.12-LTS");
    }

    /// Tests that constructors accept flexible string types.
    ///
    /// This test verifies that `impl Into<String>` works correctly,
    /// allowing both &str and String to be passed.
    #[test]
    fn test_constructor_string_flexibility() {
        let str_ref = "test";
        let version1 = SystemVersion::custom(str_ref);

        let owned_string = String::from("test");
        let version2 = SystemVersion::custom(owned_string);

        assert_eq!(version1, version2);

        let codename_str = "focal";
        let version3 = SystemVersion::rolling(Some(codename_str));

        let codename_owned = String::from("focal");
        let version4 = SystemVersion::rolling(Some(codename_owned));

        assert_eq!(version3, version4);
    }
}
