//! Working version of the code proposed in issue 5
//!
//! NB: this section would be added to the documentation of Graph
//! 
//! # How to use `Graph` in a trait bound?
//! 
//! TL;DR: don't do it. Use `OwnedGraph` instead.
//! 
//! The lifetime parameter of `Graph` has a very specific semantics:
//! it is the lifetime for which the graph can be borrowed when iterating over its triples.
//! Since lifetime parameters in traits are [invariant](https://doc.rust-lang.org/nightly/nomicon/subtyping.html),
//! a instance of `Graph<'a>` will only be usable with the *exact* lifetime `'a`,
//! but not with a shorter lifetime, which is rather counter-intuitive.
//! 
//! In most situations, it is therefore more useful to use a 
//! [Higher-Rank Trait Bounds](https://doc.rust-lang.org/nomicon/hrtb.html): 
//! ```
//!   G: for <'x> Graph<'x>
//! ```
//! which states "G implements `Graph<'x>` for *every* lifetime".
//! 
//! This kind of trait bound is quite unusual, so a trait alias is provided: 
//! `OwnedGraph<E>` (where E is the associated error type) can be used instead.
//! 
//! NB: as the name `OwnedGraph` implies,
//! the Higher-Ranked Type Bound requires that the trait is valid for *every* lifetime,
//! so the graph can not borrow anything (which would restrict its lifetime).

/// A trait alias for types which are appropriate as a graph's associate error type.
pub trait GraphError: CoercibleWith<Never> + CoercibleWith<SophiaError> {}
impl<E> GraphError for E where E: CoercibleWith<Never> + CoercibleWith<SophiaError> {}

/// A convenient trait alias, easier to use than `Graph` itself.
pub trait OwnedGraph<E>: for <'x> Graph<'x, Error=E> where E: GraphError {}
impl<G, E> OwnedGraph<E> for G where G: for <'x> Graph<'x, Error=E>, E: GraphError {}

pub trait FromGraph<T, G, E>: Sized
where
    T: Borrow<str>,
    G: OwnedGraph<E>,
    E: GraphError,
    MyError: From<E>
{
    fn from_graph(s: &Term<T>, graph: &G) -> MyResult<Self>;
}



#[derive(Debug, Clone, Copy)]
struct A {
    value: i32,
}

impl<T, G, E> FromGraph<T, G, E> for A
where
    T: Borrow<str>,
    G: OwnedGraph<E>,
    E: GraphError,
    MyError: From<E>
{
    fn from_graph(s: &Term<T>, graph: &G) -> MyResult<Self> {
        let t_value = graph.iter_for_sp(s, &rdf::value).last().ok_or(MyError{})??;
        let value = t_value.o().value().parse::<i32>()?;
        Ok(A { value })
    }
}



#[derive(Debug, Clone, Copy)]
struct B {
    a: A,
    value: i32,
}

impl<T, G, E> FromGraph<T, G, E> for B
where
    T: Borrow<str>,
    G: OwnedGraph<E>,
    E: GraphError,
    MyError: From<E>
{
    fn from_graph(s: &Term<T>, graph: &G) -> MyResult<Self> {
        let t_a = graph.iter_for_sp(s, &HAS_A).last().ok_or(MyError{})??;
        let a = A::from_graph(t_a.o(), graph)?;

        let t_value = graph.iter_for_sp(s, &rdf::value).last().ok_or(MyError{})??;
        let value = t_value.o().value().parse::<i32>()?;
        Ok(B { a, value })
    }
}

fn main() {
    let mut g = FastGraph::new();
    nt::parse_str(SRC).in_graph(&mut g).unwrap();

    let b1 = B::from_graph(&EXD_B1, &g).unwrap();
    dbg!(&b1.value);
    dbg!(&b1.a.value);
}

static SRC: &'static str = r#"
<http://ex.co/data/a1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex.co/ns/A>.
<http://ex.co/data/a1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#value> "42"^^<http://www.w3.org/2001/XMLSchema#integer>.
<http://ex.co/data/b1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex.co/ns/B>.
<http://ex.co/data/b1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#value> "101"^^<http://www.w3.org/2001/XMLSchema#integer>.
<http://ex.co/data/b1> <http://ex.co/ns/has_a> <http://ex.co/data/a1>.
"#;

::lazy_static::lazy_static! {
    static ref EX_A: StaticTerm = StaticTerm::new_iri("http://ex.co/ns/A").unwrap();
    static ref EX_B: StaticTerm = StaticTerm::new_iri("http://ex.co/ns/B").unwrap();
    static ref HAS_A: StaticTerm = StaticTerm::new_iri("http://ex.co/ns/has_a").unwrap();

    static ref EXD_A1: StaticTerm = StaticTerm::new_iri("http://ex.co/data/a1").unwrap();
    static ref EXD_B1: StaticTerm = StaticTerm::new_iri("http://ex.co/data/b1").unwrap();
}

#[derive(Debug)]
pub struct MyError {}

impl From<Never> for MyError {
    fn from(_: Never) -> MyError { unimplemented!() }
}

impl From<std::num::ParseIntError> for MyError {
    fn from(_: std::num::ParseIntError) -> MyError { MyError{} }
}

pub type MyResult<T> = Result<T, MyError>;


use ::std::borrow::Borrow;

use ::sophia::error::{CoercibleWith, Never, Error as SophiaError};
use ::sophia::graph::Graph;
use ::sophia::graph::inmem::FastGraph;
use ::sophia::ns::rdf;
use ::sophia::parsers::nt;
use ::sophia::term::{StaticTerm, Term};
use ::sophia::triple::Triple;
use ::sophia::streams::*;

