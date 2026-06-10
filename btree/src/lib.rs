#![no_std]
#![feature(
    allocator_api,
    trusted_len,
    hasher_prefixfree_extras,
    extend_one,
    dropck_eyepatch,
    core_intrinsics,
    slice_ptr_get,
    min_specialization
)]

extern crate alloc;
#[cfg(test)]
#[macro_use]
extern crate std;

mod append;
mod borrow;
mod dedup_sorted_iter;
mod fix;
mod map;
mod mem;
mod merge_iter;
mod navigate;
mod node;
mod remove;
mod search;
mod set;
mod set_val;
mod split;

mod testing;
