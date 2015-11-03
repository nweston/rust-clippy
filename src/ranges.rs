use rustc::lint::*;
use rustc_front::hir::*;
use syntax::codemap::Spanned;
use utils::{is_integer_literal, match_type, snippet};

declare_lint! {
    pub RANGE_STEP_BY_ZERO, Warn,
    "using Range::step_by(0), which produces an infinite iterator"
}
declare_lint! {
    pub RANGE_ZIP_WITH_LEN, Warn,
    "zipping iterator with a range when enumerate() would do"
}

#[derive(Copy,Clone)]
pub struct StepByZero;

impl LintPass for StepByZero {
    fn get_lints(&self) -> LintArray {
        lint_array!(RANGE_STEP_BY_ZERO, RANGE_ZIP_WITH_LEN)
    }
}

impl LateLintPass for StepByZero {
    fn check_expr(&mut self, cx: &LateContext, expr: &Expr) {
        if let ExprMethodCall(Spanned { node: ref name, .. }, _,
                              ref args) = expr.node {
            // Range with step_by(0).
            if name.as_str() == "step_by" && args.len() == 2 &&
                is_range(cx, &args[0]) && is_integer_literal(&args[1], 0) {
                cx.span_lint(RANGE_STEP_BY_ZERO, expr.span,
                             "Range::step_by(0) produces an infinite iterator. \
                              Consider using `std::iter::repeat()` instead")
            }

            // x.iter().zip(0..x.len())
            else if name.as_str() == "zip" && args.len() == 2 {
                let iter = &args[0].node;
                let zip_arg = &args[1].node;
                if_let_chain! {
                    [
                        // .iter() call
                        let &ExprMethodCall( Spanned { node: ref iter_name, .. }, _, ref iter_args ) = iter,
                        iter_name.as_str() == "iter",
                        // range expression in .zip() call: 0..x.len()
                        let &ExprRange(Some(ref from), Some(ref to)) = zip_arg,
                        is_integer_literal(from, 0),
                        // .len() call
                        let ExprMethodCall(Spanned { node: ref len_name, .. }, _, ref len_args) = to.node,
                        len_name.as_str() == "len" && len_args.len() == 1,
                        // .iter() and .len() called on same object
                        snippet(cx, iter_args[0].span, "x") == snippet(cx, len_args[0].span, "y")
                     ], {
                        cx.span_lint(RANGE_ZIP_WITH_LEN, expr.span,
                                     &format!("It is more idiomatic to use {}.iter().enumerate()",
                                              snippet(cx, iter_args[0].span, "_")));
                    }
                }
            }
        }
    }
}

fn is_range(cx: &LateContext, expr: &Expr) -> bool {
    // No need for walk_ptrs_ty here because step_by moves self, so it
    // can't be called on a borrowed range.
    let ty = cx.tcx.expr_ty(expr);
    // Note: RangeTo and RangeFull don't have step_by
    match_type(cx, ty, &["core", "ops", "Range"]) || match_type(cx, ty, &["core", "ops", "RangeFrom"])
}
