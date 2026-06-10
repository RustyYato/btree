//! Taken from std, <https://github.com/rust-lang/rust/tree/beae781308e9ddef13074a03faf57ca2fac59a5b/library/alloc/src/collections/btree/borrow/tests.rs>

use super::DormantMutRef;

#[test]
fn test_borrow() {
    let mut data = 1;
    let mut stack = vec![];
    let mut rr = &mut data;
    for factor in [2, 3, 7].iter() {
        let (r, dormant_r) = DormantMutRef::new(rr);
        rr = r;
        assert_eq!(*rr, 1);
        stack.push((factor, dormant_r));
    }
    while let Some((factor, dormant_r)) = stack.pop() {
        let r = unsafe { dormant_r.awaken() };
        *r *= factor;
    }
    assert_eq!(data, 42);
}
