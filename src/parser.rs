/*
* Parser
*/

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, digit1, multispace0},
    combinator::{map, recognize},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

use crate::syntax::{Expr, Primitive};

fn parse_int(input: &str) -> IResult<&str, Expr> {
    map(digit1, |s: &str| {
        Expr::Primitive(Primitive::Int(s.parse().unwrap()))
    })(input)
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
    let (input, _) = tag("def")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, name) = parse_variable(input)?;
    let (input, _) = multispace0(input)?;
    let (input, params) = delimited(
        tag("("),
        separated_list0(tag(","), preceded(multispace0, parse_variable)),
        tag(")"),
    )(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, body) = parse_expr(input)?;

    if let Expr::Variable(name) = name {
        Ok((
            input,
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
            ),
        ))
    } else {
        panic!("Expected variable name for function")
    }
}

fn parse_function_call(input: &str) -> IResult<&str, Expr> {
    let (input, func) = parse_variable(input)?;
    let (input, _) = multispace0(input)?;
    let (input, args) = delimited(
        tag("("),
        separated_list0(tag(","), preceded(multispace0, parse_expr)),
        tag(")"),
    )(input)?;

    Ok((
        input,
        Expr::FunctionCall(Box::new(func), args),
    ))
}

// fn parse_notation_def(input: &str) -> IResult<&str, Expr> {
//     let (input, _) = tag("notation")(input)?;
//     let (input, _) = multispace1(input)?;
//     let (input, name) = parse_identifier(input)?;
//     let (input, _) = multispace1(input)?;
//     let (input, precedence) = parse_int(input)?;
//     let (input, _) = multispace0(input)?;
//     let (input, _) = tag(":=")(input)?;
//     let (input, _) = multispace0(input)?;
//     let (input, pattern) = parse_expr(input)?;
//     let (input, _) = multispace0(input)?;
//     let (input, _) = tag("=>")(input)?;
//     let (input, _) = multispace0(input)?;
//     let (input, expansion) = parse_expr(input)?;
//
//     Ok((input, Expr::Notation(name, precedence, Box::new(pattern), Box::new(expansion))))
// }
//
// fn parse_notation_use(input: &str) -> IResult<&str, Expr> {
//     parse_function_call(input)
// }
//
fn parse_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_int,
        parse_function_def,
        parse_function_call,
        parse_variable,
    ))(input)
}

pub fn parse_program(input: &str) -> IResult<&str, Vec<Expr>> {
    many0(terminated(parse_expr, multispace0))(input)
}
