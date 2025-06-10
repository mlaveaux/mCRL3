use std::collections::VecDeque;
use std::mem::transmute;

use crate::aterm::ATermRef;

pub trait Transmutable {
    type Target<'a>
    where
        Self: 'a;

    /// Transmute the lifetime of the object to 'a, which is shorter than the given lifetime.
    fn transmute_lifetime<'a>(&'_ self) -> &'a Self::Target<'a>;

    /// Transmute the lifetime of the object to 'a, which is shorter than the given lifetime.
    fn transmute_lifetime_mut<'a>(&'_ mut self) -> &'a mut Self::Target<'a>;
}

impl Transmutable for ATermRef<'static> {
    type Target<'a> = ATermRef<'a>;

    fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
        unsafe { transmute::<&Self, &'a ATermRef<'a>>(self) }
    }

    fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
        unsafe { transmute::<&mut Self, &'a mut ATermRef<'a>>(self) }
    }
}

impl<T: Transmutable> Transmutable for Option<T> {
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

impl<T> Transmutable for Vec<T>
where
    T: Transmutable,
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

impl<T> Transmutable for VecDeque<T>
where
    T: Transmutable,
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