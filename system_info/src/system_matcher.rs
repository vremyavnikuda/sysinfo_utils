#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SystemMatcher {
    AllTrimmed,
    PrefixedWord { prefix: &'static str },
    PrefixedVersion { prefix: &'static str },
    KeyValue { key: &'static str },
}

impl SystemMatcher {
    fn find(&self, string: &str) -> Option<String> {
        match *self {
            Self::AllTrimmed => Some(string.trim().to_string()),
            Self::PrefixedWord { prefix } => {
                find_to_prefixed_word(string, prefix).map(str::to_owned)
            }
            Self::PrefixedVersion { prefix } => find_to_prefixed_word(string, prefix)
                .filter(|&v| is_valid_version(v))
                .map(str::to_owned),
            Self::KeyValue { key } => find_by_key(string, key).map(str::to_owned),
        }
    }
}

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
