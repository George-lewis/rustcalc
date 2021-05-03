// #![allow(
//     clippy::float_cmp,
//     clippy::non_ascii_literal,
//     clippy::clippy::too_many_lines
// )]

// // use crate::{
// //     lib::doeval,
// //     lib::model::constants::*,
// //     lib::model::errors::Error,
// //     lib::model::operators::*,
// //     lib::model::{tokens::*, variables::Variable},
// //     stringify,
// // };

// fn same(a: f64, b: f64) -> bool {
//     (a - b).abs() < 0.000_001
// }

// #[test]
// fn test() {
//     [
//         (
//             "1 + 1",
//             "1 + 1",
//             2.0,
//             vec![
//                 Token::Number { value: 1.0 },
//                 Token::Operator {
//                     kind: OperatorType::Add,
//                 },
//                 Token::Number { value: 1.0 },
//             ],
//         ),
//         (
//             "sin pi",
//             "sin(π)",
//             std::f64::consts::PI.sin(),
//             vec![
//                 Token::Operator {
//                     kind: OperatorType::Sin,
//                 },
//                 Token::Constant {
//                     kind: ConstantType::PI,
//                 },
//             ],
//         ),
//         (
//             "1 plus 7 sub 2 times 3",
//             "1 + 7 - 2 × 3",
//             2.0,
//             vec![
//                 Token::Number { value: 1.0 },
//                 Token::Operator {
//                     kind: OperatorType::Add,
//                 },
//                 Token::Number { value: 7.0 },
//                 Token::Operator {
//                     kind: OperatorType::Sub,
//                 },
//                 Token::Number { value: 2.0 },
//                 Token::Operator {
//                     kind: OperatorType::Mul,
//                 },
//                 Token::Number { value: 3.0 },
//             ],
//         ),
//         (
//             "sin(1 + 2 + 3)",
//             "sin(1 + 2 + 3)",
//             ((1.0 + 2.0 + 3.0) as f64).sin(),
//             vec![
//                 Token::Operator {
//                     kind: OperatorType::Sin,
//                 },
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Number { value: 1.0 },
//                 Token::Operator {
//                     kind: OperatorType::Add,
//                 },
//                 Token::Number { value: 2.0 },
//                 Token::Operator {
//                     kind: OperatorType::Add,
//                 },
//                 Token::Number { value: 3.0 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//             ],
//         ),
//         (
//             "345.67",
//             "345.67",
//             345.67,
//             vec![Token::Number { value: 345.67 }],
//         ),
//         (
//             "sin (66) pow 2 plus cos(66)^2",
//             "sin(66)^2 + cos(66)^2",
//             1.0,
//             vec![
//                 Token::Operator {
//                     kind: OperatorType::Sin,
//                 },
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Number { value: 66.0 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//                 Token::Operator {
//                     kind: OperatorType::Pow,
//                 },
//                 Token::Number { value: 2.0 },
//                 Token::Operator {
//                     kind: OperatorType::Add,
//                 },
//                 Token::Operator {
//                     kind: OperatorType::Cos,
//                 },
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Number { value: 66.0 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//                 Token::Operator {
//                     kind: OperatorType::Pow,
//                 },
//                 Token::Number { value: 2.0 },
//             ],
//         ),
//         (
//             "(1)",
//             "(1)",
//             1.0,
//             vec![
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Number { value: 1.0 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//             ],
//         ),
//         (
//             "((1))",
//             "((1))",
//             1.0,
//             vec![
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Number { value: 1.0 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//             ],
//         ),
//         (
//             "-1",
//             "-1",
//             -1.0,
//             vec![
//                 Token::Operator {
//                     kind: OperatorType::Negative,
//                 },
//                 Token::Number { value: 1.0 },
//             ],
//         ),
//         (
//             "1 + -1",
//             "1 + -1",
//             0.0,
//             vec![
//                 Token::Number { value: 1.0 },
//                 Token::Operator {
//                     kind: OperatorType::Add,
//                 },
//                 Token::Operator {
//                     kind: OperatorType::Negative,
//                 },
//                 Token::Number { value: 1.0 },
//             ],
//         ),
//         (
//             "-   (  1.1 +  2.2)",
//             "-(1.1 + 2.2)",
//             -3.3,
//             vec![
//                 Token::Operator {
//                     kind: OperatorType::Negative,
//                 },
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Number { value: 1.1 },
//                 Token::Operator {
//                     kind: OperatorType::Add,
//                 },
//                 Token::Number { value: 2.2 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//             ],
//         ),
//     ]
//     .iter()
//     .for_each(|(a, b, c, d)| {
//         let (result, tokens) = match doeval(a, &[]) {
//             Ok((x, y)) => (x, y),
//             Err(e) => panic!("error! {:?}; {}", e, a),
//         };
//         assert_eq!(tokens, *d, "Checking tokenization of [{}]", a);
//         assert!(same(result, *c), "Checking evaluation of [{}]", a);
//         assert_eq!(stringify(&tokens, |a, _| a.to_string()), *b);
//     });
// }

// #[test]
// fn fail() {
//     vec![
//         ("1 +", Error::Operand(OperatorType::Add)),
//         ("1 + 2 + 3 + h", Error::Parsing(12)),
//         ("h", Error::Parsing(0)),
//         ("(1", Error::MismatchingParens),
//         ("3 + $a", Error::UnknownVariable(4)),
//     ]
//     .iter()
//     .for_each(|(a, b)| assert_eq!(doeval(a, &[]).unwrap_err(), *b));
// }

// #[test]
// fn test_vars() {
//     let test_vars = vec![
//         Variable {
//             repr: String::from('v'),
//             value: 5.0,
//         },
//         Variable {
//             repr: String::from("pi"),
//             value: 7.0,
//         },
//     ];

//     [
//         (
//             "$v",
//             "$v",
//             5.0,
//             vec![Token::Variable {
//                 inner: &test_vars[0],
//             }],
//         ),
//         (
//             "$v + 5",
//             "$v + 5",
//             10.0,
//             vec![
//                 Token::Variable {
//                     inner: &test_vars[0],
//                 },
//                 Token::Operator {
//                     kind: OperatorType::Add,
//                 },
//                 Token::Number { value: 5.0 },
//             ],
//         ),
//         (
//             "  5 +    $v    ",
//             "5 + $v",
//             10.0,
//             vec![
//                 Token::Number { value: 5.0 },
//                 Token::Operator {
//                     kind: OperatorType::Add,
//                 },
//                 Token::Variable {
//                     inner: &test_vars[0],
//                 },
//             ],
//         ),
//         (
//             "pi + $pi",
//             "π + $pi",
//             std::f64::consts::PI + 7.0,
//             vec![
//                 Token::Constant {
//                     kind: ConstantType::PI,
//                 },
//                 Token::Operator {
//                     kind: OperatorType::Add,
//                 },
//                 Token::Variable {
//                     inner: &test_vars[1],
//                 },
//             ],
//         ),
//     ]
//     .iter()
//     .for_each(|(a, b, c, d)| {
//         let (result, tokens) = match doeval(a, &test_vars) {
//             Ok((x, y)) => (x, y),
//             Err(e) => panic!("error! {:?}; {}", e, a),
//         };
//         assert_eq!(tokens, *d, "Checking tokenization of [{}]", a);
//         assert!(same(result, *c), "Checking evaluation of [{}]", a);
//         assert_eq!(stringify(&tokens, |a, _| a.to_string()), *b);
//     });
// }

// #[test]
// fn fail_vars() {
//     vec![("3 + $a", Error::UnknownVariable(4))]
//         .iter()
//         .for_each(|(a, b)| assert_eq!(doeval(a, &[]).unwrap_err(), *b));
// }
