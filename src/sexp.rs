use std::fmt;

#[derive(PartialEq, Debug, Clone)]
pub enum Sexp {
    Number(f64),
    String(String),
    Symbol(String),
    List(Vec<Sexp>),
    BuiltInFunc(fn(Vec<Sexp>) -> SexpResult),
    Nil,
}

impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Sexp::Number(ref n) => write!(f, "{}", n),
            Sexp::String(ref s) => write!(f, "\"{}\"", s),
            Sexp::Symbol(ref s) => write!(f, "{}", s),
            Sexp::BuiltInFunc(_) => write!(f, "<fn>"),
            Sexp::List(ref v) => {
                try!(write!(f, "("));
                for (i, s) in v.iter().enumerate() {
                    if i > 0 { try!(write!(f, " ")); }
                    try!(write!(f, "{}", s));
                }
                write!(f, ")")
            },
            Sexp::Nil => write!(f, "NIL")
        }
    }
}
