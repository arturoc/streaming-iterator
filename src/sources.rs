use super::{DoubleEndedStreamingIterator, StreamingIterator};
use core::marker::PhantomData;
use core::usize;

/// Turns a normal, non-streaming iterator into a streaming iterator.
///
/// ```
/// # use streaming_iterator::{StreamingIterator, convert};
/// let scores = vec![100, 50, 80];
/// let mut streaming_iter = convert(scores);
/// while let Some(score) = streaming_iter.next() {
///     println!("The score is: {}", score);
/// }
/// ```
#[inline]
pub fn convert<I>(it: I) -> Convert<I::IntoIter>
where
    I: IntoIterator,
{
    Convert {
        it: it.into_iter(),
        item: None,
    }
}

/// Turns an iterator of references into a streaming iterator.
///
/// ```
/// # use streaming_iterator::{StreamingIterator, convert_ref};
/// let scores = vec![100, 50, 80];
/// let mut streaming_iter = convert_ref(&scores);
/// while let Some(score) = streaming_iter.next() {
///     println!("The score is: {}", score);
/// }
/// ```
#[inline]
pub fn convert_ref<'a, I, T: ?Sized>(iterator: I) -> ConvertRef<'a, I::IntoIter, T>
where
    I: IntoIterator<Item = &'a T>,
{
    ConvertRef {
        it: iterator.into_iter(),
        item: None,
    }
}

/// Creates an empty iterator.
///
/// ```
/// # use streaming_iterator::StreamingIterator;
/// let mut streaming_iter = streaming_iterator::empty::<i32>();
/// assert_eq!(streaming_iter.next(), None);
/// ```
#[inline]
pub fn empty<T>() -> Empty<T> {
    Empty {
        phantom: PhantomData,
    }
}

/// Creates an iterator that returns items from a function call.
///
/// ```
/// # use streaming_iterator::StreamingIterator;
/// let mut count = 0;
/// let mut streaming_iter = streaming_iterator::from_fn(|| {
///     count += 1;
///     if count < 4 { Some(count) } else { None }
/// });
/// assert_eq!(streaming_iter.next(), Some(&1));
/// assert_eq!(streaming_iter.next(), Some(&2));
/// assert_eq!(streaming_iter.next(), Some(&3));
/// assert_eq!(streaming_iter.next(), None);
/// ```
#[inline]
pub fn from_fn<T, F: FnMut() -> Option<T>>(gen: F) -> FromFn<T, F> {
    FromFn { gen, item: None }
}

/// Creates an iterator that returns exactly one item.
///
/// ```
/// # use streaming_iterator::StreamingIterator;
/// let mut streaming_iter = streaming_iterator::once(1);
/// assert_eq!(streaming_iter.next(), Some(&1));
/// assert_eq!(streaming_iter.next(), None);
/// ```
#[inline]
pub fn once<T>(item: T) -> Once<T> {
    Once {
        first: true,
        item: Some(item),
    }
}

/// Creates an iterator that returns exactly one item from a function call.
///
/// ```
/// # use streaming_iterator::StreamingIterator;
/// #[derive(Debug, PartialEq)]
/// struct Expensive(i32);
/// let mut streaming_iter = streaming_iterator::once_with(|| Expensive(1));
/// assert_eq!(streaming_iter.next(), Some(&Expensive(1)));
/// assert_eq!(streaming_iter.next(), None);
/// ```
#[inline]
pub fn once_with<T, F: FnOnce() -> T>(gen: F) -> OnceWith<T, F> {
    OnceWith {
        gen: Some(gen),
        item: None,
    }
}

/// Creates an iterator that returns an item endlessly.
///
/// ```
/// # use streaming_iterator::StreamingIterator;
/// let mut streaming_iter = streaming_iterator::repeat(1);
/// assert_eq!(streaming_iter.next(), Some(&1));
/// assert_eq!(streaming_iter.next(), Some(&1));
/// assert_eq!(streaming_iter.next(), Some(&1));
/// assert_eq!(streaming_iter.next(), Some(&1));
/// assert_eq!(streaming_iter.next(), Some(&1));
/// // ...
/// ```
#[inline]
pub fn repeat<T>(item: T) -> Repeat<T> {
    Repeat { item }
}

/// Creates an iterator that endlessly returns items from a function call.
///
/// ```
/// # use streaming_iterator::StreamingIterator;
/// let mut count = 0;
/// let mut streaming_iter = streaming_iterator::repeat_with(|| {
///     count += 1;
///     count
/// });
/// assert_eq!(streaming_iter.next(), Some(&1));
/// assert_eq!(streaming_iter.next(), Some(&2));
/// assert_eq!(streaming_iter.next(), Some(&3));
/// assert_eq!(streaming_iter.next(), Some(&4));
/// assert_eq!(streaming_iter.next(), Some(&5));
/// // ...
/// ```
#[inline]
pub fn repeat_with<T, F: FnMut() -> T>(gen: F) -> RepeatWith<T, F> {
    RepeatWith { gen, item: None }
}

/// Creates an iterator where each successive item is computed from the preceding one.
///
/// ```
/// # use streaming_iterator::StreamingIterator;
/// let mut streaming_iter = streaming_iterator::successors(
///     Some(1),
///     |count| if count < 3 { Some(count + 1) } else { None },
/// );
/// assert_eq!(streaming_iter.next(), Some(&1));
/// assert_eq!(streaming_iter.next(), Some(&2));
/// assert_eq!(streaming_iter.next(), Some(&3));
/// assert_eq!(streaming_iter.next(), None);
/// ```
#[inline]
pub fn successors<T, F: FnMut(T) -> Option<T>>(first: Option<T>, succ: F) -> Successors<T, F> {
    Successors {
        first: true,
        item: first,
        succ,
    }
}

/// A streaming iterator which yields elements from a normal, non-streaming, iterator.
#[derive(Clone, Debug)]
pub struct Convert<I>
where
    I: Iterator,
{
    it: I,
    item: Option<I::Item>,
}

impl<I> StreamingIterator for Convert<I>
where
    I: Iterator,
{
    type Item = I::Item;

    #[inline]
    fn advance(&mut self) {
        self.item = self.it.next();
    }

    #[inline]
    fn get(&self) -> Option<&I::Item> {
        self.item.as_ref()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.it.count()
    }

    #[inline]
    fn fold<Acc, Fold>(self, init: Acc, mut f: Fold) -> Acc
    where
        Self: Sized,
        Fold: FnMut(Acc, &Self::Item) -> Acc,
    {
        self.it.fold(init, move |acc, item| f(acc, &item))
    }
}

impl<I> DoubleEndedStreamingIterator for Convert<I>
where
    I: DoubleEndedIterator,
{
    #[inline]
    fn advance_back(&mut self) {
        self.item = self.it.next_back();
    }

    #[inline]
    fn rfold<Acc, Fold>(self, init: Acc, mut f: Fold) -> Acc
    where
        Self: Sized,
        Fold: FnMut(Acc, &Self::Item) -> Acc,
    {
        self.it.rev().fold(init, move |acc, item| f(acc, &item))
    }
}

/// A streaming iterator which yields elements from an iterator of references.
#[derive(Clone, Debug)]
pub struct ConvertRef<'a, I, T: ?Sized>
where
    I: Iterator<Item = &'a T>,
    T: 'a,
{
    it: I,
    item: Option<&'a T>,
}

impl<'a, I, T: ?Sized> StreamingIterator for ConvertRef<'a, I, T>
where
    I: Iterator<Item = &'a T>,
{
    type Item = T;

    #[inline]
    fn advance(&mut self) {
        self.item = self.it.next();
    }

    #[inline]
    fn get(&self) -> Option<&T> {
        self.item
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.it.count()
    }

    #[inline]
    fn fold<Acc, Fold>(self, init: Acc, mut f: Fold) -> Acc
    where
        Self: Sized,
        Fold: FnMut(Acc, &Self::Item) -> Acc,
    {
        self.it.fold(init, move |acc, item| f(acc, item))
    }
}

impl<'a, I, T: ?Sized> DoubleEndedStreamingIterator for ConvertRef<'a, I, T>
where
    I: DoubleEndedIterator<Item = &'a T>,
{
    #[inline]
    fn advance_back(&mut self) {
        self.item = self.it.next_back();
    }

    #[inline]
    fn rfold<Acc, Fold>(self, init: Acc, mut f: Fold) -> Acc
    where
        Self: Sized,
        Fold: FnMut(Acc, &Self::Item) -> Acc,
    {
        self.it.rev().fold(init, move |acc, item| f(acc, item))
    }
}

/// A simple iterator that returns nothing.
#[derive(Clone, Debug)]
pub struct Empty<T> {
    phantom: PhantomData<T>,
}

impl<T> StreamingIterator for Empty<T> {
    type Item = T;

    #[inline]
    fn advance(&mut self) {}

    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl<T> DoubleEndedStreamingIterator for Empty<T> {
    #[inline]
    fn advance_back(&mut self) {}
}

/// A simple iterator that returns items from a function call.
#[derive(Clone, Debug)]
pub struct FromFn<T, F> {
    gen: F,
    item: Option<T>,
}

impl<T, F: FnMut() -> Option<T>> StreamingIterator for FromFn<T, F> {
    type Item = T;

    #[inline]
    fn advance(&mut self) {
        self.item = (self.gen)();
    }

    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        self.item.as_ref()
    }
}

/// A simple iterator that returns exactly one item.
#[derive(Clone, Debug)]
pub struct Once<T> {
    first: bool,
    item: Option<T>,
}

impl<T> StreamingIterator for Once<T> {
    type Item = T;

    #[inline]
    fn advance(&mut self) {
        if self.first {
            self.first = false;
        } else {
            self.item = None;
        }
    }

    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        self.item.as_ref()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.first as usize;
        (len, Some(len))
    }
}

impl<T> DoubleEndedStreamingIterator for Once<T> {
    #[inline]
    fn advance_back(&mut self) {
        self.advance();
    }
}

/// A simple iterator that returns exactly one item from a function call.
#[derive(Clone, Debug)]
pub struct OnceWith<T, F> {
    gen: Option<F>,
    item: Option<T>,
}

impl<T, F: FnOnce() -> T> StreamingIterator for OnceWith<T, F> {
    type Item = T;

    #[inline]
    fn advance(&mut self) {
        self.item = match self.gen.take() {
            Some(gen) => Some(gen()),
            None => None,
        };
    }

    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        self.item.as_ref()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.gen.is_some() as usize;
        (len, Some(len))
    }
}

impl<T, F: FnOnce() -> T> DoubleEndedStreamingIterator for OnceWith<T, F> {
    #[inline]
    fn advance_back(&mut self) {
        self.advance();
    }
}

/// A simple iterator that repeats an item endlessly.
#[derive(Clone, Debug)]
pub struct Repeat<T> {
    item: T,
}

impl<T> StreamingIterator for Repeat<T> {
    type Item = T;

    #[inline]
    fn advance(&mut self) {}

    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        Some(&self.item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

impl<T> DoubleEndedStreamingIterator for Repeat<T> {
    #[inline]
    fn advance_back(&mut self) {}
}

/// A simple iterator that endlessly returns items from a function call.
#[derive(Clone, Debug)]
pub struct RepeatWith<T, F> {
    gen: F,
    item: Option<T>,
}

impl<T, F: FnMut() -> T> StreamingIterator for RepeatWith<T, F> {
    type Item = T;

    #[inline]
    fn advance(&mut self) {
        self.item = Some((self.gen)());
    }

    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        self.item.as_ref()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

/// An iterator where each successive item is computed from the preceding one.
#[derive(Clone, Debug)]
pub struct Successors<T, F> {
    first: bool,
    item: Option<T>,
    succ: F,
}

impl<T, F: FnMut(T) -> Option<T>> StreamingIterator for Successors<T, F> {
    type Item = T;

    #[inline]
    fn advance(&mut self) {
        if self.first {
            self.first = false;
        } else if let Some(item) = self.item.take() {
            self.item = (self.succ)(item);
        }
    }

    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        self.item.as_ref()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match (self.first, &self.item) {
            // We have a first item and unknown successors
            (true, &Some(_)) => (1, None),
            // We only have unknown successors
            (false, &Some(_)) => (0, None),
            // We have nothing.
            (_, &None) => (0, Some(0)),
        }
    }
}
