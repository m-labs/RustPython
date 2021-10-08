//! Python parsing.
//!
//! Use this module to parse python code into an AST.
//! There are three ways to parse python code. You could
//! parse a whole program, a single statement, or a single
//! expression.

use std::iter;

use crate::ast;
use crate::error::ParseError;
use crate::lexer;
pub use crate::mode::Mode;
use crate::python;

/*
 * Parse python code.
 * Grammar may be inspired by antlr grammar for python:
 * https://github.com/antlr/grammars-v4/tree/master/python3
 */

/// Parse a full python program, containing usually multiple lines.
pub fn parse_program(source: &str) -> Result<ast::Suite, ParseError> {
    parse(source, Mode::Module).map(|top| match top {
        ast::Mod::Module { body, .. } => body,
        _ => unreachable!(),
    })
}

/// Parses a python expression
///
/// # Example
/// ```
/// extern crate num_bigint;
/// use rustpython_parser::{parser, ast};
/// let expr = parser::parse_expression("1 + 2").unwrap();
///
/// assert_eq!(
///     expr,
///     ast::Expr {
///         location: ast::Location::new(1, 3),
///         custom: (),
///         node: ast::ExprKind::BinOp {
///             left: Box::new(ast::Expr {
///                 location: ast::Location::new(1, 1),
///                 custom: (),
///                 node: ast::ExprKind::Constant {
///                     value: ast::Constant::Int(1.into()),
///                     kind: None,
///                 }
///             }),
///             op: ast::Operator::Add,
///             right: Box::new(ast::Expr {
///                 location: ast::Location::new(1, 5),
///                 custom: (),
///                 node: ast::ExprKind::Constant {
///                     value: ast::Constant::Int(2.into()),
///                     kind: None,
///                 }
///             })
///         }
///     },
/// );
///
/// ```
pub fn parse_expression(source: &str) -> Result<ast::Expr, ParseError> {
    parse(source, Mode::Expression).map(|top| match top {
        ast::Mod::Expression { body } => *body,
        _ => unreachable!(),
    })
}

// Parse a given source code
pub fn parse(source: &str, mode: Mode) -> Result<ast::Mod, ParseError> {
    let lxr = lexer::make_tokenizer(source, Some(" nac3:"));
    let marker_token = (Default::default(), mode.to_marker(), Default::default());
    let tokenizer = iter::once(Ok(marker_token)).chain(lxr);

    python::TopParser::new()
        .parse(tokenizer)
        .map_err(ParseError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let parse_ast = parse_program("").unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_print_hello() {
        let source = String::from("print('Hello world')");
        let parse_ast = parse_program(&source).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_print_2() {
        let source = String::from("print('Hello world', 2)");
        let parse_ast = parse_program(&source).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_kwargs() {
        let source = String::from("my_func('positional', keyword=2)");
        let parse_ast = parse_program(&source).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_if_elif_else() {
        let source = String::from("if 1: 10\nelif 2: 20\nelse: 30");
        let parse_ast = parse_program(&source).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_lambda() {
        let source = "lambda x, y: x * y"; // lambda(x, y): x * y";
        let parse_ast = parse_program(&source).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_tuples() {
        let source = "a, b = 4, 5";

        insta::assert_debug_snapshot!(parse_program(&source).unwrap());
    }

    #[test]
    fn test_parse_class() {
        let source = "\
class Foo(A, B):
 def __init__(self):
  pass
 def method_with_default(self, arg='default'):
  pass";
        insta::assert_debug_snapshot!(parse_program(&source).unwrap());
    }

    #[test]
    fn test_parse_dict_comprehension() {
        let source = String::from("{x1: x2 for y in z}");
        let parse_ast = parse_expression(&source).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_list_comprehension() {
        let source = String::from("[x for y in z]");
        let parse_ast = parse_expression(&source).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_double_list_comprehension() {
        let source = String::from("[x for y, y2 in z for a in b if a < 5 if a > 10]");
        let parse_ast = parse_expression(&source).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_confog_comment() {
        let source = "\
class A:
    # nac3: test
    def fun():
        # nac3: unroll
    # normal comment
        for i in (1, '123'):
            # nac3: test
        ";
        insta::assert_debug_snapshot!(parse_program(&source).unwrap());
    }
}
