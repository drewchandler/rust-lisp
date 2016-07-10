use std::fmt;
use super::env::{self, Env};


#[derive(PartialEq, Debug, Clone)]
pub struct FuncData {
    params: Vec<String>,
    body: Box<Sexp>,
    env: Env,
}

impl FuncData {
    fn new(params: Vec<String>, body: Sexp, env: Env) -> FuncData {
        FuncData {
            params: params,
            body: Box::new(body),
            env: env
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Sexp {
    Number(f64),
    String(String),
    Symbol(String),
    List(Vec<Sexp>),
    BuiltInFunc(fn(Vec<Sexp>) -> SexpResult),
    UserFunc(FuncData),
    Nil,
    True,
}

pub type SexpResult = Result<Sexp, String>;

impl Sexp {
    pub fn eval(&self, env: &Env) -> SexpResult {
        match *self {
            ref s @ Sexp::Number(_) |
            ref s @ Sexp::String(_) |
            ref s @ Sexp::BuiltInFunc(_) |
            ref s @ Sexp::UserFunc(_) |
            ref s @ Sexp::Nil |
            ref s @ Sexp::True => Ok(s.clone()),
            Sexp::Symbol(ref s) => {
                match env::env_get(&env, &s) {
                    Some(v) => Ok(v.clone()),
                    None => Err(format!("The variable {} is unbound", &s)),
                }
            }
            Sexp::List(ref v) => {
                if v.len() == 0 {
                    return Ok(Sexp::Nil);
                }

                process_special_form(&v, &env).unwrap_or_else(|| {
                    let evaled: Result<Vec<Sexp>, String> =
                        v.iter().map(|s| s.eval(&env)).collect();
                    evaled.and_then(|v| v[0].apply(v[1..].to_vec()))
                })
            }
        }
    }

    fn apply(&self, args: Vec<Sexp>) -> SexpResult {
        match *self {
            Sexp::BuiltInFunc(f) => f(args),
            Sexp::UserFunc(ref d) => {
                let env = env::env_new(Some(d.env.clone()));
                for (k, v) in d.params.iter().zip(args.iter()) {
                    env::env_set(&env, k.clone(), v.clone());
                }
                d.body.eval(&env)
            },
            _ => Err("Illegal function call".to_string()),
        }
    }
}

impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Sexp::Number(ref n) => write!(f, "{}", n),
            Sexp::String(ref s) => write!(f, "\"{}\"", s.to_uppercase()),
            Sexp::Symbol(ref s) => write!(f, "{}", s),
            Sexp::BuiltInFunc(_) | Sexp::UserFunc(_) => write!(f, "<fn>"),
            Sexp::List(ref v) => {
                try!(write!(f, "("));
                for (i, s) in v.iter().enumerate() {
                    if i > 0 {
                        try!(write!(f, " "));
                    }
                    try!(write!(f, "{}", s));
                }
                write!(f, ")")
            }
            Sexp::Nil => write!(f, "NIL"),
            Sexp::True => write!(f, "T"),
        }
    }
}

fn process_special_form(v: &Vec<Sexp>, env: &Env) -> Option<SexpResult> {
    match v[0] {
        Sexp::Symbol(ref s) => {
            match &s[..] {
                "defparameter" => {
                    match v[1] {
                        Sexp::Symbol(ref s) => {
                            Some(v[2].eval(&env).and_then(|evaled| {
                                env::env_set(&env, s.clone(), evaled);
                                Ok(Sexp::Symbol(s.clone()))
                            }))
                        }
                        ref e @ _ => Some(Err(format!("{} is not a legal info name", e))),
                    }
                }
                "if" => {
                    match v[1] {
                        Sexp::Nil => Some(v[3].eval(&env)),
                        _ => Some(v[2].eval(&env)),
                    }
                }
                "quote" => Some(Ok(v[1].clone())),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Sexp, SexpResult, FuncData};
    use super::super::env;

    #[test]
    fn test_eval_with_self_evaluating_sexps() {
        let env = env::env_new(None);

        assert_eq!(Sexp::Number(5.).eval(&env), Ok(Sexp::Number(5.)));
        assert_eq!(Sexp::String("str".to_string()).eval(&env),
                   Ok(Sexp::String("str".to_string())));
        assert_eq!(Sexp::Nil.eval(&env), Ok(Sexp::Nil));
        assert_eq!(Sexp::True.eval(&env), Ok(Sexp::True));
    }

    #[test]
    fn test_eval_with_symbol() {
        let env = env::env_new(None);

        assert_eq!(Sexp::Symbol("sym".to_string()).eval(&env),
                   Err("The variable sym is unbound".to_string()));

        env::env_set(&env, "sym".to_string(), Sexp::Number(5.));
        assert_eq!(Sexp::Symbol("sym".to_string()).eval(&env),
                   Ok(Sexp::Number(5.)));
    }

    #[test]
    fn test_eval_with_empty_list() {
        let env = env::env_new(None);

        assert_eq!(Sexp::List(vec![]).eval(&env), Ok(Sexp::Nil));
    }

    #[test]
    fn test_eval_with_list_with_func_in_front() {
        let env = env::env_new(None);
        env::env_set(&env, "func".to_string(), Sexp::BuiltInFunc(ok));

        assert_eq!(Sexp::List(vec![Sexp::Symbol("func".to_string()), Sexp::Number(5.)]).eval(&env),
                   Ok(Sexp::Nil));
    }

    #[test]
    fn test_eval_with_list_with_func_in_front_that_errors() {
        let env = env::env_new(None);
        env::env_set(&env, "func".to_string(), Sexp::BuiltInFunc(err));

        assert_eq!(Sexp::List(vec![Sexp::Symbol("func".to_string()), Sexp::Number(5.)]).eval(&env),
                   Err("BOOM".to_string()));
    }

    #[test]
    fn test_eval_with_user_func_in_front() {
        let env = env::env_new(None);
        let func_data = FuncData::new(
            vec!["n".to_string()],
            Sexp::Symbol("n".to_string()),
            env.clone()
        );
        env::env_set(&env, "func".to_string(), Sexp::UserFunc(func_data));

        assert_eq!(Sexp::List(vec![Sexp::Symbol("func".to_string()), Sexp::Number(5.)]).eval(&env),
                   Ok(Sexp::Number(5.)));
    }

    #[test]
    fn test_eval_with_list_non_func() {
        let env = env::env_new(None);

        assert_eq!(Sexp::List(vec![Sexp::Number(5.)]).eval(&env),
                   Err("Illegal function call".to_string()));
    }

    #[test]
    fn test_eval_with_defparameter() {
        let env = env::env_new(None);

        assert_eq!(Sexp::List(vec![Sexp::Symbol("defparameter".to_string()),
                                   Sexp::Symbol("a".to_string()),
                                   Sexp::Number(5.)])
                       .eval(&env),
                   Ok(Sexp::Symbol("a".to_string())));
        assert_eq!(env::env_get(&env, &"a".to_string()), Some(Sexp::Number(5.)));

        assert_eq!(Sexp::List(vec![Sexp::Symbol("defparameter".to_string()),
                                   Sexp::Number(5.),
                                   Sexp::Number(5.)])
                       .eval(&env),
                   Err("5 is not a legal info name".to_string()));
    }

    #[test]
    fn test_eval_with_if() {
        let env = env::env_new(None);

        assert_eq!(Sexp::List(vec![Sexp::Symbol("if".to_string()),
                                   Sexp::True,
                                   Sexp::Number(1.),
                                   Sexp::Number(2.)])
                       .eval(&env),
                   Ok(Sexp::Number(1.)));
        assert_eq!(Sexp::List(vec![Sexp::Symbol("if".to_string()),
                                   Sexp::True,
                                   Sexp::Number(1.),
                                   Sexp::Number(2.)])
                       .eval(&env),
                   Ok(Sexp::Number(1.)));

        assert_eq!(Sexp::List(vec![Sexp::Symbol("if".to_string()),
                                   Sexp::Nil,
                                   Sexp::Number(1.),
                                   Sexp::Number(2.)])
                       .eval(&env),
                   Ok(Sexp::Number(2.)));
    }

    #[test]
    fn test_eval_with_quote() {
        let env = env::env_new(None);

        assert_eq!(Sexp::List(vec![Sexp::Symbol("quote".to_string()),
                                   Sexp::List(vec![Sexp::Number(5.)])])
                       .eval(&env),
                   Ok(Sexp::List(vec![Sexp::Number(5.)])));
    }

    fn ok(_: Vec<Sexp>) -> SexpResult {
        Ok(Sexp::Nil)
    }

    fn err(_: Vec<Sexp>) -> SexpResult {
        Err("BOOM".to_string())
    }
}
