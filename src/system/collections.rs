use crate::{Array, ClassIdentity};

#[unity2::class(namespace = "System.Collections.Generic", name = "List`1")]
pub struct List<T: ClassIdentity> {
    #[rename(name = "_items")]
    #[readonly]
    pub items: Array<T>,
    #[rename(name = "_size")]
    #[readonly]
    pub size: i32,
    #[rename(name = "_version")]
    #[readonly]
    pub version: i32,
}

#[unity2::methods]
impl<T: ClassIdentity> List<T> {
    #[method(name = ".ctor", args = 0)]
    fn ctor(self);

    #[method(name = "Add")]
    fn add(self, item: T);

    #[method(name = "Insert")]
    fn insert(self, index: i32, item: T);

    #[method(name = "Remove")]
    fn remove(self, item: T) -> bool;

    #[method(name = "RemoveAt")]
    fn remove_at(self, index: i32);

    #[method(name = "Clear")]
    fn clear(self);

    #[method(name = "Contains")]
    fn contains(self, item: T) -> bool;

    #[method(name = "IndexOf", args = 1)]
    fn index_of(self, item: T) -> i32;

    #[method(name = "get_Count")]
    fn count(self) -> i32;

    #[method(name = "get_Item")]
    fn get(self, index: i32) -> T;

    #[method(name = "set_Item")]
    fn set(self, index: i32, value: T);

    #[method(name = "ToArray")]
    fn to_array(self) -> Array<T>;

    #[method(name = "Reverse", args = 0)]
    fn reverse(self);
}

impl<T: ClassIdentity> List<T> {
    pub fn new() -> Option<Self> {
        let list = <Self as crate::FromIlInstance>::instantiate()?;
        list.ctor();
        Some(list)
    }

    pub fn with_capacity(_capacity: i32) -> Option<Self> {
        Self::new()
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.count() == 0
    }

    #[inline]
    pub fn iter(self) -> ListIter<T> {
        let len = self.count() as usize;
        let items = self.items();
        ListIter { items, index: 0, len }
    }
}

pub struct ListIter<T: ClassIdentity> {
    items: Array<T>,
    index: usize,
    len: usize,
}

impl<T: ClassIdentity> Iterator for ListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.index < self.len {
            let v = self.items.get(self.index);
            self.index += 1;
            Some(v)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<T: ClassIdentity> ExactSizeIterator for ListIter<T> {}

#[unity2::class(namespace = "System.Collections.Generic", name = "Dictionary`2")]
pub struct Dictionary<K: ClassIdentity, V: ClassIdentity> {}

#[unity2::methods]
impl<K: ClassIdentity, V: ClassIdentity> Dictionary<K, V> {
    #[method(name = "Add")]
    fn add(self, key: K, value: V);

    #[method(name = "Remove", args = 1)]
    fn remove(self, key: K) -> bool;

    #[method(name = "Clear")]
    fn clear(self);

    #[method(name = "ContainsKey")]
    fn contains_key(self, key: K) -> bool;

    #[method(name = "ContainsValue")]
    fn contains_value(self, value: V) -> bool;

    // TryGetValue(K, out V) -> bool, the out slot in C# is equivalent to &mut V in Rust
    #[method(name = "TryGetValue")]
    fn try_get_value(self, key: K, value: &mut V) -> bool;

    #[method(name = "get_Item")]
    fn get(self, key: K) -> V;

    #[method(name = "set_Item")]
    fn set(self, key: K, value: V);

    #[method(name = "get_Count")]
    fn count(self) -> i32;
}
