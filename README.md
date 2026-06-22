# btree

This crate was made as a follow up to the [ACP](https://github.com/rust-lang/libs-team/issues/807) to add a `BTree` structure to the standard library. This crate is the btree counterpart to [`hashbrown`](https://crates.io/crates/hashbrown).

This implements a standard `B-Tree` which is based on the standard library's `BTreeMap`/`BTreeSet`. This is then adapted to allow for a more generic `BTree` type which allows for more flexible usage of the data structure, analogous to `HashTable` form `hashbrown`.

`BTree` stores all keys in order as specified by the ordering function specified on access (`insert`, `find`, `entry`, etc.). If there exists an ordering where every access is a compatible with, then the iterators yield results in that ordering.

Even if the orderings provided on access don't form a total order over the elements of the tree or are not compatible with each other...
* there will not be any UB
* calling methods that take an ordering function may panic, abort, leak memory, not terminate, etc
* iterators will yield elements in an arbitrary order

## Status

This crate is under development, and is not yet ready for production.

## License

Licensed under either of:

    Apache License, Version 2.0, (LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or https://opensource.org/license/mit)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.