use super::env::{self, Env};
use super::sexp::{Sexp, SexpResult};

macro_rules! unpack_args {
    ($src:expr, N $rest:path) => {{
        unpack_arg!($src, 0usize, N $rest)
    }};

    ($src:expr, $(1 $t:path),+, N $rest:path) => {{
        let mut index = 0usize;

        (
            $({
                index += 1;
                unpack_arg!($src, index - 1, 1 $t)
            }),*,
            unpack_arg!($src, index, N $rest)
        )
    }};
}

macro_rules! unpack_arg {
    ($src:expr, $index:expr, N $rest:path) => {{
        let tmp: Result<Vec<_>, String> = $src.iter()
            .skip($index)
            .map(|i| {
                match *i {
                    $rest(ref n) => Ok(*n),
                    ref v @ _ => Err(format!("Argument error: {}", v)),
                }
            })
            .collect();

        if let Err(m) = tmp { return Err(m) }

        tmp.unwrap()
    }};

    ($src:expr, $index:expr, 1 $t:path) => {{
        let tmp = $src.iter().nth($index);

        if tmp.is_none() {
            return Err(format!("Invalid number of arguments: {}", $src.iter().len()))
        }

        match *tmp.unwrap() {
            $t(ref n) => *n,
            ref v @ _ => return Err(format!("Argument error: {}", v)),
        }
    }};
}

fn add(args: Vec<Sexp>) -> SexpResult {
    let ns = unpack_args!(args, N Sexp::Number);

    Ok(Sexp::Number(ns.iter().fold(0., |sum, n| sum + *n)))
}

fn subtract(args: Vec<Sexp>) -> SexpResult {
    let (first, rest) = unpack_args!(args, 1 Sexp::Number, N Sexp::Number);

    if rest.len() == 0 {
        Ok(Sexp::Number(0. - first))
    } else {
        Ok(Sexp::Number(rest.iter().fold(first, |result, n| result - *n)))
    }
}

fn multiply(args: Vec<Sexp>) -> SexpResult {
    let (first, rest) = unpack_args!(args, 1 Sexp::Number, N Sexp::Number);

    Ok(Sexp::Number(rest.iter().fold(first, |result, n| result * *n)))
}

fn divide(args: Vec<Sexp>) -> SexpResult {
    let (first, rest) = unpack_args!(args, 1 Sexp::Number, N Sexp::Number);

    if rest.iter().any(|&n| n == 0.) {
        return Err("Division by zero".to_string());
    }

    Ok(Sexp::Number(rest.iter().fold(first, |result, n| result / *n)))
}

fn numberp(args: Vec<Sexp>) -> SexpResult {
    let len = args.len();
    if len != 1 {
        return Err(format!("Invalid number of arguments: {}", len));
    }

    match args[0] {
        Sexp::Number(_) => Ok(Sexp::True),
        _ => Ok(Sexp::Nil),
    }
}

fn lt(args: Vec<Sexp>) -> SexpResult {
    let len = args.len();
    if len == 0 {
        return Err(format!("Invalid number of arguments: {}", len));
    }

    let ns = unpack_args!(args, N Sexp::Number);

    if ns.iter().zip(ns.iter().skip(1)).all(|(a, b)| a < b) {
        Ok(Sexp::True)
    } else {
        Ok(Sexp::Nil)
    }
}

fn lte(args: Vec<Sexp>) -> SexpResult {
    let len = args.len();
    if len == 0 {
        return Err(format!("Invalid number of arguments: {}", len));
    }

    let ns = unpack_args!(args, N Sexp::Number);

    if ns.iter().zip(ns.iter().skip(1)).all(|(a, b)| a <= b) {
        Ok(Sexp::True)
    } else {
        Ok(Sexp::Nil)
    }
}

pub fn default_env() -> Env {
    let env = env::env_new(None);
    env::env_set(&env, "t".to_string(), Sexp::True);

    env::env_set(&env, "+".to_string(), Sexp::BuiltInFunc(add));
    env::env_set(&env, "-".to_string(), Sexp::BuiltInFunc(subtract));
    env::env_set(&env, "*".to_string(), Sexp::BuiltInFunc(multiply));
    env::env_set(&env, "/".to_string(), Sexp::BuiltInFunc(divide));
    env::env_set(&env, "numberp".to_string(), Sexp::BuiltInFunc(numberp));
    env::env_set(&env, "<".to_string(), Sexp::BuiltInFunc(lt));
    env::env_set(&env, "<=".to_string(), Sexp::BuiltInFunc(lte));

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
        assert_eq!(super::add(vec![Sexp::String("3".to_string())]),
                   Err("Argument error: \"3\"".to_string()));
    }

    #[test]
    fn test_subtract() {
        assert_eq!(super::subtract(vec![Sexp::Number(1.)]),
                   Ok(Sexp::Number(-1.)));
        assert_eq!(super::subtract(vec![Sexp::Number(1.), Sexp::Number(2.), Sexp::Number(3.)]),
                   Ok(Sexp::Number(-4.)));
        assert_eq!(super::subtract(vec![Sexp::String("3".to_string())]),
                   Err("Argument error: \"3\"".to_string()));
        assert_eq!(super::subtract(vec![]),
                   Err("Invalid number of arguments: 0".to_string()));
    }

    #[test]
    fn test_multiply() {
        assert_eq!(super::multiply(vec![Sexp::Number(1.)]),
                   Ok(Sexp::Number(1.)));
        assert_eq!(super::multiply(vec![Sexp::Number(1.), Sexp::Number(2.), Sexp::Number(3.)]),
                   Ok(Sexp::Number(6.)));
        assert_eq!(super::multiply(vec![Sexp::String("3".to_string())]),
                   Err("Argument error: \"3\"".to_string()));
    }

    #[test]
    fn test_divide() {
        assert_eq!(super::divide(vec![Sexp::Number(1.)]), Ok(Sexp::Number(1.)));
        assert_eq!(super::divide(vec![Sexp::Number(1.), Sexp::Number(2.), Sexp::Number(3.)]),
                   Ok(Sexp::Number(1.0 / 6.0)));
        assert_eq!(super::divide(vec![]),
                   Err("Invalid number of arguments: 0".to_string()));
        assert_eq!(super::divide(vec![Sexp::String("3".to_string())]),
                   Err("Argument error: \"3\"".to_string()));
        assert_eq!(super::divide(vec![Sexp::Number(1.), Sexp::Number(0.)]),
                   Err("Division by zero".to_string()));
    }

    #[test]
    fn test_numberp() {
        assert_eq!(super::numberp(vec![Sexp::Number(1.)]), Ok(Sexp::True));
        assert_eq!(super::numberp(vec![Sexp::True]), Ok(Sexp::Nil));
        assert_eq!(super::numberp(vec![]),
                   Err("Invalid number of arguments: 0".to_string()));
        assert_eq!(super::numberp(vec![Sexp::True, Sexp::True]),
                   Err("Invalid number of arguments: 2".to_string()));
    }

    #[test]
    fn test_lt() {
        assert_eq!(super::lt(vec![Sexp::Number(1.)]), Ok(Sexp::True));
        assert_eq!(super::lt(vec![Sexp::Number(1.), Sexp::Number(2.)]), Ok(Sexp::True));
        assert_eq!(super::lt(vec![Sexp::Number(1.), Sexp::Number(2.), Sexp::Number(3.)]), Ok(Sexp::True));

        assert_eq!(super::lt(vec![Sexp::Number(2.), Sexp::Number(1.)]), Ok(Sexp::Nil));
        assert_eq!(super::lt(vec![Sexp::Number(2.), Sexp::Number(2.)]), Ok(Sexp::Nil));
        assert_eq!(super::lt(vec![Sexp::Number(1.), Sexp::Number(3.), Sexp::Number(2.)]), Ok(Sexp::Nil));

        assert_eq!(super::lt(vec![]), Err("Invalid number of arguments: 0".to_string()));
    }

    #[test]
    fn test_lte() {
        assert_eq!(super::lte(vec![Sexp::Number(1.)]), Ok(Sexp::True));
        assert_eq!(super::lte(vec![Sexp::Number(1.), Sexp::Number(2.)]), Ok(Sexp::True));
        assert_eq!(super::lte(vec![Sexp::Number(2.), Sexp::Number(2.)]), Ok(Sexp::True));
        assert_eq!(super::lte(vec![Sexp::Number(1.), Sexp::Number(2.), Sexp::Number(3.)]), Ok(Sexp::True));

        assert_eq!(super::lte(vec![Sexp::Number(2.), Sexp::Number(1.)]), Ok(Sexp::Nil));
        assert_eq!(super::lte(vec![Sexp::Number(1.), Sexp::Number(3.), Sexp::Number(2.)]), Ok(Sexp::Nil));

        assert_eq!(super::lte(vec![]), Err("Invalid number of arguments: 0".to_string()));
    }
}
