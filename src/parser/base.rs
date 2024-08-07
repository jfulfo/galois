// parser/base.rs

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace1},
    combinator::{map, opt, recognize},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

use crate::syntax::{Expr, Primitive};

fn parse_comment(input: &str) -> IResult<&str, ()> {
    alt((
        // Single-line comment
        map(preceded(tag("//"), take_until("\n")), |_| ()),
        // Multi-line comment
        map(delimited(tag("/*"), take_until("*/"), tag("*/")), |_| ()),
    ))(input)
}

fn ws(input: &str) -> IResult<&str, ()> {
    map(
        many0(alt((map(multispace1, |_| ()), parse_comment))),
        |_| (),
    )(input)
}

fn parse_primitive(input: &str) -> IResult<&str, Expr> {
    alt((
        map(parse_int, |i| Expr::Primitive(Primitive::Int(i))),
        map(parse_float, |f| Expr::Primitive(Primitive::Float(f))),
        map(parse_string, |s| Expr::Primitive(Primitive::String(s))),
        map(parse_bool, |b| Expr::Primitive(Primitive::Bool(b))),
    ))(input)
}

fn parse_int(input: &str) -> IResult<&str, i64> {
    map(recognize(pair(opt(char('-')), digit1)), |s: &str| {
        s.parse().unwrap()
    })(input)
}

fn parse_float(input: &str) -> IResult<&str, f64> {
    map(
        recognize(tuple((
            opt(char('-')),
            digit1,
            char('.'),
            digit1,
            opt(tuple((char('e'), opt(alt((char('+'), char('-')))), digit1))),
        ))),
        |s: &str| s.parse().unwrap(),
    )(input)
}

fn parse_string(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        map(
            many0(alt((
                map(take_until("\\\""), String::from),
                map(tag("\\\""), |_| String::from("\"")),
            ))),
            |v| v.concat(),
        ),
        char('"'),
    )(input)
}

fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(input)
}

fn parse_variable(input: &str) -> IResult<&str, Expr> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| Expr::Variable(s.to_string()),
    )(input)
}

fn parse_function_def(input: &str) -> IResult<&str, Expr> {
    map(
        tuple((
            preceded(pair(tag("def"), ws), parse_variable),
            delimited(
                char('('),
                separated_list0(delimited(ws, char(','), ws), parse_variable),
                char(')'),
            ),
            preceded(delimited(ws, char(':'), ws), parse_expr),
        )),
        |(name, params, body)| {
            if let Expr::Variable(name) = name {
                Expr::FunctionDef(
                    name,
                    params
                        .into_iter()
                        .map(|e| {
                            if let Expr::Variable(name) = e {
                                name
                            } else {
                                panic!("Expected variable in function parameters")
                            }
                        })
                        .collect(),
                    Box::new(body),
                )
            } else {
                panic!("Expected variable name for function")
            }
        },
    )(input)
}

fn parse_function_call(input: &str) -> IResult<&str, Expr> {
    map(
        pair(
            parse_term,
            delimited(
                char('('),
                separated_list0(delimited(ws, char(','), ws), parse_expr),
                char(')'),
            ),
        ),
        |(func, args)| Expr::FunctionCall(Box::new(func), args),
    )(input)
}

fn parse_return(input: &str) -> IResult<&str, Expr> {
    map(preceded(pair(tag("return"), ws), parse_expr), |expr| {
        Expr::Return(Box::new(expr))
    })(input)
}

fn parse_term(input: &str) -> IResult<&str, Expr> {
    delimited(
        ws,
        alt((
            parse_primitive,
            parse_variable,
            delimited(char('('), parse_expr, char(')')),
        )),
        ws,
    )(input)
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    delimited(
        ws,
        alt((
            parse_function_def,
            parse_function_call,
            parse_return,
            parse_term,
        )),
        ws,
    )(input)
}

pub fn parse_program(input: &str) -> IResult<&str, Vec<Expr>> {
    many1(parse_expr)(input)
}
