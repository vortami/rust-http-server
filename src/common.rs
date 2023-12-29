use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::Display,
    hash::Hash,
    ops::{Deref, DerefMut},
    str::FromStr,
};

#[allow(clippy::derive_ord_xor_partial_ord)]
#[derive(Clone, Eq, Ord)]
pub struct HeaderKey(String);

impl PartialEq for HeaderKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase().eq(&other.0.to_lowercase())
    }
}

impl PartialOrd for HeaderKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.to_lowercase().partial_cmp(&other.to_lowercase())
    }
}

impl Deref for HeaderKey {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for HeaderKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_lowercase().hash(state)
    }
}

impl<T: ToString> From<T> for HeaderKey {
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

pub struct Headers(HashMap<HeaderKey, String>);

impl Headers {
    pub fn get(&self, key: impl Into<HeaderKey>) -> Option<&String> {
        self.0.get(&key.into())
    }

    pub fn builder() -> HeadersBuilder {
        HeadersBuilder::new()
    }
}

impl<K: ToString, V: ToString> From<HashMap<K, V>> for Headers {
    fn from(value: HashMap<K, V>) -> Self {
        Self(
            value
                .iter()
                .map(|(k, v)| (k.to_string().into(), v.to_string()))
                .collect(),
        )
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .iter()
            .try_for_each(|(k, v)| writeln!(f, "{}: {}", **k, v))
    }
}

pub struct HeadersBuilder(HashMap<HeaderKey, String>);
impl HeadersBuilder {
    fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn set(mut self, key: impl Into<HeaderKey>, value: impl ToString) -> Self {
        self.0.insert(key.into(), value.to_string());
        self
    }

    pub fn build(self) -> Headers {
        Headers(self.0)
    }
}

impl Deref for HeadersBuilder {
    type Target = HashMap<HeaderKey, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HeadersBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub enum Method {
    Get,
    Post,
    Other(String),
}

impl FromStr for Method {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "GET" => Self::Get,
            "POST" => Self::Post,
            other => Self::Other(other.to_string()),
        })
    }
}

#[derive(Debug, Default)]
pub struct Search(HashMap<String, String>);

impl FromStr for Search {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = HashMap::new();
        for item in s.split('&') {
            let (l, r) = match item.split_once('=') {
                Some((l, r)) => (l.to_string(), r.to_string()),
                None => return Err(()),
            };
            map.insert(l, r);
        }
        Ok(Self(map))
    }
}

impl Deref for Search {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
