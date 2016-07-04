use super::env::{self, Env};
use super::sexp::{Sexp, SexpResult};

fn add(args: Vec<Sexp>) -> SexpResult {
    let numbers: Result<Vec<&f64>, String> = args.iter().map(|i| {
        match *i {
            Sexp::Number(ref n) => Ok(n),
            ref v @ _ => Err(format!("Argument {} is not a number", v))
        }
    }).collect();

    numbers.map( |ns| {
        Sexp::Number(ns.iter().fold(0., |sum, n| sum + *n))
    })
}

pub fn default_env() -> Env {
    let env = env::env_new(None);
    env::env_set(&env, "+".to_string(), Sexp::BuiltInFunc(add));

    env
}

#[cfg(test)]
mod tests {
    use super::super::sexp::Sexp;

    #[test]
    fn test_add() {
        assert_eq!(super::add(vec![Sexp::Number(1.)]), Ok(Sexp::Number(1.)));
        assert_eq!(
            super::add(vec![Sexp::Number(1.), Sexp::Number(2.), Sexp::Number(3.)]),
            Ok(Sexp::Number(6.))
        );
        assert_eq!(
            super::add(vec![Sexp::Number(1.), Sexp::Number(2.), Sexp::String("3".to_string())]),
            Err("Argument \"3\" is not a number".to_string())
        );
    }
}
