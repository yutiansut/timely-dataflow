use std::rc::Rc;
use std::cell::RefCell;

use progress::timestamp::RootTimestamp;
use progress::{Timestamp, Operate, Subgraph};
use progress::nested::{Source, Target};
use timely_communication::{Allocate, Data};
use {Push, Pull};

use super::Scope;

/// A `Root` is the entry point to a timely dataflow computation. It wraps a `Allocate`,
/// and has a slot for one child `Operate`. The primary intended use of `Root` is through its
/// implementation of the `Scope` trait.
pub struct Root<A: Allocate> {
    allocator: Rc<RefCell<A>>,
    graph: Rc<RefCell<Vec<Box<Operate<RootTimestamp>>>>>,
}

impl<A: Allocate> Root<A> {
    pub fn new(c: A) -> Root<A> {
        Root {
            allocator: Rc::new(RefCell::new(c)),
            graph: Rc::new(RefCell::new(Vec::new())),
        }
    }
    pub fn step(&mut self) -> bool {
        let mut active = false;
        for scope in self.graph.borrow_mut().iter_mut() {
            let sub_active = scope.pull_internal_progress(&mut [], &mut [], &mut []);
            active = active || sub_active;
        }
        active
    }
    pub fn index(&self) -> usize { self.allocator.borrow().index() }
    pub fn peers(&self) -> usize { self.allocator.borrow().peers() }
}

impl<A: Allocate> Scope for Root<A> {
    type Timestamp = RootTimestamp;

    fn name(&self) -> String { format!("Worker[{}]", self.allocator.borrow().index()) }
    fn add_edge(&self, _source: Source, _target: Target) {
        panic!("Root::connect(): root doesn't maintain edges; who are you, how did you get here?")
    }

    fn add_operator<SC: Operate<RootTimestamp>+'static>(&self, mut scope: SC) -> usize  {

        scope.get_internal_summary();
        scope.set_external_summary(Vec::new(), &mut []);
        self.graph.borrow_mut().push(Box::new(scope));
        self.graph.borrow().len() - 1
    }

    fn new_subscope<T: Timestamp>(&mut self) -> Subgraph<RootTimestamp, T>  {
        let name = format!("{}::Subgraph[Root]", self.name());
        Subgraph::new_from(&mut (*self.allocator.borrow_mut()), 0, name)
    }
}

impl<A: Allocate> Allocate for Root<A> {
    fn index(&self) -> usize { self.allocator.borrow().index() }
    fn peers(&self) -> usize { self.allocator.borrow().peers() }
    fn allocate<D: Data>(&mut self) -> (Vec<Box<Push<D>>>, Box<Pull<D>>) {
        self.allocator.borrow_mut().allocate()
    }
}

impl<A: Allocate> Clone for Root<A> {
    fn clone(&self) -> Self { Root { allocator: self.allocator.clone(), graph: self.graph.clone() }}
}
