use super::env::{self, Env};
use super::sexp::{Sexp, SexpResult};

macro_rules! unpack_args {
    ($src:expr, => $t:path) => {{
        let tmp: Result<Vec<_>, String> = $src.iter()
            .map(|i| {
                match *i {
                    $t(ref n) => Ok(*n),
                    ref v @ _ => Err(format!("Argument error: {}", v)),
                }
            })
            .collect();

        if let Err(m) = tmp { return Err(m) }

        tmp.unwrap()
    }}
}

fn add(args: Vec<Sexp>) -> SexpResult {
    let ns = unpack_args!(args, => Sexp::Number);

    Ok(Sexp::Number(ns.iter().fold(0., |sum, n| sum + *n)))
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
        assert_eq!(super::add(vec![Sexp::Number(1.), Sexp::Number(2.), Sexp::Number(3.)]),
                   Ok(Sexp::Number(6.)));
        assert_eq!(super::add(vec![Sexp::Number(1.),
                                   Sexp::Number(2.),
                                   Sexp::String("3".to_string())]),
                   Err("Argument error: \"3\"".to_string()));
    }
}
