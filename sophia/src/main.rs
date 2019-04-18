use ::std::borrow::Borrow;

use ::sophia::error::Never;
use ::sophia::graph::Graph;
use ::sophia::graph::inmem::FastGraph;
use ::sophia::ns::rdf;
use ::sophia::parsers::nt;
use ::sophia::term::{StaticTerm, Term};
use ::sophia::triple::Triple;
use ::sophia::streams::*;

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

pub trait FromGraph<'a, T, G>: Sized
where
    T: Borrow<str>,
    G: Graph<'a>,
    MyError: From<<G as Graph<'a>>::Error>,
{
    fn from_graph(s: &'a Term<T>, graph: &'a G) -> MyResult<Self>;
}

#[derive(Debug, Clone, Copy)]
struct A {
    value: i32,
}

impl<'a, T, G> FromGraph<'a, T, G> for A
where
    T: Borrow<str>,
    G: Graph<'a>,
    //MyError: From<<G as Graph<'a>>::Error>,
{
    fn from_graph(s: &'a Term<T>, graph: &'a G) -> MyResult<Self> {
        let t_value = graph.iter_for_sp(s, &rdf::value).last().ok_or(MyError{})??;
        let t_value = t_value.o();
        let value = t_value.value().parse::<i32>()?;
        Ok(A { value })
    }
}

#[derive(Debug, Clone, Copy)]
struct B {
    a: A,
    value: i32,
}

impl<'a, T, G> FromGraph<'a, T, G> for B
where
    T: Borrow<str>,
    G: Graph<'a>,
    //MyError: From<<G as Graph<'a>>::Error>,
{
    fn from_graph(s: &Term<T>, graph: &'a G) -> MyResult<Self> {
        let t_a = graph.iter_for_sp(s, &HAS_A).last().ok_or(MyError{})??;
        let t_a = t_a.o(); // here is where the error occures
        let a = A::from_graph(t_a, graph)?;

        let t_value = graph.iter_for_sp(s, &rdf::value).last().ok_or(MyError{})??;
        let t_value = t_value.o();
        let value = t_value.value().parse::<i32>()?;
        Ok(B { a, value })
    }
}

fn main() {
    let mut g = FastGraph::new();
    nt::parse_str(SRC).in_graph(&mut g).unwrap();
    //dbg!(&g.iter().size_hint());
    let a1 = A::from_graph(&EXD_A1, &g).unwrap();
    dbg!(&a1.value);
}