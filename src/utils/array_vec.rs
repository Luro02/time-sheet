use core::mem;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayVec<T, const N: usize> {
    data: [Option<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {
    pub const fn new() -> Self {
        Self {
            data: [const { None }; N],
            len: 0,
        }
    }

    pub const fn push(&mut self, value: T) {
        debug_assert!(self.len() < N);
        assert!(self.data[self.len()].is_none());

        // NOTE: the code looks so weird, because drop can not be run in const fn yet
        let mut old_value = Some(value);
        mem::swap(&mut self.data[self.len()], &mut old_value);
        self.len += 1;

        // should be okay, because the old value is None
        mem::forget(old_value);
    }

    pub const fn pop(&mut self) -> Option<T> {
        let mut result = None;

        if self.len() > 0 {
            self.len -= 1;
            mem::swap(&mut self.data[self.len()], &mut result);
        }

        result
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.iter().any(|v| v == value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.data.iter().take(self.len()).filter_map(|v| v.as_ref())
    }
}

pub trait TryExtend<A> {
    type Error;

    fn try_extend<T>(&mut self, iter: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = A>;
}

impl<A, const N: usize> TryExtend<A> for ArrayVec<A, N> {
    type Error = anyhow::Error;

    fn try_extend<T>(&mut self, iter: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = A>,
    {
        for value in iter {
            if self.len() == N {
                return Err(anyhow::anyhow!("ArrayVec is full"));
            }

            self.push(value);
        }

        Ok(())
    }
}

impl<A, const N: usize> Extend<A> for ArrayVec<A, N> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = A>,
    {
        self.try_extend(iter).expect("ArrayVec is not large enough");
    }
}

impl<A, const N: usize> FromIterator<A> for ArrayVec<A, N> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = A>,
    {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl<T, const N: usize> IntoIterator for ArrayVec<T, N> {
    type Item = T;
    type IntoIter = ::core::iter::Flatten<::core::array::IntoIter<Option<T>, N>>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter().flatten()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayVec<T, N> {
    type Item = &'a T;
    type IntoIter = ::core::iter::Flatten<::core::slice::Iter<'a, Option<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter().flatten()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = ::core::iter::Flatten<::core::slice::IterMut<'a, Option<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut().flatten()
    }
}

impl<T, const N: usize> Default for ArrayVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
