use std::iter;

/// Resize vector, default-initializing any extra elements.
pub fn vec_resize<T: Default + Clone>(v: &mut Vec<T>, new_len: usize) {
    // This should call std::Vec::resize when it is stabilized.
    if v.len() < new_len {
        let len = v.len();
        v.extend(iter::repeat(Default::default()).take(new_len - len));
    } else {
        v.truncate(new_len)
    }
}

#[cfg(test)]
mod test {
    use super::vec_resize;
    use std::iter;
    use quickcheck::quickcheck;

    #[test]
    fn test_resize() {
        fn prop<T: Clone + Default + Eq>(mut v: Vec<T>) -> bool {
            let old_size = v.len();
            let new_size = old_size * 2;

            let old = v.clone();
            if old.len() != old_size { return false; }

            vec_resize(&mut v, new_size);
            if v.len() != new_size { return false; }

            if !old.iter().eq(v.iter().take(old_size)) {
                return false;
            }
            let d: T = Default::default();
            if !iter::repeat(&d).take(old_size).eq(
                  v.iter().skip(old_size)) {
                return false;
            }
            true
        }

        quickcheck(prop::<u32> as fn(Vec<u32>) -> bool)
    }
}

