use super::{
    constants::Constant, errors::Error, operators::*, tokens::ParenType, tokens::Token, utils,
};

#[derive(Clone, Debug, PartialEq)]
enum TokenType {
    Number,
    Operator,
    Paren,
    Constant,
}

#[allow(clippy::clippy::iter_nth_zero)]
fn _type(s: &str) -> Result<TokenType, ()> {
    Ok(if Token::is_next_number(s) {
        TokenType::Number
    } else if Operator::is(s) {
        TokenType::Operator
    } else if Token::is_next_paren(s) {
        TokenType::Paren
    } else if Constant::is(s) {
        TokenType::Constant
    } else {
        return Err(());
    })
}

#[allow(clippy::unnecessary_unwrap)]
pub fn tokenize(string: &str) -> Result<Vec<Token>, Error> {
    let mut vec: Vec<Token> = Vec::new();
    let mut explicit_paren = 0;
    let mut idx = 0;
    let mut coeff = false;
    let mut unary = true;
    while idx < string.chars().count() {
        // Current character
        let c = string.chars().nth(idx).unwrap();

        // Ignore whitespace and commas
        if c.is_whitespace() || c == ',' {
            idx += 1;
            coeff = coeff && c != ',';
            continue;
        }

        // Slice the input from the index until the end
        let slice = utils::slice(string, idx, -0);

        if coeff {
            // No coefficient if the current character is an r-paren
            let is_r_paren = Token::paren_type(c) == Some(ParenType::Right);
            if !is_r_paren {
                let opt = Operator::by_repr(&slice);
                let is_left_assoc_or_pow = opt.map_or(false, |(op, _)| {
                    op.associativity == Associativity::Left || op.kind == OperatorType::Pow
                });

                // Only a coefficient if the next (current) token is not
                // A left-associative function or pow
                if !is_left_assoc_or_pow {
                    vec.push(Token::Operator {
                        kind: OperatorType::Mul,
                    });
                }
            }
            coeff = false;
        }

        let kind = match _type(&slice) {
            Ok(k) => k,
            Err(..) => {
                return Err(Error::Parsing(idx));
            }
        };

        match kind {
            TokenType::Operator => {
                let unar = Operator::unary(&slice);

                if unary && unar.is_some() {
                    // Current token is a unary operator
                    let (a, b) = unar.unwrap();
                    idx += b;
                    vec.push(Token::Operator { kind: *a });
                    unary = false;
                } else {
                    unary = true;

                    let (operator, n) = Operator::by_repr(&slice).unwrap();

                    idx += n;
                    vec.push(Token::Operator {
                        kind: operator.kind,
                    });
                }
            }
            TokenType::Paren => {
                let (t, kind) = Token::paren(c).unwrap();
                match kind {
                    ParenType::Left => {
                        // Covers cases like `sin(-x)`
                        unary = true;
                        explicit_paren += 1;
                    }
                    ParenType::Right => {
                        // Covers cases like `sin(x) y => sin(x) * y`
                        coeff = true;
                        explicit_paren -= 1;
                    }
                }

                vec.push(t);
                idx += 1;
            }
            TokenType::Number => {
                let (t, n) = match Token::number(&slice) {
                    Some(a) => a,
                    None => return Err(Error::Parsing(idx)),
                };
                idx += n;
                vec.push(t);
                coeff = true;
                unary = false;
            }
            TokenType::Constant => {
                let (constant, n) = Constant::by_repr(&slice).unwrap();
                idx += n;
                vec.push(Token::Constant {
                    kind: constant.kind,
                });
                coeff = true;
                unary = false;
            }
        }
    }
    if explicit_paren == 0 {
        Ok(vec)
    } else {
        Err(Error::MismatchingParens)
    }
}
