use std::marker::PhantomData;

struct ATermFFI {
    index: usize,
    root: usize,
}

struct ATermRefFFI<'a> {
    index: usize,
    _marker: PhantomData<&'a ()>
}

// impl ATermFFI {
//     fn new(index: usize) -> Self {
//         ATermFFI { index, root }
//     }
// }

#[cfg(feature = "import")]
mod import;
#[cfg(feature = "import")]
pub use import::*;

#[cfg(not(feature = "import"))]
mod export;
#[cfg(not(feature = "import"))]
pub use export::*;