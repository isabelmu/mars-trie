pub fn common_count<L: Iterator, R: Iterator, F: Fn(L::Item, R::Item) -> bool>(
  l: L, r: R, f: F) -> usize {
    let mut c = 0;
    for ((i, li), ri) in l.enumerate().zip(r) {
        c = i;
        if !f(li, ri) {
            return c;
        }
    }
    c
}

pub fn common_count_eq<L: Iterator, R: Iterator>(l: L, r: R) -> usize 
  where L::Item: PartialEq<R::Item> {
    common_count(l, r, |a, b| a == b)
}

#[cfg(test)]
mod test {
    use super::common_count;

    #[test]
    fn test_common_count() {
        let l = [4, 5, 6];
        let r = [4, 5, 7, 8];
        assert!(2 == common_count(l.iter(), r.iter(), |a, b| a == b));
    }
}

