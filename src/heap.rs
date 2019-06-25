use std::collections::HashMap;
use std::collections::LinkedList;

pub struct Heap<T> {
    bucktes: HashMap<u64, LinkedList<T>>,
    keys: Vec<u64>,
    current_bucket: LinkedList<T>,
    current_key: u64,
}

impl<T> Heap<T> {
    pub fn insert_key(&mut self, key: u64) {
        if self.keys.len() == 1 {
            if self.keys[0] > key {
                self.keys.insert(0, key);
            } else {
                self.keys.push(key);
            }
            return;
        }

        if self.keys.len() == 0 {
            self.keys.push(key);
            return;
        }

        if *self.keys.last().unwrap() < key {
            self.keys.push(key);
            return;
        }

        if *self.keys.first().unwrap() > key {
            self.keys.insert(0, key);
            return;
        }

        let mut idx = self.keys.len() / 2;
        let mut next_jump = self.keys.len() / 4;
        loop {
            if self.keys[idx] > key && self.keys[idx - 1] < key {
                self.keys.insert(idx, key);

                for idx in 0..self.keys.len() - 1 {
                    if self.keys[idx] > self.keys[idx + 1] {
                        panic!("Shiat");
                    }
                }
                return;
            }

            if self.keys[idx] > key {
                idx -= next_jump;
            } else {
                idx += next_jump;
            }
            next_jump /= 2;
        }
    }

    pub fn insert(&mut self, key: u64, item: T) {
        if self.current_key == key {
            self.current_bucket.push_back(item);
        } else {
            match self.bucktes.get_mut(&key) {
                None => {
                    self.insert_key(key);
                    let mut ll = LinkedList::new();
                    ll.push_back(item);
                    self.bucktes.insert(key, ll);
                }
                Some(bucket) => {
                    bucket.push_back(item);
                }
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.current_bucket.len() == 0 {
            return None;
        } else {
            return self.current_bucket.pop_front();
        }
    }

    pub fn len(&self) -> usize {
        self.current_bucket.len()
    }

    pub fn next_key(&mut self) {
        self.current_key = self.keys.remove(0);
        self.current_bucket = self.bucktes.remove(&self.current_key).unwrap();
    }
}
