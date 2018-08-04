//! Skew heaps.

#![deny(missing_docs)]
#![cfg_attr(feature = "specialization", feature(specialization))]

use std::fmt::{self, Debug};
use std::iter::FromIterator;
use std::mem::{replace, swap};

struct Node<T> {
    l: Option<Box<Node<T>>>,
    r: Option<Box<Node<T>>>,
    item: T,
}

/// Merges two possibly empty heaps into a single heap.
fn merge<T: Ord>(mut a: &mut Option<Box<Node<T>>>, mut b: Option<Box<Node<T>>>) {
    loop {
        a = {
            let a = a;

            match *a {
                None => return *a = b,
                Some(ref mut a) => match b {
                    None => return,
                    Some(mut bn) => {
                        if a.item < bn.item {
                            swap(a, &mut bn);
                        }

                        let a = &mut **a;
                        swap(&mut a.l, &mut a.r);

                        b = replace(&mut a.l, Some(bn));
                        &mut a.l
                    }
                }
            }
        };
    }
}

/// A skew heap.
pub struct SkewHeap<T: Ord> {
    nodes: Nodes<T>,
    len: usize,
}

impl<T: Ord> SkewHeap<T> {
    /// Returns an empty heap.
    pub fn new() -> Self {
        SkewHeap { nodes: Nodes { node: None } , len: 0 }
    }

    /// Returns `true` if the heap contains no items.
    pub fn is_empty(&self) -> bool {
        self.nodes.node.is_none()
    }

    /// Returns the number of items in the heap.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns an iterator that yields references to the heap's items in arbitrary order.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            nodes: self.nodes.node.as_ref().map_or(vec![], |node| vec![node]),
            len: self.len,
        }
    }

    /// Returns a reference to the heap's greatest item.
    ///
    /// Returns `None` if the heap is empty.
    pub fn peek(&self) -> Option<&T> {
        self.nodes.node.as_ref().map(|node| &node.item)
    }

    /// Pushes the given item onto the heap.
    pub fn push(&mut self, item: T) {
        self.push_node(Box::new(Node { l: None, r: None, item: item }));
    }

    /// Moves all items from the given heap into the heap.
    ///
    /// # Examples
    ///
    /// ```
    /// use skew_heap::SkewHeap;
    ///
    /// let mut h1 = SkewHeap::new();
    /// h1.push(4);
    /// h1.push(2);
    /// h1.push(3);
    ///
    /// let mut h2 = SkewHeap::new();
    /// h2.push(1);
    /// h2.push(8);
    ///
    /// h1.append(&mut h2);
    ///
    /// assert_eq!(h1.len(), 5);
    /// assert_eq!(h1.peek(), Some(&8));
    ///
    /// assert_eq!(h2.len(), 0);
    /// assert_eq!(h2.peek(), None);
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        self.len += replace(&mut other.len, 0);
        merge(&mut self.nodes.node, other.nodes.node.take());
    }

    /// Removes all items from the heap.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Removes the heap's greatest item and returns it.
    ///
    /// Returns `None` if the heap was empty.
    pub fn pop(&mut self) -> Option<T> {
        self.pop_node().map(|node| node.item)
    }

    /// Pushes the given item onto to the heap, then removes the heap's greatest item and returns
    /// it.
    ///
    /// # Examples
    ///
    /// ```
    /// use skew_heap::SkewHeap;
    ///
    /// let mut h = SkewHeap::new();
    ///
    /// assert_eq!(h.push_pop(5), 5);
    /// assert_eq!(h.len(), 0);
    /// assert_eq!(h.peek(), None);
    ///
    /// h.extend(&[4, 5]);
    /// assert_eq!(h.len(), 2);
    /// assert_eq!(h.peek(), Some(&5));
    ///
    /// assert_eq!(h.push_pop(6), 6);
    /// assert_eq!(h.len(), 2);
    /// assert_eq!(h.peek(), Some(&5));
    ///
    /// assert_eq!(h.push_pop(3), 5);
    /// assert_eq!(h.len(), 2);
    /// assert_eq!(h.peek(), Some(&4));
    ///
    /// assert_eq!(h.pop(), Some(4));
    /// assert_eq!(h.pop(), Some(3));
    /// assert_eq!(h.pop(), None);
    /// ```
    pub fn push_pop(&mut self, mut item: T) -> T {
        match self.nodes.node {
            Some(ref root) if item >= root.item => {}
            _ => if let Some(mut node) = self.pop_node() {
                swap(&mut node.item, &mut item);
                self.push_node(node);
            },
        }

        item
    }

    /// Removes the greatest item from the heap, then pushes the given item onto the heap.
    ///
    /// Returns the item that was removed, or `None` if the heap was empty before the push.
    ///
    /// # Examples
    ///
    /// ```
    /// use skew_heap::SkewHeap;
    ///
    /// let mut h = SkewHeap::new();
    ///
    /// assert_eq!(h.replace(5), None);
    /// assert_eq!(h.len(), 1);
    /// assert_eq!(h.peek(), Some(&5));
    ///
    /// assert_eq!(h.replace(4), Some(5));
    /// assert_eq!(h.len(), 1);
    /// assert_eq!(h.peek(), Some(&4));
    ///
    /// assert_eq!(h.replace(6), Some(4));
    /// assert_eq!(h.len(), 1);
    /// assert_eq!(h.peek(), Some(&6));
    ///
    /// assert_eq!(h.pop(), Some(6));
    /// assert_eq!(h.pop(), None);
    /// ```
    pub fn replace(&mut self, mut item: T) -> Option<T> {
        match self.nodes.node {
            Some(ref mut root) if item >= root.item => Some(replace(&mut root.item, item)),
            _ => match self.pop_node() {
                None => {
                    self.push(item);
                    None
                }
                Some(mut node) => {
                    swap(&mut node.item, &mut item);
                    self.push_node(node);
                    Some(item)
                }
            },
        }
    }
}

impl<T: Ord> SkewHeap<T> {
    fn push_node(&mut self, node: Box<Node<T>>) {
        debug_assert!(node.l.is_none());
        debug_assert!(node.r.is_none());

        self.len += 1;
        merge(&mut self.nodes.node, Some(node));
    }

    fn pop_node(&mut self) -> Option<Box<Node<T>>> {
        self.nodes.node.take().map(|mut node| {
            self.len -= 1;
            self.nodes.node = node.l.take();
            merge(&mut self.nodes.node, node.r.take());
            node
        })
    }
}

impl<T: Ord> Default for SkewHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone> Clone for SkewHeap<T> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }

    fn clone_from(&mut self, other: &Self) {
        let nodes = replace(self, Self::new()).nodes;
        let mut other = other.iter();

        for (mut node, item) in nodes.zip(&mut other) {
            node.item.clone_from(item);
            self.push_node(node);
        }

        for item in other {
            self.push(item.clone());
        }
    }
}

impl<T: Ord> Extend<T> for SkewHeap<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, items: I) {
        <Self as SpecExtend<I>>::spec_extend(self, items);
    }
}

impl<'a, T: 'a + Ord + Copy> Extend<&'a T> for SkewHeap<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, items: I) {
        for item in items {
            self.push(*item);
        }
    }
}

trait SpecExtend<I: IntoIterator> {
    fn spec_extend(&mut self, items: I);
}

#[cfg(not(feature = "specialization"))]
impl<I: IntoIterator> SpecExtend<I> for SkewHeap<I::Item> where I::Item: Ord {
    fn spec_extend(&mut self, items: I) {
        for item in items {
            self.push(item);
        }
    }
}

#[cfg(feature = "specialization")]
macro_rules! spec_extend {
    () => {
        impl<I: IntoIterator> SpecExtend<I> for SkewHeap<I::Item> where I::Item: Ord {
            default fn spec_extend(&mut self, items: I) {
                for item in items {
                    self.push(item);
                }
            }
        }

        impl<T: Ord> SpecExtend<SkewHeap<T>> for SkewHeap<T> {
            fn spec_extend(&mut self, ref mut other: SkewHeap<T>) {
                self.append(other);
            }
        }
    }
}

#[cfg(feature="specialization")]
spec_extend!{}

impl<T: Ord> FromIterator<T> for SkewHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(items: I) -> Self {
        let mut heap = Self::new();
        heap.extend(items);
        heap
    }
}

impl<'a, T: 'a + Ord + Copy> FromIterator<&'a T> for SkewHeap<T> {
    fn from_iter<I: IntoIterator<Item = &'a T>>(items: I) -> Self {
        let mut heap = Self::new();
        heap.extend(items);
        heap
    }
}

impl<T: Ord + Debug> Debug for SkewHeap<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

struct Nodes<T> {
    node: Option<Box<Node<T>>>,
}

impl<T> Drop for Nodes<T> {
    fn drop(&mut self) {
        for _ in self {}
    }
}

impl<T> Iterator for Nodes<T> {
    type Item = Box<Node<T>>;

    fn next(&mut self) -> Option<Box<Node<T>>> {
        self.node.take().map(|mut node| {
            loop {
                match node.l.take() {
                    None => {
                        self.node = node.r.take();
                        return node;
                    }
                    Some(mut l) => {
                        node.l = l.r.take();
                        l.r = Some(node);
                        node = l;
                    }
                }
            }
        })
    }
}

impl<T: Ord> IntoIterator for SkewHeap<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter { nodes: self.nodes, len: self.len }
    }
}

/// An iterator that yields a `SkewHeap`'s items in arbitrary order.
pub struct IntoIter<T> {
    nodes: Nodes<T>,
    len: usize,
}

impl<T> Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("IntoIter").field("len", &self.len).finish()
    }
}

impl<T> Default for IntoIter<T> {
    fn default() -> Self {
        IntoIter {
            nodes: Nodes { node: None },
            len: 0,
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.nodes.next().map(|node| {
            self.len -= 1;
            node.item
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.next()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T: Ord> IntoIterator for &'a SkewHeap<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

/// An iterator that yields references to a `SkewHeap`'s items in arbitrary order.
pub struct Iter<'a, T: 'a> {
    nodes: Vec<&'a Node<T>>,
    len: usize,
}

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter { nodes: self.nodes.clone(), len: self.len }
    }

    fn clone_from(&mut self, other: &Self) {
        self.nodes.clone_from(&other.nodes);
        self.len = other.len;
    }
}

impl<'a, T> Debug for Iter<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Iter").field("len", &self.len).finish()
    }
}

impl<'a, T> Default for Iter<'a, T> {
    fn default() -> Self {
        Iter {
            nodes: vec![],
            len: 0,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.nodes.pop().map(|node| {
            self.len -= 1;
            if let Some(ref l) = node.l { self.nodes.push(l); }
            if let Some(ref r) = node.r { self.nodes.push(r); }
            &node.item
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.next()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

#[allow(dead_code)]
fn assert_covariant() {
    fn heap<'a, T: Ord>(heap: SkewHeap<&'static T>) -> SkewHeap<&'a T> {
        heap
    }

    fn into_iter<'a, T: Ord>(iter: IntoIter<&'static T>) -> IntoIter<&'a T> {
        iter
    }

    fn iter<'a, 'i, T: Ord>(iter: Iter<'i, &'static T>) -> Iter<'i, &'a T> {
        iter
    }
}

#[allow(dead_code)]
fn assert_send_sync<A: Ord + Send, B: Ord + Sync>() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<IntoIter<A>>();
    assert_sync::<IntoIter<B>>();

    assert_send::<Iter<B>>();
    assert_sync::<Iter<B>>();

    assert_send::<SkewHeap<A>>();
    assert_sync::<SkewHeap<B>>();
}
