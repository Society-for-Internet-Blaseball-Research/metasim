use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct History<T: PartialEq>(BTreeMap<u64, T>);

impl<T: PartialEq> History<T> {
    pub fn new() -> History<T> {
        History(BTreeMap::new())
    }

    pub fn get(&self, time: u64) -> Option<&T> {
        self.0.range(..=time).rev().next().map(|(_, v)| v)
    }

    pub fn insert(&mut self, time: u64, value: T) -> Option<T> {
        self.0.insert(time, value)
    }

    pub fn dedup(&mut self) {
        // https://github.com/rust-lang/rust/issues/70530 would be nice here
        let mut prev = None;
        let mut remove = Vec::new();
        for (k, v) in self.0.iter() {
            if Some(v) == prev {
                remove.push(*k);
            } else {
                prev = Some(v)
            }
        }
        for k in remove {
            self.0.remove(&k);
        }
    }
}

impl<T: PartialEq> Default for History<T> {
    fn default() -> History<T> {
        History::new()
    }
}

#[cfg(test)]
mod tests {
    use super::History;
    use maplit::btreemap;

    #[test]
    fn test_get() {
        let history = History(btreemap! {
            1 => "a",
            3 => "b",
            5 => "c",
        });
        assert_eq!(history.get(0), None);
        assert_eq!(history.get(1), Some(&"a"));
        assert_eq!(history.get(2), Some(&"a"));
        assert_eq!(history.get(3), Some(&"b"));
        assert_eq!(history.get(4), Some(&"b"));
        assert_eq!(history.get(5), Some(&"c"));
        assert_eq!(history.get(6), Some(&"c"));
    }

    #[test]
    fn test_dedup() {
        let mut history = History(btreemap! {
            1 => "a",
            2 => "a",
            3 => "b",
            4 => "b",
            5 => "c",
        });
        history.dedup();
        assert_eq!(
            history.0,
            btreemap! {
                1 => "a",
                3 => "b",
                5 => "c",
            }
        );
    }
}
