#[derive(Debug, Clone)]
#[allow(dead_code)]

pub enum SystemMatcher {
    AllTrimmed,
    PrefixedWord { prefix: &'static str },
    PrefixedVersion { prefix: &'static str },
    KeyValue { key: &'static str },
}



//FIXME: Implement the SystemMatcher trait for the SystemMatcher enum.
impl SystemMatcher {
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
/// A new `String` that is the result of concatenating the `key` with an equals sign.
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
mod tests {
    use super::*;
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
