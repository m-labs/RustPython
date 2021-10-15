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
pub fn parse_program(source: &str, config_comment_prefix: Option<&'static str>) -> Result<ast::Suite, ParseError> {
    parse(source, Mode::Module, config_comment_prefix).map(|top| match top {
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
/// let expr = parser::parse_expression("1 + 2", Some("prefix")).unwrap();
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
pub fn parse_expression(source: &str, config_comment_prefix: Option<&'static str>) -> Result<ast::Expr, ParseError> {
    parse(source, Mode::Expression, config_comment_prefix).map(|top| match top {
        ast::Mod::Expression { body } => *body,
        _ => unreachable!(),
    })
}

// Parse a given source code
pub fn parse(source: &str, mode: Mode, config_comment_prefix: Option<&'static str>) -> Result<ast::Mod, ParseError> {
    let lxr = lexer::make_tokenizer(source, config_comment_prefix);
    let marker_token = (Default::default(), mode.to_marker(), Default::default());
    let tokenizer = iter::once(Ok(marker_token)).chain(lxr);

    python::TopParser::new()
        .parse(tokenizer)
        .map_err(ParseError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn lex_source(source: &str) -> Vec<lexer::Tok> {
        let lexer = lexer::make_tokenizer(source, Some(" nac3:"));
        lexer.map(|x| x.unwrap().1).collect()
    }

    #[test]
    fn test_parse_empty() {
        let parse_ast = parse_program("", Some(" nac3:")).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_print_hello() {
        let source = String::from("print('Hello world')");
        let parse_ast = parse_program(&source, Some(" nac3:")).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_print_2() {
        let source = String::from("print('Hello world', 2)");
        let parse_ast = parse_program(&source, Some(" nac3:")).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_kwargs() {
        let source = String::from("my_func('positional', keyword=2)");
        let parse_ast = parse_program(&source, Some(" nac3:")).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_if_elif_else() {
        let source = String::from("\
if 1:
    10
elif 2:
    20
else:
    30");
        let parse_ast = parse_program(&source, Some(" nac3:")).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_lambda() {
        let source = "lambda x, y: x * y"; // lambda(x, y): x * y";
        let parse_ast = parse_program(&source, Some(" nac3:")).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_tuples() {
        let source = "a, b = 4, 5";

        insta::assert_debug_snapshot!(parse_program(&source, Some(" nac3:")).unwrap());
    }

    #[test]
    fn test_parse_class() {
        let source = "\
class Foo(A, B):
 def __init__(self):
  pass
 def method_with_default(self, arg='default'):
  pass";
        insta::assert_debug_snapshot!(parse_program(&source, Some(" nac3:")).unwrap());
    }

    #[test]
    fn test_parse_dict_comprehension() {
        let source = String::from("{x1: x2 for y in z}");
        let parse_ast = parse_expression(&source, Some(" nac3:")).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_list_comprehension() {
        let source = String::from("[x for y in z]");
        let parse_ast = parse_expression(&source, Some(" nac3:")).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_double_list_comprehension() {
        let source = String::from("[x for y, y2 in z for a in b if a < 5 if a > 10]");
        let parse_ast = parse_expression(&source, Some(" nac3:")).unwrap();
        insta::assert_debug_snapshot!(parse_ast);
    }

    #[test]
    fn test_parse_class_with_nac3comment() {
        let source = "\
class Foo(A, B):
    a: int32 # nac3: no_warn_unused_1
# normal comment
    # nomal indent comment
    # nac3: no_warn_unused_2
# normal comment
    # normal indent comment
    b: int64

    c: int32
    def __init__(self):
        # nac3: unroll
        for p in ('123', 2):
            pass";
        insta::assert_debug_snapshot!(parse_program(&source, Some(" nac3:")).unwrap());
    }

    #[test]
    fn test_nac3_error() {
        let source = "\
class Foo(A, B):
    a: int32 # nac3: no_warn_unused_1
# normal comment
    # nomal indent comment
    # nac3: no_warn_unused_2
# normal comment
    # normal indent comment
    b: int64

    c: int32
    def __init__(self):
        # nac3: unroll
        for p in ('123', 2):
        # normal comment
            # nac3: unroll
            for pp in (2, '123'):
                b = 3
                # nac3: cannot comment
                a = 3";
        parse_program(&source, Some(" nac3:")).unwrap();
    }
    
    #[test]
    fn test_more_comment() {
        let source = "\
a: int # nac3: sf1
# nac3: sdf4
for i in (1, '12'): # nac3: sf2
    a: int
# nac3: 3
# nac3: 5
while i < 2: # nac3: 4
    # nac3: real pass
    pass
    # nac3: expr1
    # nac3: expr3
    1 + 2 # nac3: expr2
    # nac3: if3
    # nac3: if1
    if 1: # nac3: if2
        3";
        insta::assert_debug_snapshot!(parse_program(&source, Some(" nac3:")).unwrap());
    }

    #[test]
    fn test_comment_semicolon() {
        let source = "\
for i in ('12'):
    # nac3: comment
    i = i;
    # nac3: cc
    print(i)
";
        insta::assert_debug_snapshot!(parse_program(&source, Some(" nac3:")).unwrap());
    }
    
    #[test]
    fn test_comment_sameline() {
        let source = "\
while 1:
    a + 1;
    # nac3: pass
    pass; pass # nac3: second pass
    # nac3: assign
    a = 3; b = a + 2
    # nac3: del
    del a
    # nac3: sameline if
    if 1: b = b + 3

if 1: # nac3: s
    a
";
        insta::assert_debug_snapshot!(parse_program(&source, Some(" nac3:")).unwrap());
    }

    #[test]
    fn test_sample_comment() {
        let source = "\
# nac3: while1 
# nac3: while2 
# normal comment 
while test: # nac3: while3 
    # nac3: simple assign0 
    a = 3 # nac3: simple assign1
";
        insta::assert_debug_snapshot!(parse_program(&source, Some(" nac3:")).unwrap());
    }

    #[test]
    fn test_comment_ambiguity() {
        let source = "\
if a: d; # nac3: for d
if b: c # nac3: for c
if d: # nac3: for if d
    b; b + 3; # nac3: for b + 3
a = 3; a + 3; b = a; # nac3: notif
# nac3: smallsingle1
# nac3: smallsingle3
aa = 3 # nac3: smallsingle2
if a: # nac3: small2
    a
for i in a: # nac3: for1
    pass
";
        println!("{:?}", lex_source(source));
        insta::assert_debug_snapshot!(parse_program(&source, Some(" nac3:")).unwrap());
    }
}
