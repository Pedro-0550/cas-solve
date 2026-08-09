// use std::{
//     fmt::Display,
//     ops::{Add, Mul},
// };

// use num::complex::Complex64;

// use crate::{
//     Complex,
//     dimension::{Dimension, DimensionalAnalysisError, Quantity},
//     intrinsic::Intrinsic,
//     symbol::{Symbol, SymbolicContext},
// };

// /* ---------------------------------- ENUMS --------------------------------- */
// #[derive(PartialEq, Clone, Debug)]
// pub enum Expr {
//     Symbol(Symbol),
//     Const(Quantity),

//     Func(Box<Function>),
//     ElementwiseFunc(Box<Function>),

//     Intrinsic(Box<Intrinsic>),
//     ElementwiseIntrinsic(Box<Intrinsic>),

//     Matrix { rows: usize, cols: usize, elements: Vec<Expr> },
// }

// /* --------------------------------- STRUCTS -------------------------------- */
// #[derive(PartialEq, Clone, Debug)]
// pub struct Function {
//     arguments: Vec<Symbol>,
//     value: Expr,
// }

// /* ---------------------------------- IMPLS --------------------------------- */
// impl Expr {
//     pub fn simplify(self) -> Expr {
//         fn simplify_inner(expr: Expr) -> Expr {
//             match expr {
//                 Expr::Intrinsic(intrinsic) => intrinsic.simplify(),
//                 Expr::Const(_) | Expr::Symbol(_) => expr,
//                 _ => todo!(),
//             }
//         }

//         let mut simplified = simplify_inner(self);

//         loop {
//             let step = simplify_inner(simplified.clone());

//             if simplified == step {
//                 break;
//             }

//             simplified = step;
//         }

//         simplified
//     }

//     pub fn to_string(&self, ctx: &SymbolicContext) -> String {
//         let mut buf = String::new();
//         self.format(&mut buf, ctx);
//         buf
//     }

//     pub fn format(&self, buf: &mut String, ctx: &SymbolicContext) {
//         match self {
//             Expr::Symbol(symb) => buf.extend(ctx.name(symb)),
//             Expr::Intrinsic(intr) => {
//                 buf.push('(');
//                 intr.format(buf, ctx);
//                 buf.push(')')
//             }
//             Expr::Const(val) => {
//                 if val.im == 0.0 {
//                     buf.push_str(&val.re.to_string());
//                 } else if val.re == 0.0 {
//                     buf.push_str(&val.re.to_string());
//                     buf.push('i');
//                 } else {
//                     buf.push('(');
//                     buf.push_str(&val.to_string());
//                     buf.push(')')
//                 }
//             }
//             _ => todo!(),
//         }
//     }

//     /// Returns `true` if the expr is [`Symbol`].
//     ///
//     /// [`Symbol`]: Expr::Symbol
//     #[must_use]
//     pub fn is_symbol(&self) -> bool {
//         matches!(self, Self::Symbol(..))
//     }

//     pub fn as_symbol(&self) -> Option<&Symbol> {
//         if let Self::Symbol(v) = self { Some(v) } else { None }
//     }

//     /// Returns `true` if the expr is [`Const`].
//     ///
//     /// [`Const`]: Expr::Const
//     #[must_use]
//     pub fn is_const(&self) -> bool {
//         matches!(self, Self::Const(..))
//     }

//     pub fn as_const(&self) -> Option<&Complex> {
//         if let Self::Const(v) = self { Some(v) } else { None }
//     }

//     /// Returns `true` if the expr is [`Func`].
//     ///
//     /// [`Func`]: Expr::Func
//     #[must_use]
//     pub fn is_func(&self) -> bool {
//         matches!(self, Self::Func(..))
//     }

//     pub fn as_func(&self) -> Option<&Box<Function>> {
//         if let Self::Func(v) = self { Some(v) } else { None }
//     }

//     /// Returns `true` if the expr is [`ElementwiseFunc`].
//     ///
//     /// [`ElementwiseFunc`]: Expr::ElementwiseFunc
//     #[must_use]
//     pub fn is_elementwise_func(&self) -> bool {
//         matches!(self, Self::ElementwiseFunc(..))
//     }

//     pub fn as_elementwise_func(&self) -> Option<&Box<Function>> {
//         if let Self::ElementwiseFunc(v) = self { Some(v) } else { None }
//     }

//     /// Returns `true` if the expr is [`Intrinsic`].
//     ///
//     /// [`Intrinsic`]: Expr::Intrinsic
//     #[must_use]
//     pub fn is_intrinsic(&self) -> bool {
//         matches!(self, Self::Intrinsic(..))
//     }

//     pub fn as_intrinsic(&self) -> Option<&Box<Intrinsic>> {
//         if let Self::Intrinsic(v) = self { Some(v) } else { None }
//     }

//     /// Returns `true` if the expr is [`ElementwiseIntrinsic`].
//     ///
//     /// [`ElementwiseIntrinsic`]: Expr::ElementwiseIntrinsic
//     #[must_use]
//     pub fn is_elementwise_intrinsic(&self) -> bool {
//         matches!(self, Self::ElementwiseIntrinsic(..))
//     }

//     pub fn as_elementwise_intrinsic(&self) -> Option<&Box<Intrinsic>> {
//         if let Self::ElementwiseIntrinsic(v) = self { Some(v) } else { None }
//     }

//     /// Returns `true` if the expr is [`Matrix`].
//     ///
//     /// [`Matrix`]: Expr::Matrix
//     #[must_use]
//     pub fn is_matrix(&self) -> bool {
//         matches!(self, Self::Matrix { .. })
//     }
// }

// macro_rules! impl_expr_op {
//     ($trait:ident, $method:ident, $intrinsic:path, $rhs:ty, commutative) => {
//         impl const std::ops::$trait<$rhs> for Expr {
//             type Output = Expr;

//             fn $method(self, rhs: $rhs) -> Self::Output {
//                 Expr::Intrinsic(Box::new($intrinsic(vec![
//                     self,
//                     Expr::from(rhs),
//                 ])))
//             }
//         }

//         impl const std::ops::$trait<Expr> for $rhs {
//             type Output = Expr;

//             fn $method(self, rhs: Expr) -> Self::Output {
//                 Expr::Intrinsic(Box::new($intrinsic(vec![
//                     Expr::from(self),
//                     rhs,
//                 ])))
//             }
//         }
//     };

//     ($trait:ident, $method:ident, $intrinsic:path, $rhs:ty) => {
//         impl const std::ops::$trait<$rhs> for Expr {
//             type Output = Expr;

//             fn $method(self, rhs: $rhs) -> Self::Output {
//                 Expr::Intrinsic(Box::new($intrinsic(vec![
//                     self,
//                     Expr::from(rhs),
//                 ])))
//             }
//         }
//     };
// }

// impl_expr_op!(Add, add, Intrinsic::Add, Symbol);
// impl_expr_op!(Add, add, Intrinsic::Add, Expr);
// impl_expr_op!(Add, add, Intrinsic::Add, Complex, commutative);
// impl_expr_op!(Add, add, Intrinsic::Add, f64, commutative);

// impl_expr_op!(Mul, mul, Intrinsic::Mul, Symbol);
// impl_expr_op!(Mul, mul, Intrinsic::Mul, Expr);
// impl_expr_op!(Mul, mul, Intrinsic::Mul, Complex, commutative);
// impl_expr_op!(Mul, mul, Intrinsic::Mul, f64, commutative);

// impl const From<Symbol> for Expr {
//     fn from(value: Symbol) -> Self {
//         Expr::Symbol(value)
//     }
// }

// impl const From<Complex> for Expr {
//     fn from(value: Complex) -> Self {
//         Expr::Const(value)
//     }
// }

// impl const From<f64> for Expr {
//     fn from(value: f64) -> Self {
//         Expr::Const(Complex { re: value, im: 0.0 })
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::{
//         dimension::{DIMENSIONLESS, Dimension},
//         symbol::{Symbol, SymbolicContext},
//     };
//     #[test]
//     pub fn simplify_distributive() {
//         let mut ctx = SymbolicContext::new();
//         let a = ctx.symbol("a", DIMENSIONLESS);
//         let b = ctx.symbol("b", DIMENSIONLESS);
//         let c = ctx.symbol("c", DIMENSIONLESS);
//         let d = ctx.symbol("d", DIMENSIONLESS);

//         let expr = (a + b) * (c + d);
//         let simplified_expr = expr.clone().simplify();

//         // panic!("{}\n{}", expr.to_string(&ctx), simplified_expr.to_string(&ctx));
//         assert_eq!(simplified_expr, (a * c + a * d + b * c + b * d).simplify())
//     }

//     #[test]
//     pub fn simplify_folding() {
//         let mut ctx = SymbolicContext::new();
//         let a = ctx.symbol("a", DIMENSIONLESS);
//         let b = ctx.symbol("b", DIMENSIONLESS);

//         let expr = -5.0 + a + b + 10.0;
//         let simplified_expr = expr.simplify();
//     }
// }
