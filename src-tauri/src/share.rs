use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Share {
    pub token: String,
    pub file_path: PathBuf,
    pub name: String,
    pub size: u64,
    pub password_hash: Option<String>,
    pub download_count: u64,
    pub created_at: SystemTime,
}

impl Share {
    /// Creates a new share with download_count 0 and created_at now.
    pub fn new(
        token: String,
        file_path: PathBuf,
        name: String,
        size: u64,
        password_hash: Option<String>,
    ) -> Self {
        Share {
            token,
            file_path,
            name,
            size,
            password_hash,
            download_count: 0,
            created_at: SystemTime::now(),
        }
    }
}

#[derive(Default)]
pub struct ShareRegistry {
    shares: HashMap<String, Share>,
}

impl ShareRegistry {
    pub fn new() -> Self {
        ShareRegistry { shares: HashMap::new() }
    }

    pub fn insert(&mut self, share: Share) {
        self.shares.insert(share.token.clone(), share);
    }

    pub fn get(&self, token: &str) -> Option<&Share> {
        self.shares.get(token)
    }

    pub fn get_mut(&mut self, token: &str) -> Option<&mut Share> {
        self.shares.get_mut(token)
    }

    pub fn remove(&mut self, token: &str) -> Option<Share> {
        self.shares.remove(token)
    }

    pub fn list(&self) -> Vec<Share> {
        self.shares.values().cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.shares.is_empty()
    }

    pub fn len(&self) -> usize {
        self.shares.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(token: &str) -> Share {
        Share::new(token.to_string(), PathBuf::from("/tmp/x"), "x".into(), 10, None)
    }

    #[test]
    fn insert_and_get() {
        let mut r = ShareRegistry::new();
        r.insert(sample("abc"));
        assert_eq!(r.get("abc").unwrap().name, "x");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn remove_makes_it_gone() {
        let mut r = ShareRegistry::new();
        r.insert(sample("abc"));
        assert!(r.remove("abc").is_some());
        assert!(r.get("abc").is_none());
        assert!(r.is_empty());
    }

    #[test]
    fn increment_download_count_via_get_mut() {
        let mut r = ShareRegistry::new();
        r.insert(sample("abc"));
        r.get_mut("abc").unwrap().download_count += 1;
        assert_eq!(r.get("abc").unwrap().download_count, 1);
    }

    #[test]
    fn list_returns_all() {
        let mut r = ShareRegistry::new();
        r.insert(sample("a"));
        r.insert(sample("b"));
        assert_eq!(r.list().len(), 2);
    }
}
