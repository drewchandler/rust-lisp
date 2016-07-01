#[derive(PartialEq, Debug)]
pub enum Sexp {
    Number(f64),
    String(String),
    Symbol(String),
    List(Vec<Sexp>),
}
