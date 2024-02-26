use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Default)]
pub struct TypeMap {
    raw: HashMap<TypeId, Box<dyn Any>>,
}

impl TypeMap {
    fn new() -> Self {
        Self::default()
    }

    fn get<T: Any>(&self) -> Option<&T> {
        let key = TypeId::of::<T>();
        let Some(val) = self.raw.get(&key) else {
            return None;
        };
        val.downcast_ref::<T>()
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let key = TypeId::of::<T>();
        let Some(val) = self.raw.get_mut(&key) else {
            return None;
        };
        val.downcast_mut::<T>()
    }

    fn insert<T: Any>(&mut self, val: T) {
        let key = TypeId::of::<T>();
        self.raw.insert(key, Box::new(val));
    }

    fn remove<T: Any>(&mut self) {
        let key = TypeId::of::<T>();
        self.raw.remove(&key);
    }

    fn len(&self) -> usize {
        self.raw.len()
    }
}

#[cfg(test)]
mod test {
    use super::TypeMap;

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct S1 {
        a: i32,
        b: u32,
    }

    impl S1 {
        fn smth() -> Self {
            S1 { a: -10, b: 23 }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct S2(u8, u8, u8);

    impl S2 {
        fn smth() -> Self {
            S2(11, 22, 33)
        }
    }

    fn prefilled_map() -> TypeMap {
        let mut tm = TypeMap::new();
        tm.insert(S1::smth());
        tm.insert(S2::smth());
        tm
    }

    #[test]
    fn type_map_create() {
        let tm = prefilled_map();
        assert_eq!(tm.len(), 2);
    }

    #[test]
    fn type_map_get() {
        let tm = prefilled_map();
        let Some(s1) = tm.get::<S1>() else {
            panic!("type not found");
        };
        assert_eq!(*s1, S1::smth());
        let Some(s2) = tm.get::<S2>() else {
            panic!("type not found");
        };
        assert_eq!(*s2, S2::smth());
    }

    #[test]
    fn type_map_get_mut() {
        let mut tm = prefilled_map();
        let Some(s1) = tm.get_mut::<S1>() else {
            panic!("type not found");
        };
        assert_eq!(*s1, S1::smth());
        s1.a = -55;
        s1.b = 55;
        let Some(s1) = tm.get_mut::<S1>() else {
            panic!("type not found");
        };
        assert_eq!(*s1, S1 { a: -55, b: 55 });
    }
}
