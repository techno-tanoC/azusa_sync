use indexmap::IndexMap;
use std::sync::{Arc, Mutex};
use std::ops::Drop;

#[derive(Clone)]
pub struct Table<K, V>(Arc<Mutex<IndexMap<K, V>>>);

impl<V: Clone> Table<String, V> {
    pub fn new() -> Self {
        Table(Arc::new(Mutex::new(IndexMap::new())))
    }

    pub fn add(&self, v: V) -> AddDrop<'_, V> {
        let id = Self::generate_id();
        self.0.lock().unwrap().insert(id.to_string(), v);
        AddDrop {
            id: id.clone(),
            table: &self,
        }
    }

    pub fn to_vec(&self) -> Vec<(String, V)> {
        self.0.lock().unwrap().iter().map(|(k, v)| {
            (k.clone(), v.clone())
        }).collect()
    }
}

impl<V> Table<String, V> {
    fn delete(&self, id: impl ToString) {
        self.0.lock().unwrap().remove(&id.to_string());
    }

    fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

pub struct AddDrop<'a, V> {
    id: String,
    table: &'a Table<String, V>,
}

impl<'a, V> Drop for AddDrop<'a, V> {
    fn drop(&mut self) {
        self.table.delete(&self.id);
    }
}
