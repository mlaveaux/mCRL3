#![doc = include_str!("../README.md")]

mod block_allocator;
mod counting_allocator;
mod erasable;
mod freelist;
mod global_allocator;
mod index_edge;
mod protection_set;
mod slice_dst;
mod stable_pointer_set;

pub use block_allocator::*;
pub use counting_allocator::*;
pub use erasable::*;
pub use freelist::*;
pub use global_allocator::*;
pub use index_edge::*;
pub use protection_set::*;
pub use slice_dst::*;
pub use stable_pointer_set::*;