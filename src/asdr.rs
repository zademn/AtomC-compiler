#[allow(dead_code)]
use crate::lexer::{Token, TokenType};

pub struct SyntaxAnalyser {
    pub token_vec: Vec<Token>,
    pub current_token_idx: usize,
    pub consumed_token: Option<Token>,
}
impl Default for SyntaxAnalyser {
    fn default() -> Self {
        Self {
            token_vec: vec![],
            current_token_idx: 0,
            consumed_token: None,
            
        }
    }
}
impl SyntaxAnalyser {
    /// New function. Takes a Vec<Token> and sets the current token as the first one.
    /// If the provided Vec<Token> is empty set the consumed token to None
    pub fn new(token_vec: Vec<Token>) -> Self {
        if token_vec.is_empty() {
            Default::default()
        }
        Self {
            token_vec,
            current_token_idx: 0,
            consumed_token: None,
        }
    }
    /// Start function. Use this function to analyse the syntax of the Vec<Token> provided in the constructor
    pub fn analyse_syntax(&mut self) -> bool {
        let x = self.rule_unit();
        //dbg!(&x);
        x
    }

    /// Error function. Takes a message. Prints the line of the current_token and the message provided
    fn token_error(&self, msg: &str) {
        // eprintln!(
        //     "Error in line:{}, {}",
        //     self.token_vec[self.current_token_idx].line, msg
        // );
        panic!(
            "Error in line:{}, {}",
            self.token_vec[self.current_token_idx].line, msg
        );
    }

    /// Consumes the current token if it matches the code provided and moves forward
    fn consume(&mut self, code: u8) -> bool {
        let current_code = self.token_vec[self.current_token_idx]
            .token_type
            .discriminant_value();
        if current_code == code {
            self.consumed_token = Some(self.token_vec[self.current_token_idx].clone());
            self.current_token_idx += 1;
            return true;
        }
        false
    }

    /// Sets current token and idx to the given idx
    fn set_current_token(&mut self, idx: usize) {
        self.current_token_idx = idx;
    }

    /// unit: ( declStruct | declFunc | declVar )* END ;
    /// Checks structure, functions or variables
    fn rule_unit(&mut self) -> bool {
        loop {
            let temp_idx = self.current_token_idx;
            if {
                self.current_token_idx = temp_idx;
                self.rule_decl_struct()
            } || {
                self.current_token_idx = temp_idx;
                self.rule_decl_func()
            } || {
                self.current_token_idx = temp_idx;
                self.rule_decl_var()
            } {
            } else {
                break;
            }
        }
        if self.consume(TokenType::End.discriminant_value()) {
            return true;
        } else {
            self.token_error("Top level error: Expected function / struct / variable definition");
        }
        false
    }

    /// declStruct: STRUCT ID LACC declVar* RACC SEMICOLON ;
    /// Example:
    /// struct Something {
    /// int x;
    /// };
    fn rule_decl_struct(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.consume(TokenType::Struct.discriminant_value()) {
            if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                if self.consume(TokenType::Lacc.discriminant_value()) {
                    loop {
                        if self.rule_decl_var() {
                        } else {
                            //TODO  Should i reset self.current_token_idx here?
                            break;
                        }
                    }
                    if self.consume(TokenType::Racc.discriminant_value()) {
                        if self.consume(TokenType::Semicolon.discriminant_value()) {
                            return true;
                        } else {
                            self.token_error("Expected semicolon `;` after struct declaration");
                        }
                    } else {
                        self.token_error("Expected closing bracket `}` at the end of the struct");
                    }
                } // No error if no `{`
            } else {
                self.token_error("Expected struct identifier");
            }
        }
        self.current_token_idx = start_token_idx;
        false
    }
    /// declVar:  typeBase ID arrayDecl? ( COMMA ID arrayDecl? )* SEMICOLON ;
    /// Examples:
    /// int x;
    /// int x, y[];
    fn rule_decl_var(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_type_base() {
            if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                self.rule_array_decl();
                loop {
                    if self.consume(TokenType::Comma.discriminant_value()) {
                        if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                            self.rule_array_decl();
                        } else {
                            self.token_error("Expected variable identifier after comma `,` ");
                        }
                    } else {
                        break;
                    }
                }
                if self.consume(TokenType::Semicolon.discriminant_value()) {
                    return true;
                } else {
                    self.token_error("Expected semicolon `;` after the variable declaration");
                }
            } else {
                self.token_error("Expected identifier");
            }
        }
        self.current_token_idx = start_token_idx;
        false
    }
    /// typeBase: INT | DOUBLE | CHAR | STRUCT ID ;
    /// Type declaration
    fn rule_type_base(&mut self) -> bool {
        let _start_token_idx = self.current_token_idx;
        if self.consume(TokenType::Int.discriminant_value())
            || self.consume(TokenType::Double.discriminant_value())
            || self.consume(TokenType::Char.discriminant_value())
            || (self.consume(TokenType::Struct.discriminant_value()) && {
                if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                    true
                } else {
                    self.token_error("Missing / invalid struct identifier");
                    false
                }
            })
        {
            return true;
        }

        // if self.consume(TokenType::Struct.discriminant_value()) {
        //     if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
        //         return true;
        //     } else {
        //         self.token_error("Missing / invalid struct identifier");
        //     }
        //     // Reset token if sequence not satisfied
        //     self.set_current_token(start_token_idx);
        // }
        false
    }
    /// arrayDecl: LBRACKET expr? RBRACKET ;
    /// Examples:
    /// [23]
    fn rule_array_decl(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.consume(TokenType::Lbracket.discriminant_value()) {
            self.rule_expr();
            if self.consume(TokenType::Rbracket.discriminant_value()) {
                return true;
            } else {
                self.token_error("Expected `]` at the end of array declaration");
            }
        }
        self.current_token_idx = start_token_idx;
        false
    }
    /// typeName: typeBase arrayDecl? ;
    fn rule_type_name(&mut self) -> bool {
        if self.rule_type_base() {
            self.rule_array_decl();
            return true;
        }
        false
    }
    /// declFunc: ( typeBase MUL? | VOID ) ID
    ///                     LPAR ( funcArg ( COMMA funcArg )* )? RPAR
    ///                     stmCompound ;
    fn rule_decl_func(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        let has_type = {
            if self.rule_type_base() {
                self.consume(TokenType::Mul.discriminant_value());
                true
            } else {
                false
            }
        };
        if (has_type || self.consume(TokenType::Void.discriminant_value())) && self.consume(TokenType::Id("".to_string()).discriminant_value()) && self.consume(TokenType::Lpar.discriminant_value()) {
            self.rule_func_arg(); // funcarg is optional
            loop {
                if self.consume(TokenType::Comma.discriminant_value()) {
                    if self.rule_func_arg() {
                    } else {
                        self.token_error("Expected function argument after ,");
                    }
                } else {
                    break;
                }
            }
            if self.consume(TokenType::Rpar.discriminant_value()) {
                if self.rule_stm_compound() {
                    return true;
                } else {
                    self.token_error("Expected statement after function declaration")
                }
            } else {
                self.token_error("Expected `)` at the end of function declaration");
            }
        }
        self.current_token_idx = start_token_idx;
        false
    }

    /// funcArg: typeBase ID arrayDecl? ;
    fn rule_func_arg(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_type_base() {
            if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                self.rule_array_decl();
                return true;
            } else {
                self.token_error("Expected function argument identifier");
            }
        }
        self.current_token_idx = start_token_idx;
        false
    }

    /// stm: stmCompound
    ///        | IF LPAR expr RPAR stm ( ELSE stm )?
    ///        | WHILE LPAR expr RPAR stm
    ///       | FOR LPAR expr? SEMICOLON expr? SEMICOLON expr? RPAR stm
    ///        | BREAK SEMICOLON
    ///        | RETURN expr? SEMICOLON
    ///        | expr? SEMICOLON ;
    ///
    fn rule_stm(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_stm_compound() {
            return true;
        }

        // If condition
        if self.consume(TokenType::If.discriminant_value()) {
            if self.consume(TokenType::Lpar.discriminant_value()) {
                if self.rule_expr() {
                    if self.consume(TokenType::Rpar.discriminant_value()) {
                        if self.rule_stm() {
                            // Optional else
                            if self.consume(TokenType::Else.discriminant_value()) {
                                if self.rule_stm() {
                                } else {
                                    self.token_error("Expected `else` statement");
                                }
                            }
                            return true;
                        } else {
                            self.token_error("Expected `if` statement");
                        }
                    } else {
                        self.token_error("Expected closing `)` after `if` condition")
                    }
                } else {
                    self.token_error("Expected `if` condition");
                }
            } else {
                self.token_error("Expected opening `(` before the `if` condition")
            }
        }

        // While
        if self.consume(TokenType::While.discriminant_value()) {
            if self.consume(TokenType::Lpar.discriminant_value()) {
                if self.rule_expr() {
                    if self.consume(TokenType::Rpar.discriminant_value()) {
                        if self.rule_stm() {
                            return true;
                        } else {
                            self.token_error("Expected `while` statement");
                        }
                    } else {
                        self.token_error("Expected `)` after `while` condition")
                    }
                } else {
                    self.token_error("Expected `while` condition");
                }
            } else {
                self.token_error("Expected `(` before the `while` condition")
            }
        }
        // For
        if self.consume(TokenType::For.discriminant_value()) {
            if self.consume(TokenType::Lpar.discriminant_value()) {
                self.rule_expr(); // TODO should i reset if this fails?
                if self.consume(TokenType::Semicolon.discriminant_value()) {
                    self.rule_expr(); // TODO should i reset if this fails?
                    if self.consume(TokenType::Semicolon.discriminant_value()) {
                        self.rule_expr(); // TODO should i reset if this fails?
                        if self.consume(TokenType::Rpar.discriminant_value()) {
                            if self.rule_stm() {
                                return true;
                            } else {
                                self.token_error("Expected `for` statement")
                            }
                        } else {
                            self.token_error("Expected `)` at the end of the `for`")
                        }
                    } else {
                        self.token_error("Expected semicolon `;` after the second `for` expression")
                    }
                } else {
                    self.token_error("Expected semicolon `;` after the first `for` expression")
                }
            } else {
                self.token_error("Expected `(` at the start of the `for`")
            }
        }

        if self.consume(TokenType::Break.discriminant_value()) {
            if self.consume(TokenType::Semicolon.discriminant_value()) {
                return true;
            } else {
                self.token_error("Expected semicolon `;` at the end of the `break` statement")
            }
        }

        if self.consume(TokenType::Return.discriminant_value()) {
            self.rule_expr();
            if self.consume(TokenType::Semicolon.discriminant_value()) {
                return true;
            } else {
                self.token_error("Expected semicolon `;` at the end of the `return` statement")
            }
        }
        if self.rule_expr() {
            if self.consume(TokenType::Semicolon.discriminant_value()) {
                return true;
            } else {
                self.token_error("Expected semicolon `;` at the end of the expression")
            }
        }
        if self.consume(TokenType::Semicolon.discriminant_value()) {
            return true;
        };
        self.current_token_idx = start_token_idx;
        false
    }
    /// stmCompound: LACC ( declVar | stm )* RACC ;
    fn rule_stm_compound(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.consume(TokenType::Lacc.discriminant_value()) {
            loop {
                let temp_idx = self.current_token_idx;
                if {
                    self.current_token_idx = temp_idx;
                    self.rule_decl_var()
                } || {
                    self.current_token_idx = temp_idx;
                    self.rule_stm()
                } {
                } else {
                    break;
                }
            }
            if self.consume(TokenType::Racc.discriminant_value()) {
                return true;
            } else {
                self.token_error("Expected } at the end of the statement")
            }
        }
        self.current_token_idx = start_token_idx;
        false
    }
    /// expr: exprAssign ;
    fn rule_expr(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_assign() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    /// exprAssign: exprUnary ASSIGN exprAssign | exprOr ;
    fn rule_expr_assign(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_unary() {
            if self.consume(TokenType::Assign.discriminant_value()) {
                if self.rule_expr_assign() {
                    return true;
                } else {
                    self.token_error("Missing right operand after `=` in assign operation")
                }
            } // No need to expect assign operator
            self.current_token_idx = start_token_idx;
        }

        // Reset before or variable
        //self.current_token_idx = start_token_idx;
        if self.rule_expr_or() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }
    /// exprOr: exprOr OR exprAnd | exprAnd ;
    fn rule_expr_or(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_and() && self.rule_expr_or1() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    /// exprOr1: (OR exprAnd exprOr1)?
    fn rule_expr_or1(&mut self) -> bool {
        //let start_token_idx = self.current_token_idx;
        if self.consume(TokenType::Or.discriminant_value()) {
            if self.rule_expr_and() {
                if self.rule_expr_or1() {
                    return true;
                }
            } else {
                self.token_error("Expected operand in `or` expression body");
            }
        };
        //self.current_token_idx = start_token_idx;
        true
    }

    /// exprAnd: exprAnd AND exprEq | exprEq ;
    fn rule_expr_and(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_eq() && self.rule_expr_and1() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }
    /// exprAnd1:  (AND exprEq | exprAnd1)? ;
    fn rule_expr_and1(&mut self) -> bool {
        if self.consume(TokenType::And.discriminant_value()) {
            if self.rule_expr_eq() {
                if self.rule_expr_and1() {}
            } else {
                self.token_error("Expected operand in `and` expression body");
            }
        };
        true
    }
    /// exprEq: exprEq ( EQUAL | NOTEQ ) exprRel | exprRel ;
    fn rule_expr_eq(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_rel() && self.rule_expr_eq1() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }
    /// exprEq1: (( EQUAL | NOTEQ ) exprRel exprEq1)?' ;
    fn rule_expr_eq1(&mut self) -> bool {
        if self.consume(TokenType::Equal.discriminant_value())
            || self.consume(TokenType::NotEq.discriminant_value())
        {
            if self.rule_expr_rel() {
                if self.rule_expr_eq1() {
                    return true;
                }
            } else {
                self.token_error("Expected operand in `equals` expression body");
            }
        }
        true
    }

    /// exprRel: exprRel ( LESS | LESSEQ | GREATER | GREATEREQ ) exprAdd | exprAdd ;
    fn rule_expr_rel(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_add() && self.rule_expr_rel1() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    fn rule_expr_rel1(&mut self) -> bool {
        if self.consume(TokenType::Less.discriminant_value())
            || self.consume(TokenType::LessEq.discriminant_value())
            || self.consume(TokenType::Greater.discriminant_value())
            || self.consume(TokenType::GreaterEq.discriminant_value())
        {
            if self.rule_expr_add() {
                if self.rule_expr_rel1() {
                    return true;
                }
            } else {
                self.token_error("Expected operand in `relation` expression body");
            }
        }
        true
    }
    /// exprAdd: exprAdd ( ADD | SUB ) exprMul | exprMul ;
    fn rule_expr_add(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_mul() && self.rule_expr_add1() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }
    fn rule_expr_add1(&mut self) -> bool {
        if self.consume(TokenType::Add.discriminant_value())
            || self.consume(TokenType::Sub.discriminant_value())
        {
            if self.rule_expr_mul() {
                if self.rule_expr_add1() {
                    return true;
                }
            } else {
                self.token_error("Expected operand in `addition / subtraction` expression body");
            }
        }
        true
    }
    /// exprMul: exprMul ( MUL | DIV ) exprCast | exprCast ;
    fn rule_expr_mul(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_cast() && self.rule_expr_mul1() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    fn rule_expr_mul1(&mut self) -> bool {
        if self.consume(TokenType::Mul.discriminant_value())
            || self.consume(TokenType::Div.discriminant_value())
        {
            if self.rule_expr_cast() {
                if self.rule_expr_mul1() {
                    return true;
                }
            } else {
                self.token_error("Expected operand in `multiplication / division` expression body");
            }
        }
        true
    }
    /// exprCast: LPAR typeName RPAR exprCast | exprUnary ;
    /// Examples:
    /// (int)x;
    /// (int)(double)x;
    fn rule_expr_cast(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.consume(TokenType::Lpar.discriminant_value()) {
            if self.rule_type_name() {
                if self.consume(TokenType::Rpar.discriminant_value()) {
                    if self.rule_expr_cast() {
                        return true;
                    }
                    else {
                        self.token_error("Invalid `cast` expression")
                    }
                } else {
                    self.token_error("Expected closing `)` in `cast` expression")
                }
            } else {
                self.token_error("Expected `type` in `cast` expression")
            }
            self.current_token_idx = start_token_idx;
        }
        // TODO should i reset here?
        self.current_token_idx = start_token_idx;
        if self.rule_expr_unary() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    /// exprUnary: ( SUB | NOT ) exprUnary | exprPostfix ;
    /// Check if and expression starts with `-` or `!`
    fn rule_expr_unary(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if (self.consume(TokenType::Sub.discriminant_value())
            || self.consume(TokenType::Not.discriminant_value())) && self.rule_expr_unary() {
            return true;
        }

        //self.current_token_idx = start_token_idx;
        if self.rule_expr_postfix() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    /// exprPostfix: exprPostfix LBRACKET expr RBRACKET
    /// | exprPostfix DOT ID
    /// | exprPrimary ;
    fn rule_expr_postfix(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_primary() && self.rule_expr_postfix1() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }
    fn rule_expr_postfix1(&mut self) -> bool {
        if self.consume(TokenType::Lbracket.discriminant_value()) {
            if self.rule_expr() {
                if self.consume(TokenType::Rbracket.discriminant_value()) {
                    if self.rule_expr_postfix1() {
                        return true;
                    }
                } else {
                    self.token_error("Expected `]` in `postfix` rule");
                }
            } else {
                self.token_error("Expected `expression` after `[`")
            }
        }
        // TODO should i reset here?
        //self.current_token_idx = start_token_idx;
        if self.consume(TokenType::Dot.discriminant_value()) {
            if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                if self.rule_expr_postfix1() {
                    return true;
                }
            } else {
                self.token_error("Expected identifier after `.`");
            }
        }
        //self.current_token_idx = start_token_idx;
        true
    }

    /// exprPrimary: ID ( LPAR ( expr ( COMMA expr )* )? RPAR )?
    /// | CT_INT
    /// | CT_REAL
    /// | CT_CHAR
    /// | CT_STRING
    /// | LPAR expr RPAR ;
    fn rule_expr_primary(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
            // Optional
            if self.consume(TokenType::Lpar.discriminant_value()) {
                if self.rule_expr() {
                    loop {
                        if self.consume(TokenType::Comma.discriminant_value()) {
                            if self.rule_expr() {
                            } else {
                                self.token_error(
                                    "expected `expression` after `comma` in primary rule",
                                );
                            }
                        } else {
                            break;
                        }
                    }
                } // no else because it's optional
                if self.consume(TokenType::Rpar.discriminant_value()) {
                } else {
                    self.token_error("Expected closing `)` after expression body");
                }
            }
            return true;
        }

        //self.current_token_idx = start_token_idx;
        if self.consume(TokenType::CtInt(0).discriminant_value())
            || self.consume(TokenType::CtReal(0.).discriminant_value())
            || self.consume(TokenType::CtChar('a').discriminant_value())
            || self.consume(TokenType::CtString("".to_string()).discriminant_value())
        {
            return true;
        }
        if self.consume(TokenType::Lpar.discriminant_value()) && self.rule_expr() {
            if self.consume(TokenType::Rpar.discriminant_value()) {
                return true;
            } else {
                self.token_error("Expected closing `)` after expression");
            }
        }
        self.current_token_idx = start_token_idx;
        false
    }
}

#[cfg(test)]
pub mod tests {
    use crate::asdr::SyntaxAnalyser;
    use crate::lexer::Lexer;
    #[test]
    fn syntax_test() {
        //let mut lexer = Lexer::from_file("../tests/test_syntax.txt");
        let mut lexer = Lexer::from_file("../tests/test_syntax.c");
        let token_vec = lexer.get_tokens();
        for elem in &token_vec {
            println!("{:?}", elem);
        }
        let mut syntax_analyser: SyntaxAnalyser = SyntaxAnalyser::new(token_vec);
        syntax_analyser.analyse_syntax();
    }
}
