# btree

This crate is an implementation of a general btree based on the one in `std` (currently implemented at commit `485ec3fbcc12fa14ef6596dabb125ad710499c9e`).
It has then been adapted to work on `stable` and expose `BTree` as a separate data structure, analogous to `hashbrown`'s `HashTable`.

Thi [ACP](https://github.com/rust-lang/libs-team/issues/807) was the impetus to create this crate.