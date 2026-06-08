use std::collections::VecDeque;
use std::mem::transmute;

use merc_collections::IndexedSet;

use crate::SymbolRef;
use crate::aterm::ATermRef;

/// A trait for transmuting the lifetime of an object to a shorter lifetime.
///
/// # Safety
///
/// The implementation of this trait must ensure that the transmuted lifetime is
/// always shorter than the original lifetime.
pub unsafe trait Transmutable {
    type Target<'a>: ?Sized
    where
        Self: 'a;

    /// Transmute the lifetime of the object to 'a, which is shorter than the given lifetime.
    fn transmute_lifetime<'a>(&'_ self) -> &'a Self::Target<'a>;

    /// Transmute the lifetime of the object to 'a, which is shorter than the given lifetime.
    fn transmute_lifetime_mut<'a>(&'_ mut self) -> &'a mut Self::Target<'a>;
}

unsafe impl Transmutable for ATermRef<'static> {
    type Target<'a> = ATermRef<'a>;

    fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
        unsafe { transmute::<&Self, &'a ATermRef<'a>>(self) }
    }

    fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
        unsafe { transmute::<&mut Self, &'a mut ATermRef<'a>>(self) }
    }
}

unsafe impl Transmutable for SymbolRef<'static> {
    type Target<'a> = SymbolRef<'a>;

    fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
        unsafe { transmute::<&Self, &'a SymbolRef<'a>>(self) }
    }

    fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
        unsafe { transmute::<&mut Self, &'a mut SymbolRef<'a>>(self) }
    }
}

unsafe impl<T: Transmutable> Transmutable for Option<T> {
    type Target<'a>
        = Option<T>
    where
        T: 'a;

    fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
        unsafe { transmute::<&Self, &'a Option<T>>(self) }
    }

    fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
        unsafe { transmute::<&mut Self, &'a mut Option<T>>(self) }
    }
}

unsafe impl<T: Transmutable> Transmutable for Vec<T>
where
    for<'a> T::Target<'a>: Sized,
{
    type Target<'a>
        = Vec<T::Target<'a>>
    where
        T: 'a;

    fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
        unsafe { transmute::<&Self, &'a Vec<T::Target<'a>>>(self) }
    }

    fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
        unsafe { transmute::<&mut Self, &'a mut Vec<T::Target<'a>>>(self) }
    }
}

unsafe impl<T: Transmutable> Transmutable for VecDeque<T>
where
    for<'a> T::Target<'a>: Sized,
{
    type Target<'a>
        = VecDeque<T::Target<'a>>
    where
        T: 'a;

    fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
        unsafe { transmute::<&Self, &'a VecDeque<T::Target<'a>>>(self) }
    }

    fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
        unsafe { transmute::<&mut Self, &'a mut VecDeque<T::Target<'a>>>(self) }
    }
}

unsafe impl<T: Transmutable> Transmutable for IndexedSet<T>
where
    for<'a> T::Target<'a>: Sized,
{
    type Target<'a>
        = IndexedSet<T::Target<'a>>
    where
        T: 'a;

    fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
        unsafe { transmute::<&Self, &'a IndexedSet<T::Target<'a>>>(self) }
    }

    fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
        unsafe { transmute::<&mut Self, &'a mut IndexedSet<T::Target<'a>>>(self) }
    }
}

// In Rust Its not yet possible to implement it for any tuples, so we implement it for some common sizes.
unsafe impl<T1: Transmutable, T2: Transmutable> Transmutable for (T1, T2)
where
    for<'a> T1::Target<'a>: Sized,
    for<'a> T2::Target<'a>: Sized,
{
    type Target<'a>
        = (T1::Target<'a>, T2::Target<'a>)
    where
        T1: 'a,
        T2: 'a;

    fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
        unsafe { transmute::<&Self, &'a (T1::Target<'a>, T2::Target<'a>)>(self) }
    }

    fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
        unsafe { transmute::<&mut Self, &'a mut (T1::Target<'a>, T2::Target<'a>)>(self) }
    }
}

unsafe impl<T: Transmutable> Transmutable for [T]
where
    for<'a> T::Target<'a>: Sized,
{
    type Target<'a>
        = [T::Target<'a>]
    where
        T: 'a;

    fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
        unsafe { transmute::<&Self, &'a [T::Target<'a>]>(self) }
    }

    fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
        unsafe { transmute::<&mut Self, &'a mut [T::Target<'a>]>(self) }
    }
}

unsafe impl Transmutable for bool {
    type Target<'a> = bool;

    fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
        unsafe { transmute::<&Self, &'a bool>(self) }
    }

    fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
        unsafe { transmute::<&mut Self, &'a mut bool>(self) }
    }
}
