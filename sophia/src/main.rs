//! Working version of the code proposed in issue 5

pub trait FromGraph<T, G, E>: Sized
where
    T: Borrow<str>,
    G: for <'x> Graph<'x, Error=E>,
    E: CoercibleWith<Never>,
    E: CoercibleWith<SophiaError>,
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
    G: for <'x> Graph<'x, Error=E>,
    E: CoercibleWith<Never>,
    E: CoercibleWith<SophiaError>,
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
    G: for <'x> Graph<'x, Error=E>,
    E: CoercibleWith<Never>,
    E: CoercibleWith<SophiaError>,
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

