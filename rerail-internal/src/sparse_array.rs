use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Index, IndexMut},
};
use tsify::Tsify;

use std::marker::PhantomData;

#[derive(Serialize, Deserialize)]
pub struct SparseArray<T> {
    data: Vec<(SparseArrayId<T>, T)>,
    id_to_index: HashMap<SparseArrayId<T>, usize>,
    unused_indices: Vec<usize>,
}

impl<T> SparseArray<T> {
    pub fn new() -> SparseArray<T> {
        SparseArray {
            data: vec![],
            id_to_index: HashMap::new(),
            unused_indices: vec![],
        }
    }

    pub fn push(&mut self, item: T) -> SparseArrayId<T> {
        let id = SparseArrayId(
            self.unused_indices.pop().unwrap_or(self.data.len()),
            PhantomData,
        );
        let idx = self.data.len();
        assert!(self.id_to_index.insert(id, idx).is_none());
        self.data.push((id, item));
        id
    }

    pub fn get(&self, id: SparseArrayId<T>) -> Option<&T> {
        let idx = self.id_to_index.get(&id)?;
        Some(&self.data[*idx].1)
    }

    pub fn get_mut(&mut self, id: SparseArrayId<T>) -> Option<&mut T> {
        let idx = self.id_to_index.get(&id)?;
        Some(&mut self.data[*idx].1)
    }

    #[allow(unused)]
    pub fn delete(&mut self, id: SparseArrayId<T>) {
        let idx = self.id_to_index.remove(&id).unwrap();
        if idx + 1 == self.data.len() {
            self.data.pop();
        } else {
            let last_idx = self.data.len() - 1;
            self.data.swap(idx, last_idx);
            self.id_to_index.insert(self.data[idx].0, idx);
            self.data.pop();
        }
        self.unused_indices.push(id.0);
    }

    #[allow(unused)]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().map(|(_, data)| data)
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (SparseArrayId<T>, &T)> {
        self.data.iter().map(|(id, data)| (*id, data))
    }
}

impl<T> Index<SparseArrayId<T>> for SparseArray<T> {
    type Output = T;
    fn index(&self, index: SparseArrayId<T>) -> &Self::Output {
        let index = *self.id_to_index.get(&index).unwrap();
        &self.data[index].1
    }
}

impl<T> IndexMut<SparseArrayId<T>> for SparseArray<T> {
    fn index_mut(&mut self, index: SparseArrayId<T>) -> &mut Self::Output {
        let index = *self.id_to_index.get(&index).unwrap();
        &mut self.data[index].1
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct SparseArrayId<T>(usize, PhantomData<T>);

impl<T> SparseArrayId<T> {
    pub fn as_usize(&self) -> usize {
        self.0
    }

    pub fn from_usize(data: usize) -> SparseArrayId<T> {
        SparseArrayId(data, PhantomData)
    }
}

impl<T> PartialEq for SparseArrayId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for SparseArrayId<T> {}

impl<T> PartialOrd for SparseArrayId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for SparseArrayId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for SparseArrayId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T> Clone for SparseArrayId<T> {
    fn clone(&self) -> Self {
        SparseArrayId(self.0, PhantomData)
    }
}

impl<T> Copy for SparseArrayId<T> {}
