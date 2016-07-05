use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use super::sexp::Sexp;

pub struct EnvData {
    data: HashMap<String, Sexp>,
    enclosing: Option<Env>,
}

impl EnvData {
    fn new(enclosing: Option<Env>) -> EnvData {
        EnvData {
            data: HashMap::new(),
            enclosing: enclosing,
        }
    }
}

pub type Env = Rc<RefCell<EnvData>>;

pub fn env_new(enclosing: Option<Env>) -> Env {
    Rc::new(RefCell::new(EnvData::new(enclosing)))
}

pub fn env_set(env: &Env, k: String, v: Sexp) {
    env.borrow_mut().data.insert(k, v);
}

pub fn env_get(env: &Env, k: &String) -> Option<Sexp> {
    let e = env.borrow();

    match e.data.get(k) {
        Some(v) => Some((*v).clone()),
        None => {
            match e.enclosing {
                Some(ref enc) => env_get(&enc, k),
                None => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::sexp::Sexp;
    use super::{env_new, env_get, env_set};

    #[test]
    fn test_get_and_set_with_no_enclosing_env() {
        let env = env_new(None);

        assert_eq!(env_get(&env, &"k".to_string()), None);

        env_set(&env, "k".to_string(), Sexp::Number(5.0));
        assert_eq!(env_get(&env, &"k".to_string()), Some(Sexp::Number(5.0)));
    }

    #[test]
    fn test_get_and_set_with_enclosing_env() {
        let enclosing = env_new(None);
        let env = env_new(Some(enclosing.clone()));

        assert_eq!(env_get(&env, &"k".to_string()), None);

        env_set(&enclosing, "k".to_string(), Sexp::Number(5.0));
        assert_eq!(env_get(&env, &"k".to_string()), Some(Sexp::Number(5.0)));
    }
}
