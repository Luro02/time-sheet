use core::iter;
use core::marker::PhantomData;

use crate::utils::TryExtend;

#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct MapWith<I, F, B, R> {
    iter: I,
    f: F,
    value: Option<B>,
    _p: PhantomData<R>,
}

impl<I, F, B, R> Iterator for MapWith<I, F, B, R>
where
    I: Iterator,
    F: FnMut(I::Item, B) -> (R, B),
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        let iter_value = self.iter.next()?;
        // value will be inserted again later on, so it is always present
        let value = self.value.take().unwrap();
        let (result, value) = (self.f)(iter_value, value);
        self.value = Some(value);
        Some(result)
    }
}

pub trait IteratorExt: Iterator {
    fn map_with<F, B, R>(self, init: B, f: F) -> MapWith<Self, F, B, R>
    where
        F: FnMut(Self::Item, B) -> (R, B),
        Self: Sized,
    {
        MapWith {
            iter: self,
            f,
            value: Some(init),
            _p: PhantomData,
        }
    }

    // TODO: test this?
    fn filter_map_with<F, B, R>(
        self,
        init: B,
        f: F,
    ) -> iter::Flatten<MapWith<Self, F, B, Option<R>>>
    where
        F: FnMut(Self::Item, B) -> (Option<R>, B),
        Self: Sized,
    {
        <Self as IteratorExt>::map_with(self, init, f).flatten()
    }

    fn try_collect<B>(self) -> Result<B, B::Error>
    where
        B: TryExtend<Self::Item> + Default,
        Self: Sized,
    {
        let mut result = B::default();
        result.try_extend(self)?;
        Ok(result)
    }
}

impl<I: Iterator> IteratorExt for I {}
