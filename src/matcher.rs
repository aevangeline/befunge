// Implements a convenience wrapper around regex
use std::hash::{Hash, Hasher};
use std::ops::Deref;
extern crate regex;


pub struct Matcher {
    r: regex::Regex,
}

impl PartialEq for Matcher {
    fn eq(&self, other: &Matcher) -> bool {
        self.r.as_str() == other.r.as_str()
    }
}

impl Eq for Matcher {}

impl Hash for Matcher {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.as_str().hash(state);
    }
}

impl Deref for Matcher {
    type Target = regex::Regex;

    fn deref(&self) -> &regex::Regex {
        return &self.r;
    }
}

impl From<regex::Regex> for Matcher {
    fn from(rgx: regex::Regex) -> Self {
        Matcher { r: rgx }
    }
}

impl From<Matcher> for regex::Regex {
    fn from(m: Matcher) -> Self {
        m.r
    }
}
