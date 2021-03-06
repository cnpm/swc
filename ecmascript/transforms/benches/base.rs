#![feature(test)]
extern crate test;

use swc_common::{FileName, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_transforms::{pass::noop, util::ExprFactory};
use swc_ecma_visit::{FoldWith, Node, Visit, VisitWith};
use test::Bencher;

static SOURCE: &str = r#"
'use strict';
/**
 * Extract red color out of a color integer:
 *
 * 0x00DEAD -> 0x00
 *
 * @param  {Number} color
 * @return {Number}
 */
function red( color )
{
    let foo = 3.14;
    return color >> 16;
}
/**
 * Extract green out of a color integer:
 *
 * 0x00DEAD -> 0xDE
 *
 * @param  {Number} color
 * @return {Number}
 */
function green( color )
{
    return ( color >> 8 ) & 0xFF;
}
/**
 * Extract blue color out of a color integer:
 *
 * 0x00DEAD -> 0xAD
 *
 * @param  {Number} color
 * @return {Number}
 */
function blue( color )
{
    return color & 0xFF;
}
/**
 * Converts an integer containing a color such as 0x00DEAD to a hex
 * string, such as '#00DEAD';
 *
 * @param  {Number} int
 * @return {String}
 */
function intToHex( int )
{
    const mask = '#000000';
    const hex = int.toString( 16 );
    return mask.substring( 0, 7 - hex.length ) + hex;
}
/**
 * Converts a hex string containing a color such as '#00DEAD' to
 * an integer, such as 0x00DEAD;
 *
 * @param  {Number} num
 * @return {String}
 */
function hexToInt( hex )
{
    return parseInt( hex.substring( 1 ), 16 );
}
module.exports = {
    red,
    green,
    blue,
    intToHex,
    hexToInt,
};"#;

#[bench]
fn module_clone(b: &mut Bencher) {
    b.bytes = SOURCE.len() as _;

    let _ = ::testing::run_test(false, |cm, handler| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());
        let lexer = Lexer::new(
            Syntax::default(),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .map_err(|e| {
                e.into_diagnostic(handler).emit();
            })
            .unwrap();

        for e in parser.take_errors() {
            e.into_diagnostic(handler).emit();
        }

        b.iter(|| test::black_box(module.clone()));
        Ok(())
    });
}

#[bench]
fn fold_empty(b: &mut Bencher) {
    b.bytes = SOURCE.len() as _;

    let _ = ::testing::run_test(false, |cm, handler| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());
        let lexer = Lexer::new(
            Syntax::default(),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .map_err(|e| {
                e.into_diagnostic(&handler).emit();
            })
            .unwrap();

        for e in parser.take_errors() {
            e.into_diagnostic(&handler).emit();
        }

        let mut folder = noop();

        b.iter(|| test::black_box(module.clone().fold_with(&mut folder)));
        Ok(())
    });
}

/// Optimized out
#[bench]
fn fold_noop_impl_all(b: &mut Bencher) {
    b.bytes = SOURCE.len() as _;

    let _ = ::testing::run_test(false, |cm, handler| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());

        let lexer = Lexer::new(
            Syntax::default(),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .map_err(|e| e.into_diagnostic(&handler).emit())
            .unwrap();

        for e in parser.take_errors() {
            e.into_diagnostic(&handler).emit();
        }

        let mut folder = noop();

        b.iter(|| test::black_box(module.clone().fold_with(&mut folder)));
        Ok(())
    });
}

/// Optimized out
#[bench]
fn fold_noop_impl_vec(b: &mut Bencher) {
    b.bytes = SOURCE.len() as _;

    let _ = ::testing::run_test(false, |cm, handler| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());
        let lexer = Lexer::new(
            Syntax::default(),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .map_err(|e| {
                e.into_diagnostic(&handler).emit();
            })
            .unwrap();

        for e in parser.take_errors() {
            e.into_diagnostic(&handler).emit();
        }

        let mut folder = noop();

        b.iter(|| test::black_box(module.clone().fold_with(&mut folder)));
        Ok(())
    });
}

fn mk_expr() -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Ident::new("foo".into(), DUMMY_SP).as_callee(),
        args: vec![],
        type_args: None,
    })
}

#[bench]
fn boxing_boxed_clone(b: &mut Bencher) {
    let _ = ::testing::run_test(false, |_, _| {
        let expr = Box::new(mk_expr());

        b.iter(|| test::black_box(expr.clone()));
        Ok(())
    });
}

#[bench]
fn boxing_unboxed_clone(b: &mut Bencher) {
    let _ = ::testing::run_test(false, |_, _| {
        let expr = mk_expr();

        b.iter(|| test::black_box(expr.clone()));
        Ok(())
    });
}

#[bench]
fn boxing_boxed(b: &mut Bencher) {
    let _ = ::testing::run_test(false, |_, _| {
        let mut folder = noop();
        let expr = Box::new(mk_expr());

        b.iter(|| test::black_box(expr.clone().fold_with(&mut folder)));
        Ok(())
    });
}

#[bench]
fn boxing_unboxed(b: &mut Bencher) {
    let _ = ::testing::run_test(false, |_, _| {
        let mut folder = noop();
        let expr = mk_expr();

        b.iter(|| test::black_box(expr.clone().fold_with(&mut folder)));
        Ok(())
    });
}

#[bench]
fn visit_empty(b: &mut Bencher) {
    b.bytes = SOURCE.len() as _;

    let _ = ::testing::run_test(false, |cm, _| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());
        let lexer = Lexer::new(
            Syntax::default(),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let _module = parser.parse_module().map_err(|_| ()).unwrap();

        b.iter(|| test::black_box(()));
        Ok(())
    });
}

#[bench]
fn visit_contains_this(b: &mut Bencher) {
    fn contains_this_expr(body: &Module) -> bool {
        struct Visitor {
            found: bool,
        }

        impl Visit for Visitor {
            /// Don't recurse into fn
            fn visit_fn_expr(&mut self, _: &FnExpr, _: &dyn Node) {}

            /// Don't recurse into fn
            fn visit_fn_decl(&mut self, _: &FnDecl, _: &dyn Node) {}

            fn visit_this_expr(&mut self, _: &ThisExpr, _: &dyn Node) {
                self.found = true;
            }
        }

        let mut visitor = Visitor { found: false };
        body.visit_with(&Invalid { span: DUMMY_SP } as _, &mut visitor);
        visitor.found
    }

    b.bytes = SOURCE.len() as _;

    let _ = ::testing::run_test(false, |cm, _| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());
        let lexer = Lexer::new(
            Syntax::default(),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().map_err(|_| ()).unwrap();

        b.iter(|| test::black_box(contains_this_expr(&module)));
        Ok(())
    });
}
