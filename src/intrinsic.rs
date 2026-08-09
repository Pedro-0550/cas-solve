// use std::ops::Add;

// use itertools::Itertools;
// use num::traits::{ConstOne, ConstZero};

// use crate::{
//     Complex, dimension::Quantity, expr::Expr, symbol::SymbolicContext,
// };

// /* ---------------------------------- ENUMS --------------------------------- */
// #[derive(PartialEq, Clone, Debug)]
// pub enum Intrinsic {
//     Add(Vec<Expr>),
//     Mul(Vec<Expr>),
//     Div { num: Expr, denom: Expr },
//     Neg(Expr),

//     Sin(Expr),
//     Cos(Expr),
//     Asin(Expr),
//     Acos(Expr),

//     Pow { base: Expr, exp: Expr },
//     Log { base: Expr, arg: Expr },
//     Norm(Expr),

//     Inv(Expr),
//     Transpose(Expr),
// }

// /* ---------------------------------- IMPLS --------------------------------- */
// impl Intrinsic {
//     pub fn simplify(self) -> Expr {
//         match self {
//             Intrinsic::Add(operands) => {
//                 let flattened = operands
//                     .into_iter()
//                     .flat_map(|expr| {
//                         expr.as_intrinsic()
//                             .and_then(|intr| intr.as_add().cloned())
//                             .unwrap_or(vec![expr])
//                     })
//                     .collect::<Vec<_>>();

//                 let (mut simplified, folded_const) =
//                     fold_consts(flattened, Complex::ZERO, |accum, x| accum + x);

//                 if folded_const != Complex::ZERO {
//                     simplified.push(Expr::Const(folded_const));
//                 }

//                 if simplified.len() <= 1 {
//                     return simplified
//                         .try_remove(0)
//                         .unwrap_or(Expr::Const(Complex::ZERO));
//                 } else {
//                     return Expr::Intrinsic(Box::new(Intrinsic::Add(
//                         simplified,
//                     )));
//                 }
//             }
//             Intrinsic::Mul(mut operands) => {
//                 if operands.len() == 1 {
//                     return operands.remove(0);
//                 }

//                 let flattened = operands
//                     .into_iter()
//                     .flat_map(|expr| {
//                         expr.as_intrinsic()
//                             .and_then(|intr| intr.as_mul().cloned())
//                             .unwrap_or(vec![expr])
//                     })
//                     .collect::<Vec<_>>();

//                 let (mut simplified, folded_const) =
//                     fold_consts(flattened, Complex::ONE, |accum, x| accum * x);

//                 if folded_const == Complex::ZERO {
//                     return Expr::Const(Complex::ZERO);
//                 }

//                 // Distributive Property
//                 let add_ops = simplified
//                     .extract_if(.., |expr| {
//                         expr.as_intrinsic().is_some_and(|intr| intr.is_add())
//                     })
//                     .filter_map(|expr| {
//                         expr.as_intrinsic()
//                             .and_then(|intr| intr.as_add().cloned())
//                     })
//                     .collect::<Vec<_>>();

//                 let mut remaining_factors = simplified;
//                 if folded_const != Complex::ONE {
//                     remaining_factors.push(Expr::Const(folded_const));
//                 }

//                 let expanded = if add_ops.is_empty() {
//                     Intrinsic::Mul(remaining_factors)
//                 } else {
//                     Intrinsic::Add(
//                         add_ops
//                             .into_iter()
//                             .multi_cartesian_product()
//                             .map(|mut exprs| {
//                                 exprs.extend(remaining_factors.iter().cloned());
//                                 Expr::Intrinsic(Box::new(Intrinsic::Mul(exprs)))
//                             })
//                             .collect::<Vec<_>>(),
//                     )
//                 };

//                 return Expr::Intrinsic(Box::new(expanded));
//             }
//             _ => todo!(),
//         }
//     }

//     pub fn format(&self, buf: &mut String, ctx: &SymbolicContext) {
//         match self {
//             Intrinsic::Add(ops) => {
//                 for (i, op) in ops.iter().enumerate() {
//                     op.format(buf, ctx);
//                     if i < ops.len() - 1 {
//                         buf.push('+');
//                     }
//                 }
//             }
//             Intrinsic::Mul(ops) => {
//                 for (i, op) in ops.iter().enumerate() {
//                     op.format(buf, ctx);
//                     if i < ops.len() - 1 {
//                         buf.push('*');
//                     }
//                 }
//             }
//             _ => todo!(),
//         }
//     }

//     /// Returns `true` if the intrinsic is [`Add`].
//     ///
//     /// [`Add`]: Intrinsic::Add
//     #[must_use]
//     pub fn is_add(&self) -> bool {
//         matches!(self, Self::Add(..))
//     }

//     pub fn as_add(&self) -> Option<&Vec<Expr>> {
//         if let Self::Add(v) = self { Some(v) } else { None }
//     }

//     pub fn as_add_mut(&mut self) -> Option<&mut Vec<Expr>> {
//         if let Self::Add(v) = self { Some(v) } else { None }
//     }

//     /// Returns `true` if the intrinsic is [`Mul`].
//     ///
//     /// [`Mul`]: Intrinsic::Mul
//     #[must_use]
//     pub fn is_mul(&self) -> bool {
//         matches!(self, Self::Mul(..))
//     }

//     pub fn as_mul(&self) -> Option<&Vec<Expr>> {
//         if let Self::Mul(v) = self { Some(v) } else { None }
//     }

//     /// Returns `true` if the intrinsic is [`Neg`].
//     ///
//     /// [`Neg`]: Intrinsic::Neg
//     #[must_use]
//     pub fn is_neg(&self) -> bool {
//         matches!(self, Self::Neg(..))
//     }

//     pub fn as_neg(&self) -> Option<&Expr> {
//         if let Self::Neg(v) = self { Some(v) } else { None }
//     }
// }

// fn fold_consts(
//     operands: Vec<Expr>,
//     init: Quantity,
//     fold: impl FnMut(Quantity, &Quantity) -> Quantity,
// ) -> (Vec<Expr>, Quantity) {
//     let mut simplified_operands =
//         operands.into_iter().map(Expr::simplify).collect::<Vec<_>>();

//     let folded_const =
//         simplified_operands
//             .iter()
//             .filter_map(|expr| {
//                 if let Expr::Const(val) = expr { Some(val) } else { None }
//             })
//             .fold(init, fold);

//     simplified_operands.retain(|expr| !matches!(expr, Expr::Const(_)));

//     (simplified_operands, folded_const)
// }
