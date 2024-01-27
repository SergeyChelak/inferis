use std::collections::VecDeque;

const PACKED_ARRAY_DEFAULT_CAPACITY: usize = 1000;

pub type ValueID = usize;

pub struct PackedArray<T> {
    data: Vec<T>,
    // bidirectional mapping
    id_to_index: Vec<usize>,
    index_to_id: Vec<usize>,
    // allocator data
    alloc_counter: usize,
    alloc_recycled: VecDeque<usize>,
    length: usize,
}

impl<T> Default for PackedArray<T> {
    fn default() -> Self {
        Self::with_capacity(PACKED_ARRAY_DEFAULT_CAPACITY)
    }
}

impl<T> PackedArray<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            id_to_index: Vec::with_capacity(capacity),
            index_to_id: Vec::with_capacity(capacity),
            alloc_recycled: VecDeque::default(),
            alloc_counter: 0,
            length: 0,
        }
    }

    pub fn get(&self, id: ValueID) -> Option<&T> {
        if !self.is_valid_id(id) {
            return None;
        }
        let idx = self.id_to_index[id];
        self.data.get(idx)
    }

    pub fn get_mut(&mut self, id: ValueID) -> Option<&mut T> {
        if !self.is_valid_id(id) {
            return None;
        }
        let idx = self.id_to_index[id];
        self.data.get_mut(idx)
    }

    pub fn ids(&self) -> &[ValueID] {
        &self.index_to_id[..self.length]
    }

    pub fn values(&self) -> &[T] {
        &self.data
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ValueID, &T)> {
        self.index_to_id.iter().zip(self.data.iter())
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    // appends new element and returns its id
    pub fn add(&mut self, val: T) -> ValueID {
        let next_id: ValueID;
        if let Some(id) = self.alloc_recycled.pop_front() {
            next_id = id;
        } else {
            next_id = self.alloc_counter;
            self.alloc_counter += 1;
        }
        if self.length == self.data.len() {
            self.data.push(val);
            self.id_to_index.push(self.length);
            self.index_to_id.push(next_id);
        } else {
            self.id_to_index[next_id] = self.length;
            self.index_to_id[self.length] = next_id;
            self.data[self.length] = val;
        }
        self.length += 1;
        next_id
    }

    // removes an object with id, returns true if object was presented otherwise false
    pub fn remove(&mut self, id: ValueID) -> bool {
        if self.is_empty() || !self.is_valid_id(id) {
            return false;
        }
        self.alloc_recycled.push_back(id);
        let idx = self.id_to_index[id];
        let last_idx = self.length - 1;
        self.data.swap(last_idx, idx);
        let last_id = self.index_to_id[last_idx];
        self.id_to_index[last_id] = idx;
        self.index_to_id[idx] = last_id;
        self.length -= 1;
        true
    }

    fn is_valid_id(&self, id: ValueID) -> bool {
        if id >= self.alloc_counter {
            return false;
        }
        if self.alloc_recycled.contains(&id) {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pa_test_create() {
        let mut array: PackedArray<i32> = PackedArray::default();
        array.add(1);
        array.add(2);
        array.add(3);
        assert_eq!(array.length, 3);
        assert_eq!(array.alloc_counter, 3);
        assert_eq!(array.data.len(), 3);
        assert_eq!(array.id_to_index, [0, 1, 2]);
    }

    #[test]
    fn pa_test_remove() {
        let mut array: PackedArray<i32> = PackedArray::default();
        array.add(1);
        array.add(2);
        let id = array.add(3);
        array.add(4);
        array.remove(id);
        assert_eq!(array.length, 3);
        assert_eq!(array.alloc_counter, 4);
        assert_eq!(array.id_to_index, [0, 1, 2, 2]);
        assert!(array.alloc_recycled.contains(&id));
    }

    #[test]
    fn pa_test_remove_add() {
        let mut array: PackedArray<i32> = PackedArray::default();
        array.add(1);
        array.add(2);
        let id_1 = array.add(3);
        array.add(4);
        array.remove(id_1);
        let id_2 = array.add(5);
        assert_eq!(id_1, id_2);
        assert_eq!(array.id_to_index, [0, 1, 3, 2]);
        assert_eq!(array.data, [1, 2, 4, 5]);
    }

    #[test]
    fn pa_test_remove_last() {
        let mut array: PackedArray<i32> = PackedArray::default();
        let id = array.add(1);
        array.remove(id);
        assert!(array.is_empty());
    }

    #[test]
    fn pa_test_remove_twice() {
        let mut array: PackedArray<i32> = PackedArray::default();
        let id = array.add(1);
        array.add(2);
        array.add(3);

        let res = array.remove(id);
        assert!(res);

        let res = array.remove(id);
        assert!(!res);
    }

    #[test]
    fn pa_test_remove_wrong() {
        let mut array: PackedArray<i32> = PackedArray::default();
        array.add(1);
        array.add(2);
        array.add(3);
        assert!(!array.remove(90000));
    }

    #[test]
    fn pa_test_get() {
        let mut array: PackedArray<i32> = PackedArray::default();
        let id_1 = array.add(1);
        let id_2 = array.add(2);
        let id_3 = array.add(3);
        assert_eq!(array.get(id_1), Some(&1));
        assert_eq!(array.get(id_2), Some(&2));
        assert_eq!(array.get(id_3), Some(&3));
    }

    #[test]
    fn pa_test_get_mut() {
        let mut array: PackedArray<i32> = PackedArray::default();
        let id_1 = array.add(1);
        let id_2 = array.add(2);
        let id_3 = array.add(3);
        *array.get_mut(id_2).unwrap() = 555;
        assert_eq!(array.get(id_1), Some(&1));
        assert_eq!(array.get(id_2), Some(&555));
        assert_eq!(array.get(id_3), Some(&3));
    }

    #[test]
    fn pa_test_get_ids() {
        let mut array: PackedArray<i32> = PackedArray::default();
        array.add(1);
        array.add(2);
        array.add(3);
        let id_1 = array.add(4);
        let id_2 = array.add(5);
        array.add(6);
        array.add(7);
        array.remove(id_1);
        array.remove(id_2);
        array.add(8);

        // order of ids isn't determined, sort it before assertion
        let mut ids = array.ids().to_vec();
        ids.sort();
        assert_eq!(ids, &[0, 1, 2, 3, 5, 6]);
    }
}
