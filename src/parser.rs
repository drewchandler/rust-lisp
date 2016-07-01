use std::str::{self, FromStr};
use super::sexp::Sexp;
use nom::{is_alphanumeric, is_digit, multispace};

named!(pub sexp<Sexp>, alt_complete!(number | string | symbol | list));

named!(sign, alt!(tag!("-") | tag!("+")));

named!(digits, take_while1!(is_digit));

named!(integer_with_fractional,
    recognize!(chain!(
        opt!(sign) ~
        digits ~
        tag!(".") ~
        opt!(digits),
        || ()
    )
));

named!(fractional,
    recognize!(chain!(
        opt!(sign) ~
        tag!(".") ~
        digits,
        || ()
    )
));

named!(integer,
    recognize!(chain!(
        opt!(sign) ~
        digits,
        || ()
    )
));

named!(float<f64>, map_res!(
    map_res!(
        alt_complete!(
            integer_with_fractional |
            fractional |
            integer
        ),
        str::from_utf8
    ),
    FromStr::from_str
));

named!(number<Sexp>, map!(
    float,
    Sexp::Number
));

named!(quote<char>, preceded!(char!('\\'), char!('"')));

named!(non_quote<char>, none_of!("\""));

named!(string<Sexp>, map!(
    map_res!(
        delimited!(
            char!('"'),
            recognize!(many0!(alt!(quote | non_quote))),
            char!('"')
        ),
        str::from_utf8
    ),
    |s| Sexp::String(String::from_str(s).unwrap())
));

fn is_extended(chr: u8) -> bool {
    "!$%&*+-./:<=>?@^_~".contains(chr as char)
}

fn is_extended_alphanumeric(chr: u8) -> bool {
    is_alphanumeric(chr) || is_extended(chr)
}

named!(extended_alphanumeric, take_while1!(is_extended_alphanumeric));

named!(symbol<Sexp>, map!(
    map_res!(
        extended_alphanumeric,
        str::from_utf8
    ),
    |s| Sexp::Symbol(String::from_str(s).unwrap())
));

named!(list<Sexp>, map!(
    delimited!(
        preceded!(char!('('), opt!(multispace)),
        separated_list!(multispace, sexp),
        preceded!(opt!(multispace), char!(')'))
    ),
    |sexps| Sexp::List(sexps)
));

#[cfg(test)]
mod tests {
    use super::super::sexp::Sexp;
    use super::{sexp, list, string, symbol, number};
    use nom::IResult::Done;

    #[test]
    fn test_number() {
        assert_eq!(number(b"12"), Done(&b""[..], Sexp::Number(12.)));
        assert_eq!(number(b"+12"), Done(&b""[..], Sexp::Number(12.)));
        assert_eq!(number(b"-12"), Done(&b""[..], Sexp::Number(-12.)));
        assert_eq!(number(b"12."), Done(&b""[..], Sexp::Number(12.)));
        assert_eq!(number(b"12.34"), Done(&b""[..], Sexp::Number(12.34)));
        assert_eq!(number(b"-12.34"), Done(&b""[..], Sexp::Number(-12.34)));
        assert_eq!(number(b".34"), Done(&b""[..], Sexp::Number(0.34)));
        assert_eq!(number(b"-.34"), Done(&b""[..], Sexp::Number(-0.34)));
    }

    #[test]
    fn test_string() {
        assert_eq!(string(b"\"\""), Done(&b""[..], Sexp::String("".to_string())));
        assert_eq!(string(b"\"string\""), Done(&b""[..], Sexp::String("string".to_string())));
        assert_eq!(
            string(b"\"str\\\"ing\""),
            Done(&b""[..], Sexp::String("str\\\"ing".to_string()))
        );
    }

    #[test]
    fn test_symbol() {
        assert_eq!(symbol(b"sym"), Done(&b""[..], Sexp::Symbol("sym".to_string())));
        assert_eq!(symbol(b"sym12"), Done(&b""[..], Sexp::Symbol("sym12".to_string())));
        assert_eq!(symbol(b"12sym"), Done(&b""[..], Sexp::Symbol("12sym".to_string())));
        assert_eq!(symbol(b"sym!"), Done(&b""[..], Sexp::Symbol("sym!".to_string())));
        assert_eq!(symbol(b"!sym"), Done(&b""[..], Sexp::Symbol("!sym".to_string())));
    }

    #[test]
    fn test_list() {
        assert_eq!(list(b"()"), Done(&b""[..], Sexp::List(vec![])));
        assert_eq!(list(b"(())"), Done(&b""[..], Sexp::List(vec![Sexp::List(vec![])])));
        assert_eq!(list(b"(a)"), Done(&b""[..], Sexp::List(vec![Sexp::Symbol("a".to_string())])));
        assert_eq!(
            list(b"(\ta\t)"),
            Done(&b""[..], Sexp::List(vec![Sexp::Symbol("a".to_string())]))
        );
        assert_eq!(
            list(b"(a b)"),
            Done(
                &b""[..],
                Sexp::List(vec![Sexp::Symbol("a".to_string()), Sexp::Symbol("b".to_string())])
            )
        );
        assert_eq!(
            list(b"(a\t\tb)"),
            Done(
                &b""[..],
                Sexp::List(vec![Sexp::Symbol("a".to_string()), Sexp::Symbol("b".to_string())])
            )
        );
    }

    #[test]
    fn test_sexp() {
        assert_eq!(sexp(b"12"), Done(&b""[..], Sexp::Number(12.)));
        assert_eq!(sexp(b"\"\""), Done(&b""[..], Sexp::String("".to_string())));
        assert_eq!(sexp(b"sym"), Done(&b""[..], Sexp::Symbol("sym".to_string())));
        assert_eq!(sexp(b"()"), Done(&b""[..], Sexp::List(vec![])));
    }
}
