use std::slice::SliceIndex;

/// An enum used to indicate an edge or a self loop.
pub enum Edge<T> {
    Regular(T, T),

    /// For a self loop we only provide a mutable reference to the single state.
    Selfloop(T),
}

/// Index two locations (from, to) of an edge, returns mutable references to it.
pub fn index_edge<T, I: PartialEq + PartialOrd<usize> + SliceIndex<[T], Output = T>>(
    slice: &mut [T],
    a: I,
    b: I,
) -> Edge<&mut T> {
    if a == b {
        assert!(a <= slice.len());
        Edge::Selfloop(slice.get_mut(a).unwrap())
    } else {
        assert!(a <= slice.len() && b < slice.len());

        // safe because a, b are in bounds and distinct
        unsafe {
            let ar = &mut *(slice.get_unchecked_mut(a) as *mut _);
            let br = &mut *(slice.get_unchecked_mut(b) as *mut _);
            Edge::Regular(ar, br)
        }
    }
}
