use std::{collections::HashMap, ops::{Deref, DerefMut}, sync::Arc};

pub struct Manager<T> {
    data: HashMap<String, Arc<T>>,
}

impl<T> Manager<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }

    pub fn get(&self, name: &str) -> Option<Arc<T>> {
        self.data.get(name).cloned()
    }

    pub fn get_or_init(&mut self, name: &str, init: impl FnOnce() -> Arc<T>) -> Arc<T> {
        self.data
            .entry(name.to_string())
            .or_insert_with(init)
            .clone()
    }

    pub fn insert(&mut self, name: impl Into<String>, value: T) -> Option<Arc<T>> {
        self.data.insert(name.into(), Arc::new(value))
    }

    pub fn insert_rc(&mut self, name: impl Into<String>, value: Arc<T>) -> Option<Arc<T>> {
        self.data.insert(name.into(), value)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.data.contains_key(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Arc<T>> {
        self.data.remove(name)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn update(&mut self, name: &str, value: T) -> Option<Arc<T>> {
        if self.contains(name) {
            self.insert(name.to_string(), value)
        } else {
            None
        }
    }
}

impl<T> Deref for Manager<T> {
    type Target = HashMap<String, Arc<T>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Manager<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> Default for Manager<T> {
    fn default() -> Self {
        Self::new()
    }
}

