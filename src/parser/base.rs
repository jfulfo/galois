// parser/base.rs

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace1, one_of},
    combinator::{all_consuming, map, opt, recognize, value},
    error::{context, VerboseError},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use crate::syntax::{Associativity, Expr, NotationPattern, Primitive};

type ParseResult<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

fn log_parse_attempt<'a, F, O>(context: &str, mut f: F) -> impl FnMut(&'a str) -> ParseResult<'a, O>
where
    F: FnMut(&'a str) -> ParseResult<'a, O>,
{
    move |input: &'a str| {
        println!("Attempting to parse {}: {:?}", context, input);
        let result = f(input);
        match &result {
            Ok((remaining, _)) => println!(
                "Successfully parsed {}. Remaining: {:?}",
                context, remaining
            ),
            Err(e) => println!("Failed to parse {}: {:?}", context, e),
        }
        result
    }
}

fn ws(input: &str) -> ParseResult<()> {
    value(
        (),
        many0(alt((
            value((), multispace1),
            value((), preceded(tag("//"), take_until("\n"))),
            value((), delimited(tag("/*"), take_until("*/"), tag("*/"))),
        ))),
    )(input)
}

fn parse_primitive(input: &str) -> ParseResult<Expr> {
    context(
        "primitive",
        alt((
            map(parse_float, |f| Expr::Primitive(Primitive::Float(f))),
            map(parse_int, |i| Expr::Primitive(Primitive::Int(i))),
            map(parse_string, |s| Expr::Primitive(Primitive::String(s))),
            map(parse_bool, |b| Expr::Primitive(Primitive::Bool(b))),
        )),
    )(input)
}

fn parse_int(input: &str) -> ParseResult<i64> {
    context(
        "integer",
        map(recognize(pair(opt(char('-')), digit1)), |s: &str| {
            s.parse().unwrap()
        }),
    )(input)
}

fn parse_float(input: &str) -> ParseResult<f64> {
    context(
        "float",
        map(
            recognize(tuple((
                opt(char('-')),
                digit1,
                char('.'),
                digit1,
                opt(tuple((one_of("eE"), opt(one_of("+-")), digit1))),
            ))),
            |s: &str| s.parse().unwrap(),
        ),
    )(input)
}

fn parse_string(input: &str) -> ParseResult<String> {
    context(
        "string",
        delimited(
            char('"'),
            map(
                many0(alt((
                    map(take_while1(|c| c != '"' && c != '\\'), String::from),
                    map(tag("\\\""), |_| String::from("\"")),
                    map(tag("\\\\"), |_| String::from("\\")),
                    map(tag("\\n"), |_| String::from("\n")),
                    map(tag("\\r"), |_| String::from("\r")),
                    map(tag("\\t"), |_| String::from("\t")),
                ))),
                |chunks| chunks.concat(),
            ),
            char('"'),
        ),
    )(input)
}

fn parse_bool(input: &str) -> ParseResult<bool> {
    context(
        "boolean",
        alt((value(true, tag("true")), value(false, tag("false")))),
    )(input)
}

fn parse_variable(input: &str) -> ParseResult<Expr> {
    context(
        "variable",
        map(
            recognize(pair(
                alt((alpha1, tag("_"))),
                many0(alt((alphanumeric1, tag("_")))),
            )),
            |s: &str| Expr::Variable(s.to_string()),
        ),
    )(input)
}

fn parse_assignment(input: &str) -> ParseResult<Expr> {
    context(
        "assignment",
        map(
            tuple((parse_variable, delimited(ws, char('='), ws), parse_expr)),
            |(var, _, expr)| {
                if let Expr::Variable(name) = var {
                    Expr::Assignment(name, Box::new(expr))
                } else {
                    panic!("Expected variable name in assignment")
                }
            },
        ),
    )(input)
}

fn parse_function_def(input: &str) -> ParseResult<Expr> {
    context(
        "function definition",
        map(
            tuple((
                preceded(pair(tag("def"), ws), parse_variable),
                delimited(
                    char('('),
                    separated_list0(delimited(ws, char(','), ws), parse_variable),
                    char(')'),
                ),
                delimited(ws, char('{'), ws),
                parse_block,
                delimited(ws, char('}'), ws),
            )),
            |(name, params, _, body, _)| {
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
        ),
    )(input)
}

fn parse_block(input: &str) -> ParseResult<Expr> {
    context(
        "block",
        map(
            many0(terminated(
                alt((parse_function_def, parse_expr)),
                delimited(ws, opt(char(';')), ws),
            )),
            Expr::Block,
        ),
    )(input)
}

fn parse_function_call(input: &str) -> ParseResult<Expr> {
    context(
        "function call",
        map(
            pair(
                parse_variable,
                delimited(
                    char('('),
                    separated_list0(delimited(ws, char(','), ws), parse_expr),
                    char(')'),
                ),
            ),
            |(func, args)| Expr::FunctionCall(Box::new(func), args),
        ),
    )(input)
}

fn parse_return(input: &str) -> ParseResult<Expr> {
    context(
        "return",
        map(preceded(pair(tag("return"), ws), parse_expr), |expr| {
            Expr::Return(Box::new(expr))
        }),
    )(input)
}

fn parse_term(input: &str) -> ParseResult<Expr> {
    context(
        "term",
        delimited(
            ws,
            alt((
                parse_primitive,
                parse_function_call,
                parse_variable,
                delimited(char('('), parse_expr, char(')')),
            )),
            ws,
        ),
    )(input)
}

fn parse_infix_op(input: &str) -> ParseResult<&str> {
    recognize(many1(one_of("!@#$%^&*-+=|<>?/:~")))(input)
}

fn parse_infix_expr(input: &str) -> ParseResult<Expr> {
    let (input, first_term) = parse_term(input)?;
    let (input, rest) = many0(tuple((delimited(ws, parse_infix_op, ws), parse_term)))(input)?;

    Ok((
        input,
        rest.into_iter().fold(first_term, |acc, (op, term)| {
            Expr::InfixOp(Box::new(acc), op.to_string(), Box::new(term))
        }),
    ))
}

fn parse_notation_pattern(input: &str) -> ParseResult<NotationPattern> {
    context(
        "notation pattern",
        map(
            tuple((
                delimited(char('"'), take_until("\""), char('"')),
                opt(preceded(
                    delimited(ws, tag("with"), ws),
                    separated_list0(delimited(ws, char(','), ws), parse_variable),
                )),
                opt(preceded(delimited(ws, tag("precedence"), ws), parse_int)),
                opt(preceded(
                    delimited(ws, tag("associativity"), ws),
                    alt((
                        value(Associativity::Left, tag("left")),
                        value(Associativity::Right, tag("right")),
                        value(Associativity::None, tag("none")),
                    )),
                )),
            )),
            |(pattern, variables, precedence, associativity)| NotationPattern {
                pattern: pattern.to_string(),
                variables: variables
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| {
                        if let Expr::Variable(name) = v {
                            name
                        } else {
                            panic!("Expected variable in notation pattern")
                        }
                    })
                    .collect(),
                precedence: precedence.map(|p| p as i32),
                associativity: associativity.unwrap_or(Associativity::None),
            },
        ),
    )(input)
}

fn parse_notation_decl(input: &str) -> ParseResult<Expr> {
    context(
        "notation declaration",
        map(
            tuple((
                preceded(pair(tag("notation"), ws), parse_notation_pattern),
                delimited(ws, tag(":="), ws),
                parse_expr,
            )),
            |(pattern, _, expansion)| Expr::NotationDecl(pattern, Box::new(expansion)),
        ),
    )(input)
}

fn parse_expr(input: &str) -> ParseResult<Expr> {
    context(
        "expression",
        delimited(
            ws,
            alt((parse_assignment, parse_return, parse_infix_expr)),
            ws,
        ),
    )(input)
}

fn parse_top_level_expr(input: &str) -> ParseResult<Expr> {
    context(
        "top level expression",
        alt((
            parse_function_def,
            parse_notation_decl,
            terminated(parse_expr, delimited(ws, opt(char(';')), ws)),
        )),
    )(input)
}

pub fn parse_program(input: &str) -> ParseResult<Vec<Expr>> {
    context(
        "program",
        all_consuming(delimited(ws, many1(parse_top_level_expr), ws)),
    )(input)
}
