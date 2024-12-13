use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

#[derive(Default)]
pub enum Matcher {
    #[default]
    Substring,
    Fuzzy(Box<SkimMatcherV2>),
}

impl Matcher {
    pub fn substring() -> Self {
        Matcher::Substring
    }

    pub fn fuzzy() -> Self {
        Matcher::Fuzzy(Box::default())
    }

    pub fn match_indices(&self, text: &str, pattern: &str) -> Option<Vec<usize>> {
        match self {
            Matcher::Substring => text
                .find(pattern)
                .map(|pos| (pos..pos + pattern.len()).collect()),
            Matcher::Fuzzy(matcher) => matcher
                .fuzzy_indices(text, pattern)
                .map(|(_, indices)| indices),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matcher_substring() {
        let matcher = Matcher::substring();
        assert_eq!(matcher.match_indices("hello", "he"), Some(vec![0, 1]));
        assert_eq!(matcher.match_indices("hello", "lo"), Some(vec![3, 4]));
        assert_eq!(matcher.match_indices("hello", "ho"), None);
        assert_eq!(matcher.match_indices("hello", "wr"), None);
    }

    #[test]
    fn test_matcher_fuzzy() {
        let matcher = Matcher::fuzzy();
        assert_eq!(matcher.match_indices("hello", "he"), Some(vec![0, 1]));
        assert_eq!(matcher.match_indices("hello", "lo"), Some(vec![3, 4]));
        assert_eq!(matcher.match_indices("hello", "ho"), Some(vec![0, 4]));
        assert_eq!(matcher.match_indices("hello", "wr"), None);
    }
}
