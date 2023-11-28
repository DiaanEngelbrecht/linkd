use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use crate::Registerable;

pub type MapValue = Arc<dyn Any + Send + Sync>;

pub struct Registry {
    entries: HashMap<TypeId, MapValue>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, type_: TypeId, child: MapValue) {
        self.entries.insert(type_, child);
    }

    pub fn get_child(&self, type_: TypeId) -> Option<&MapValue> {
        self.entries.get(&type_)
    }
}
