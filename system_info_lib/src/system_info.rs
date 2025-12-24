//src/system_info.rs
use crate::bit_depth::BitDepth;
use crate::system_os::Type;
use crate::SystemVersion;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Contains comprehensive information about an operating system.
///
/// Provides details such as system type, version, edition, codename,
/// bit depth, and architecture.
pub struct Info {
    /// The type of the operating system.
    pub(crate) system_type: Type,

    /// The version of the operating system.
    pub(crate) version: SystemVersion,

    /// The edition of the operating system, if known.
    pub(crate) edition: Option<String>,

    /// The codename of the operating system, if known.
    pub(crate) codename: Option<String>,

    /// The bit depth of the operating system.
    pub(crate) bit_depth: BitDepth,

    /// The architecture of the operating system, if known.
    pub(crate) architecture: Option<String>,

    /// The kernel version of the operating system, if known.
    pub(crate) kernel_version: Option<String>,
}

impl Info {
    /// Returns an `Info` instance with all fields set to their unknown or default values.
    ///
    /// # Returns
    ///
    /// * `Info` - An instance of `Info` with fields:
    ///     - `system_type`: `Type::Unknown`
    ///     - `version`: `Version::unknown()`
    ///     - `edition`: `None`
    ///     - `codename`: `None`
    ///     - `bit_depth`: `BitDepth::Unknown`
    ///     - `architecture`: `None`
    ///     - `kernel_version`: `None`
    pub fn unknown() -> Self {
        Self {
            system_type: Type::Unknown,
            version: SystemVersion::Unknown,
            edition: None,
            codename: None,
            bit_depth: BitDepth::Unknown,
            architecture: None,
            kernel_version: None,
        }
    }

    /// Creates a init instance with the specified system type, using default values for other fields.
    ///
    /// # Arguments
    ///
    /// * `system_type` - The type of system to be set
    ///
    /// # Returns
    ///
    /// A init instance of the struct with the given system type
    pub fn with_type(system_type: Type) -> Self {
        Self {
            system_type,
            ..Default::default()
        }
    }
    /// Returns the system type.
    ///
    /// # Returns
    /// Returns the version of the OS.
    ///
    /// # Returns
    ///
    /// * `&Version` - The version of the OS.
    ///
    /// * `Type` - The system type.
    pub fn system_type(&self) -> Type {
        self.system_type
    }

    /// Returns the version of the OS.
    ///
    /// # Returns
    ///
    /// * `&Version` - The version of the OS.
    pub fn version(&self) -> &SystemVersion {
        &self.version
    }

    /// Returns the edition of the OS.
    ///
    /// # Returns
    ///
    /// * `Option<&str>` - The edition of the OS, if known.
    pub fn edition(&self) -> Option<&str> {
        self.edition.as_ref().map(String::as_ref)
    }

    /// Returns the codename of the OS.
    ///
    /// # Returns
    ///
    /// * `Option<&str>` - The codename of the OS, if known.
    pub fn codename(&self) -> Option<&str> {
        self.codename.as_ref().map(String::as_ref)
    }

    /// Returns the bit depth of the OS.
    ///
    /// # Returns
    ///
    /// * `BitDepth` - The bit depth of the OS.
    pub fn bit_depth(&self) -> BitDepth {
        self.bit_depth
    }

    /// Returns the architecture of the OS.
    ///
    /// # Returns
    ///
    /// * `Option<&str>` - The architecture of the OS, if known.
    pub fn architecture(&self) -> Option<&str> {
        self.architecture.as_ref().map(String::as_ref)
    }

    /// Returns the kernel version of the OS.
    ///
    /// # Returns
    ///
    /// * `Option<&str>` - The kernel version of the OS, if known.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::get;
    ///
    /// let info = get();
    /// if let Some(kernel) = info.kernel_version() {
    ///     println!("Kernel version: {}", kernel);
    /// }
    /// ```
    pub fn kernel_version(&self) -> Option<&str> {
        self.kernel_version.as_ref().map(String::as_ref)
    }

    /// Creates a new [`InfoBuilder`] for constructing an `Info` instance.
    ///
    /// # Returns
    ///
    /// A new `InfoBuilder` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, Type, SystemVersion, BitDepth};
    ///
    /// let info = Info::builder()
    ///     .system_type(Type::Linux)
    ///     .version(SystemVersion::Semantic(5, 15, 0))
    ///     .bit_depth(BitDepth::X64)
    ///     .build();
    /// ```
    pub fn builder() -> InfoBuilder {
        InfoBuilder::new()
    }
}

impl Default for Info {
    /// Creates a init `Info` instance with all fields set to their unknown or default values.
    ///
    /// # Returns
    ///
    /// * `Info` - An instance of `Info` with fields:
    ///     - `system_type`: `Type::Unknown`
    ///     - `version`: `Version::unknown()`
    ///     - `edition`: `None`
    ///     - `codename`: `None`
    ///     - `bit_depth`: `BitDepth::Unknown`
    ///     - `architecture`: `None`
    ///     - `kernel_version`: `None`
    fn default() -> Self {
        Self::unknown()
    }
}

impl Display for Info {
    /// Formats the value using the given formatter.
    ///
    /// The format is as follows:
    ///
    /// `<system_type> [<edition>] (<codename>) <version>, <bit_depth>, <architecture>`
    ///
    /// Where:
    ///
    /// - `<system_type>` is the type of the operating system
    /// - `<edition>` is the edition of the operating system, if known
    /// - `<codename>` is the codename of the operating system, if known
    /// - `<version>` is the version of the operating system
    /// - `<bit_depth>` is the bit depth of the operating system, if known
    /// - `<architecture>` is the architecture of the operating system, if known
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.system_type)?;
        if let Some(edition) = &self.edition {
            write!(f, " {}", edition)?;
        }
        if let Some(codename) = &self.codename {
            write!(f, " ({})", codename)?;
        }
        write!(f, " {}", self.version)?;
        if self.bit_depth != BitDepth::Unknown {
            write!(f, ", {}", self.bit_depth)?;
        }
        if let Some(architecture) = &self.architecture {
            write!(f, ", {}", architecture)?;
        }
        Ok(())
    }
}

/// Builder for constructing [`Info`] instances with method chaining.
///
/// Provides an ergonomic API for building system information objects.
/// All fields are optional and will default to unknown values if not set.
///
/// # Examples
///
/// ```
/// use system_info_lib::{Info, Type, SystemVersion, BitDepth};
///
/// let info = Info::builder()
///     .system_type(Type::Linux)
///     .version(SystemVersion::Semantic(5, 15, 0))
///     .edition("Pro")
///     .codename("Focal")
///     .bit_depth(BitDepth::X64)
///     .architecture("x86_64")
///     .build();
///
/// assert_eq!(info.system_type(), Type::Linux);
/// assert_eq!(info.edition(), Some("Pro"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct InfoBuilder {
    system_type: Option<Type>,
    version: Option<SystemVersion>,
    edition: Option<String>,
    codename: Option<String>,
    bit_depth: Option<BitDepth>,
    architecture: Option<String>,
    kernel_version: Option<String>,
}

impl InfoBuilder {
    /// Creates a new empty builder.
    ///
    /// # Returns
    ///
    /// A new `InfoBuilder` instance with all fields set to `None`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the system type.
    ///
    /// # Arguments
    ///
    /// * `system_type` - The type of the operating system.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn system_type(mut self, system_type: Type) -> Self {
        self.system_type = Some(system_type);
        self
    }

    /// Sets the system version.
    ///
    /// # Arguments
    ///
    /// * `version` - The version of the operating system.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn version(mut self, version: SystemVersion) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets the system edition.
    ///
    /// # Arguments
    ///
    /// * `edition` - The edition of the operating system.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn edition(mut self, edition: impl Into<String>) -> Self {
        self.edition = Some(edition.into());
        self
    }

    /// Sets the system codename.
    ///
    /// # Arguments
    ///
    /// * `codename` - The codename of the operating system.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn codename(mut self, codename: impl Into<String>) -> Self {
        self.codename = Some(codename.into());
        self
    }

    /// Sets the bit depth.
    ///
    /// # Arguments
    ///
    /// * `bit_depth` - The bit depth of the operating system.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn bit_depth(mut self, bit_depth: BitDepth) -> Self {
        self.bit_depth = Some(bit_depth);
        self
    }

    /// Sets the system architecture.
    ///
    /// # Arguments
    ///
    /// * `architecture` - The architecture of the operating system.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn architecture(mut self, architecture: impl Into<String>) -> Self {
        self.architecture = Some(architecture.into());
        self
    }

    /// Sets the kernel version.
    ///
    /// # Arguments
    ///
    /// * `kernel_version` - The kernel version of the operating system.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::Info;
    ///
    /// let info = Info::builder()
    ///     .kernel_version("5.15.0-76-generic")
    ///     .build();
    ///
    /// assert_eq!(info.kernel_version(), Some("5.15.0-76-generic"));
    /// ```
    pub fn kernel_version(mut self, kernel_version: impl Into<String>) -> Self {
        self.kernel_version = Some(kernel_version.into());
        self
    }

    /// Builds the [`Info`] instance.
    ///
    /// All unset fields will default to their unknown values:
    /// - `system_type`: `Type::Unknown`
    /// - `version`: `SystemVersion::Unknown`
    /// - `edition`: `None`
    /// - `codename`: `None`
    /// - `bit_depth`: `BitDepth::Unknown`
    /// - `architecture`: `None`
    /// - `kernel_version`: `None`
    ///
    /// # Returns
    ///
    /// A new `Info` instance.
    pub fn build(self) -> Info {
        Info {
            system_type: self.system_type.unwrap_or(Type::Unknown),
            version: self.version.unwrap_or(SystemVersion::Unknown),
            edition: self.edition,
            codename: self.codename,
            bit_depth: self.bit_depth.unwrap_or(BitDepth::Unknown),
            architecture: self.architecture,
            kernel_version: self.kernel_version,
        }
    }

    /// Builds the [`Info`] instance with validation.
    ///
    /// This method builds the `Info` instance. Since all system information
    /// values are inherently valid, this method always succeeds. It is provided
    /// for API consistency with other builders in the workspace.
    ///
    /// # Returns
    ///
    /// * `Ok(Info)` - The built `Info` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use system_info_lib::{Info, Type, SystemVersion, BitDepth};
    ///
    /// let info = Info::builder()
    ///     .system_type(Type::Linux)
    ///     .version(SystemVersion::Semantic(5, 15, 0))
    ///     .try_build();
    ///
    /// assert!(info.is_ok());
    /// ```
    pub fn try_build(self) -> Result<Info, std::convert::Infallible> {
        Ok(self.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_builder_all_fields() {
        let info = Info::builder()
            .system_type(Type::Linux)
            .version(SystemVersion::Semantic(5, 15, 0))
            .edition("Pro")
            .codename("Focal")
            .bit_depth(BitDepth::X64)
            .architecture("x86_64")
            .build();

        assert_eq!(info.system_type(), Type::Linux);
        assert_eq!(info.version(), &SystemVersion::Semantic(5, 15, 0));
        assert_eq!(info.edition(), Some("Pro"));
        assert_eq!(info.codename(), Some("Focal"));
        assert_eq!(info.bit_depth(), BitDepth::X64);
        assert_eq!(info.architecture(), Some("x86_64"));
    }

    #[test]
    fn test_builder_minimal() {
        let info = Info::builder().system_type(Type::Linux).build();

        assert_eq!(info.system_type(), Type::Linux);
        assert_eq!(info.version(), &SystemVersion::Unknown);
        assert_eq!(info.edition(), None);
        assert_eq!(info.codename(), None);
        assert_eq!(info.bit_depth(), BitDepth::Unknown);
        assert_eq!(info.architecture(), None);
    }

    #[test]
    fn test_builder_defaults() {
        let info = Info::builder().build();

        assert_eq!(info.system_type(), Type::Unknown);
        assert_eq!(info.version(), &SystemVersion::Unknown);
        assert_eq!(info.edition(), None);
        assert_eq!(info.codename(), None);
        assert_eq!(info.bit_depth(), BitDepth::Unknown);
        assert_eq!(info.architecture(), None);
    }

    #[test]
    fn test_builder_partial() {
        let info = Info::builder()
            .system_type(Type::Windows)
            .edition("Home")
            .bit_depth(BitDepth::X64)
            .build();

        assert_eq!(info.system_type(), Type::Windows);
        assert_eq!(info.version(), &SystemVersion::Unknown);
        assert_eq!(info.edition(), Some("Home"));
        assert_eq!(info.codename(), None);
        assert_eq!(info.bit_depth(), BitDepth::X64);
        assert_eq!(info.architecture(), None);
    }

    #[test]
    fn test_builder_string_conversions() {
        let edition_string = String::from("Enterprise");
        let codename_str = "Buster";
        let arch_string = String::from("aarch64");

        let info = Info::builder()
            .edition(edition_string)
            .codename(codename_str)
            .architecture(arch_string)
            .build();

        assert_eq!(info.edition(), Some("Enterprise"));
        assert_eq!(info.codename(), Some("Buster"));
        assert_eq!(info.architecture(), Some("aarch64"));
    }

    #[test]
    fn test_builder_method_chaining() {
        let builder = InfoBuilder::new();
        let builder = builder.system_type(Type::Macos);
        let builder = builder.version(SystemVersion::Semantic(12, 0, 0));
        let info = builder.build();

        assert_eq!(info.system_type(), Type::Macos);
        assert_eq!(info.version(), &SystemVersion::Semantic(12, 0, 0));
    }

    #[test]
    fn test_builder_with_kernel_version() {
        let info = Info::builder()
            .system_type(Type::Linux)
            .kernel_version("5.15.0-76-generic")
            .build();

        assert_eq!(info.kernel_version(), Some("5.15.0-76-generic"));
    }

    #[test]
    fn test_builder_kernel_version_string_conversions() {
        let kernel_string = String::from("6.1.0-13-amd64");
        let info = Info::builder().kernel_version(kernel_string).build();

        assert_eq!(info.kernel_version(), Some("6.1.0-13-amd64"));
    }

    #[test]
    fn test_info_unknown_has_no_kernel_version() {
        let info = Info::unknown();
        assert_eq!(info.kernel_version(), None);
    }

    mod proptest_tests {
        use super::{BitDepth, Info, InfoBuilder, SystemVersion, Type};
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_builder_with_any_strings(
                edition in proptest::option::of("\\PC+"),
                codename in proptest::option::of("\\PC+"),
                architecture in proptest::option::of("\\PC+")
            ) {
                let builder = InfoBuilder::new()
                    .system_type(Type::Linux)
                    .bit_depth(BitDepth::X64);

                let builder = if let Some(e) = &edition {
                    builder.edition(e.as_str())
                } else {
                    builder
                };

                let builder = if let Some(c) = &codename {
                    builder.codename(c.as_str())
                } else {
                    builder
                };

                let builder = if let Some(a) = &architecture {
                    builder.architecture(a.as_str())
                } else {
                    builder
                };

                let info = builder.build();

                assert_eq!(info.system_type(), Type::Linux);
                assert_eq!(info.bit_depth(), BitDepth::X64);
                assert_eq!(info.edition(), edition.as_deref());
                assert_eq!(info.codename(), codename.as_deref());
                assert_eq!(info.architecture(), architecture.as_deref());
            }

            #[test]
            fn test_builder_clone_equals_original(major in 0u64..100, minor in 0u64..100, patch in 0u64..100) {
                let info1 = Info::builder()
                    .system_type(Type::Ubuntu)
                    .version(SystemVersion::semantic(major, minor, patch))
                    .bit_depth(BitDepth::X64)
                    .build();

                let info2 = info1.clone();

                assert_eq!(info1, info2);
                assert_eq!(info1.system_type(), info2.system_type());
                assert_eq!(info1.version(), info2.version());
                assert_eq!(info1.bit_depth(), info2.bit_depth());
            }

            #[test]
            fn test_info_display_never_panics(
                edition in proptest::option::of("\\PC{0,50}"),
                codename in proptest::option::of("\\PC{0,50}")
            ) {
                let builder = Info::builder()
                    .system_type(Type::Fedora)
                    .version(SystemVersion::semantic(39, 0, 0))
                    .bit_depth(BitDepth::X64);

                let builder = if let Some(e) = edition {
                    builder.edition(e)
                } else {
                    builder
                };

                let builder = if let Some(c) = codename {
                    builder.codename(c)
                } else {
                    builder
                };

                let info = builder.build();
                let display = info.to_string();

                assert!(!display.is_empty());
                assert!(display.contains("Fedora"));
            }
        }
    }
}
