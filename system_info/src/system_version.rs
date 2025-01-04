use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Operating system version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SystemVersion {
    Unknown,
    Semantic(u64, u64, u64),
    Rolling(Option<String>),
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
                if let &Some(ref codename) = codename {
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
    /// Tests that a rolling version without a codename is formatted as `"Rolling"`.
    ///
    /// This test case ensures that the `Display` trait for `SystemVersion` returns
    /// the string `"Rolling"` when given a `SystemVersion::Rolling` instance
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
}
