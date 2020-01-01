use serde::{
    Serialize,
    Deserialize
};

#[derive(Serialize, Deserialize)]
pub struct Dictionary<K, V> {
    categories: Vec<Category<K, V>>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Category<K, V> {
    key: K,
    values: Vec<V>
}

impl<K: Eq + std::fmt::Debug + Clone, V: Clone> Dictionary<K, V> {
    pub fn new() -> Dictionary<K, V> {
        Dictionary {
            categories: Vec::new()
        }
    }

    pub fn push(&mut self, target_key: K, value: V) {
        if self.contains_key(|key| key == &target_key) {
            for category in self.categories.iter_mut() {
                if category.is(&target_key) {
                    category.push(value);
                    break;
                }
            }
        } else {
            self.categories.push(Category::new(target_key, vec![value]));
        }
    }

    pub fn pop<FK, FV>(&mut self, key_predicate: FK, value_predicate: FV) -> Vec<V> where
        FK: Fn(&K) -> bool,
        FV: Fn(&V) -> bool {
        let mut values: Vec<V> = vec![];
        let mut indices: Vec<usize> = vec![];
        let mut shift_factor: usize = 0;

        for (index, category) in self.categories.iter_mut().enumerate() {
            if key_predicate(category.key()) {
                values.extend(category.pop(&value_predicate));
                indices.push(index);
            }
        }

        indices.iter().for_each(|index| {
            self.categories.remove(index - shift_factor);
            shift_factor += 1;
        });

        values
    }

    pub fn values<F>(&self, predicate: F) -> Vec<V> where
        F: Fn(&K) -> bool {
        let mut result: Vec<V> = vec![];

        for category in self.categories.iter() {
            if predicate(category.key()) {
                result.extend(category.values());
            }
        }

        result
    }

    pub fn contains_key<F>(&self, predicate: F) -> bool where
        F: Fn(&K) -> bool {
        self.categories.iter().any(|category| predicate(category.key()))
    }
}

impl <K: Eq + std::fmt::Debug + Clone, V: Clone> Category<K, V> {
    pub fn new(key: K, values: Vec<V>) -> Category<K, V> {
        Category {
            key,
            values
        }
    }

    pub fn push(&mut self, value: V) {
        self.values.push(value)
    }

    pub fn pop<F>(&mut self, predicate: F) -> Vec<V> where
        F: Fn(&V) -> bool {
        let mut indices: Vec<usize> = vec![];
        let mut shift: usize = 0;

        for (index, value) in self.values.iter_mut().enumerate() {
            if predicate(&value) {
                indices.push(index);
            }
        }

        indices.iter().map(|index| {
            let result = self.values.remove(index - shift);
            shift += 1;
            result
        }).collect()
    }

    pub fn values(&self) -> Vec<V> {
        self.values.clone()
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn is(&self, key: &K) -> bool {
        key == &self.key
    }
}