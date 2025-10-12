//src/system_info.rs
#[derive(Debug, Clone)]
#[allow(dead_code)]
/// The `SystemMatcher` enum provides various strategies for searching and extracting data from strings.
///
/// It is used to parse strings, such as configuration files or system metadata,
/// to extract specific values, such as key-value pairs, versions, or strings with a certain prefix.
///
/// # Variants
///
/// - `AllTrimmed`
///     - Returns the input string with leading and trailing whitespace removed.
///
/// - `PrefixedWord`
///     - Extracts the word following a specified prefix.
///     - Example: For the input string `"prefix value"` and the prefix `"prefix"`, it will return `"value"`.
///
/// - `PrefixedVersion`
///     - Extracts the version number following a specified prefix.
///     - Ignores values that start or end with a dot (`.`).
///     - Example: For the input string `"version 1.2.3"`, it will return `"1.2.3"`.
///
/// - `KeyValue`
///     - Extracts the value for a given key in a key-value format (`key=value`).
///     - Removes surrounding quotes from the value, if present.
///     - Example: For the input string `"key=\"value\""`, it will return `"value"`.
///
pub enum SystemMatcher {
    /// Trims leading and trailing whitespace from the string.
    AllTrimmed,
    /// Finds the word following a specified prefix.
    ///
    /// # Fields
    ///
    /// - `prefix`: The prefix string to search for.
    PrefixedWord {
        /// The prefix to search for in the string.
        prefix: &'static str,
    },
    /// Finds a version number following a specified prefix.
    ///
    /// Skips invalid version formats.
    ///
    /// # Fields
    ///
    /// - `prefix`: The prefix string to search for.
    PrefixedVersion {
        /// The prefix to search for in the string.
        prefix: &'static str,
    },
    /// Finds the value associated with a given key in a key-value pair.
    ///
    /// # Fields
    ///
    /// - `key`: The key to search for in the key-value pair.
    KeyValue {
        /// The key to search for in the string.
        key: &'static str,
    },
}

impl SystemMatcher {
    /// Searches for a specific value in the given string based on the `SystemMatcher` variant.
    ///
    /// # Arguments
    ///
    /// * `string` - The input string to search within.
    ///
    /// # Returns
    ///
    /// Returns an `Option<String>` containing the matched value, or `None` if no match is found.
    ///
    /// # Variants Behavior
    ///
    /// - **AllTrimmed**: Trims leading and trailing whitespace from the input string.
    /// - **PrefixedWord**: Finds the word following the specified prefix.
    /// - **PrefixedVersion**: Finds the version following the specified prefix, skipping invalid formats.
    /// - **KeyValue**: Extracts the value associated with a key in the `key=value` format.
    ///
    pub fn find(&self, string: &str) -> Option<String> {
        match *self {
            Self::AllTrimmed => Some(string.trim().to_string()),
            Self::PrefixedWord { prefix } => {
                find_to_prefixed_word(string, prefix).map(str::to_owned)
            }
            Self::PrefixedVersion { prefix } => find_to_prefixed_word(string, prefix)
                .filter(|&version| is_valid_version(version))
                .map(str::to_owned),
            Self::KeyValue { key } => find_by_key(string, key).map(str::to_owned),
        }
    }
}

/// Concatenates the given `key` with an equals sign (`=`).
///
/// # Arguments
///
/// * `key` - A string slice that holds the key to be concatenated.
///
/// # Returns
///
/// A init `String` that is the result of concatenating the `key` with an equals sign.
fn find_by_key<'a>(string: &'a str, key: &str) -> Option<&'a str> {
    let key = [key, "="].concat();
    for line in string.lines() {
        if line.starts_with(&key) {
            return Some(line[key.len()..].trim_matches(|c: char| c == '"' || c.is_whitespace()));
        }
    }
    None
}

fn is_valid_version(word: &str) -> bool {
    !word.starts_with('.') && !word.ends_with('.')
}

/// Finds the position of the first whitespace character in the given string.
/// If no whitespace is found, returns the length of the string.
///
/// # Parameters
/// - `string`: The input string to search for whitespace.
///
/// # Returns
/// - `usize`: The position of the first whitespace character, or the length of the string if no whitespace is found.
fn find_to_prefixed_word<'a>(string: &'a str, prefix: &str) -> Option<&'a str> {
    if let Some(prefix_start) = string.find(prefix) {
        let string = &string[prefix_start + prefix.len()..].trim_start();

        let word_end = string
            .find(|c: char| c.is_whitespace())
            .unwrap_or(string.len());
        let string = &string[..word_end];

        Some(string)
    } else {
        None
    }
}

#[cfg(test)]
mod system_matcher_tests {
    use super::SystemMatcher;
    use pretty_assertions::assert_eq;

    #[test]
    fn trimmed() {
        let data = [
            ("", Some("")),
            ("test", Some("test")),
            (" 		 test", Some("test")),
            ("test  	   ", Some("test")),
            ("  test 	", Some("test")),
        ];

        let matcher = SystemMatcher::AllTrimmed;

        for (input, expected) in &data {
            let result = matcher.find(input);
            assert_eq!(result.as_deref(), *expected);
        }
    }

    #[test]
    fn prefixed_word() {
        let data = [
            ("", None),
            ("test", Some("")),
            ("test1", Some("1")),
            ("test 1", Some("1")),
            (" test 1", Some("1")),
            ("test 1.2.3", Some("1.2.3")),
            (" 		test 1.2.3", Some("1.2.3")),
        ];

        let matcher = SystemMatcher::PrefixedWord { prefix: "test" };

        for (input, expected) in &data {
            let result = matcher.find(input);
            assert_eq!(result.as_deref(), *expected);
        }
    }

    #[test]
    fn prefixed_version() {
        let data = [
            ("", None),
            ("test", Some("")),
            ("test 1", Some("1")),
            ("test .1", None),
            ("test 1.", None),
            ("test .1.", None),
            (" test 1", Some("1")),
            ("test 1.2.3", Some("1.2.3")),
            (" 		test 1.2.3", Some("1.2.3")),
        ];

        let matcher = SystemMatcher::PrefixedVersion { prefix: "test" };

        for (input, expected) in &data {
            let result = matcher.find(input);
            assert_eq!(result.as_deref(), *expected);
        }
    }

    #[test]
    fn key_value() {
        let data = [
            ("", None),
            ("key", None),
            ("key=value", Some("value")),
            ("key=1", Some("1")),
            ("key=\"1\"", Some("1")),
            ("key=\"CentOS Linux\"", Some("CentOS Linux")),
        ];

        let matcher = SystemMatcher::KeyValue { key: "key" };

        for (input, expected) in &data {
            let result = matcher.find(input);
            assert_eq!(result.as_deref(), *expected);
        }
    }
}
