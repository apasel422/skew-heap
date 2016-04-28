extern crate quickcheck;
extern crate skew_heap;

use quickcheck::{Arbitrary, Gen, quickcheck};
use skew_heap::SkewHeap;
use std::collections::BinaryHeap;

#[derive(Clone, Debug)]
enum Op<T> {
    Extend(Vec<T>),
    Pop,
    Push(T),
    PushPop(T),
    Replace(T),
}

impl<T: Arbitrary> Arbitrary for Op<T> {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        match gen.gen_range(0, 5) {
            0 => Op::Extend(<Vec<T>>::arbitrary(gen)),
            1 => Op::Pop,
            2 => Op::PushPop(T::arbitrary(gen)),
            3 => Op::Push(T::arbitrary(gen)),
            _ => Op::Replace(T::arbitrary(gen)),
        }
    }

    fn shrink(&self) -> Box<Iterator<Item = Self>> {
        match *self {
            Op::Extend(ref ts) => Box::new(ts.shrink().map(Op::Extend)),
            Op::Pop => Box::new(std::iter::empty()),
            Op::Push(ref t) => Box::new(t.shrink().map(Op::Push)),
            Op::PushPop(ref t) => Box::new(t.shrink().map(Op::PushPop)),
            Op::Replace(ref t) => Box::new(t.shrink().map(Op::Replace)),
        }
    }
}

impl<T: Ord> Op<T> {
    fn exec(self, heap: &mut SkewHeap<T>) -> Option<T> {
        match self {
            Op::Extend(ts) => { heap.extend(ts); None }
            Op::Pop => heap.pop(),
            Op::Push(t) => { heap.push(t); None }
            Op::PushPop(t) => Some(heap.push_pop(t)),
            Op::Replace(t) => heap.replace(t),
        }
    }

    fn exec_binary(self, heap: &mut BinaryHeap<T>) -> Option<T> {
        match self {
            Op::Extend(ts) => { heap.extend(ts); None }
            Op::Pop => heap.pop(),
            Op::Push(t) => { heap.push(t); None }
            Op::PushPop(t) => { heap.push(t); heap.pop() }
            Op::Replace(t) => { let x = heap.pop(); heap.push(t); x }
        }
    }
}

#[derive(Debug)]
enum Disagreement<T> {
    Result(Option<T>, Option<T>),
    Peek(Option<T>, Option<T>),
    Len(usize, usize),
    Pop(Option<T>, Option<T>),
}

#[test]
fn agrees_with_binary_heap() {
    fn t(ops: Vec<Op<i32>>) -> Result<(), Disagreement<i32>> {
        let mut skew = SkewHeap::new();
        let mut bin = BinaryHeap::new();

        for op in ops {
            let skew_r = op.clone().exec(&mut skew);
            let bin_r = op.exec_binary(&mut bin);

            if skew_r != bin_r {
                return Err(Disagreement::Result(skew_r, bin_r));
            }

            if skew.peek() != bin.peek() {
                return Err(Disagreement::Peek(skew.peek().cloned(), bin.peek().cloned()));
            }

            if skew.len() != bin.len() {
                return Err(Disagreement::Len(skew.len(), bin.len()));
            }
        }

        loop {
            match (skew.pop(), bin.pop()) {
                (Some(skew), Some(bin)) if skew == bin => {}
                (None, None) => return Ok(()),
                (skew, bin) => return Err(Disagreement::Pop(skew, bin)),
            }
        }
    }

    quickcheck(t as fn(_) -> _);
}

#[test]
fn iter_agrees_with_binary_heap_iter() {
    fn t(ops: Vec<Op<i32>>) -> bool {
        let mut skew = SkewHeap::new();
        let mut bin = BinaryHeap::new();

        for op in ops {
            op.clone().exec(&mut skew);
            op.exec_binary(&mut bin);
        }

        let mut skew_items: Vec<_> = skew.iter().collect();
        skew_items.sort();

        let mut bin_items: Vec<_> = bin.iter().collect();
        bin_items.sort();

        skew_items == bin_items
    }

    quickcheck(t as fn(_) -> _);
}

#[test]
fn into_iter_agrees_with_binary_heap_iter() {
    fn t(ops: Vec<Op<i32>>) -> bool {
        let mut skew = SkewHeap::new();
        let mut bin = BinaryHeap::new();

        for op in ops {
            op.clone().exec(&mut skew);
            op.exec_binary(&mut bin);
        }

        let mut skew_items: Vec<_> = skew.into_iter().collect();
        skew_items.sort();

        let mut bin_items: Vec<_> = bin.into_iter().collect();
        bin_items.sort();

        skew_items == bin_items
    }

    quickcheck(t as fn(_) -> _);
}

#[test]
fn clone_from() {
    fn t(ops1: Vec<Op<i32>>, ops2: Vec<Op<i32>>) -> bool {
        let mut heap1 = SkewHeap::new();

        for op in ops1 {
            op.exec(&mut heap1);
        }

        let mut heap2 = SkewHeap::new();

        for op in ops2 {
            op.exec(&mut heap2);
        }

        heap1.clone_from(&heap2);

        loop {
            match (heap1.pop(), heap2.pop()) {
                (Some(a), Some(b)) if a == b => {}
                (None, None) => return true,
                _ => return false,
            }
        }
    }

    quickcheck(t as fn(_, _) -> _);
}

#[test]
fn append() {
    fn t(ops1: Vec<Op<i32>>, ops2: Vec<Op<i32>>) -> bool {
        let mut heap1 = SkewHeap::new();

        for op in ops1 {
            op.exec(&mut heap1);
        }

        let mut heap2 = SkewHeap::new();

        for op in ops2 {
            op.exec(&mut heap2);
        }

        let mut all: Vec<_> = heap1.iter().cloned().collect();
        all.extend(&heap2);
        all.sort();

        heap1.append(&mut heap2);

        if !heap2.is_empty() || heap2.len() > 0 || heap2.iter().next().is_some() {
            return false;
        }

        let mut heap1: Vec<_> = heap1.into_iter().collect();
        heap1.sort();

        heap1 == all
    }

    quickcheck(t as fn(_, _) -> _);
}
