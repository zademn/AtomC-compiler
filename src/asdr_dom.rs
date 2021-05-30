use crate::lexer::{Token, TokenType};
use crate::symbols::*;
use std::collections::HashMap;

pub struct SyntaxAnalyser {
    pub token_vec: Vec<Token>,
    pub current_token_idx: usize,
    pub consumed_token: Option<Token>,
    pub current_table_idx: usize, // current symbol table
    pub symbol_tables: Vec<Context>,
    pub current_symbol: Option<Symbol>,
    pub current_dot_struct: Option<Symbol>,
    pub is_function_context: bool,
}
impl Default for SyntaxAnalyser {
    fn default() -> Self {
        Self {
            token_vec: vec![],
            current_token_idx: 0,
            consumed_token: None,
            current_table_idx: 0,
            symbol_tables: vec![],
            current_symbol: None,
            current_dot_struct: None,
            is_function_context: false,
        }
    }
}
impl SyntaxAnalyser {
    /// New function. Takes a Vec<Token> and sets the current token as the first one.
    /// If the provided Vec<Token> is empty set the consumed token to None
    pub fn new(token_vec: Vec<Token>) -> Self {
        if token_vec.len() == 0 {
            Default::default()
        }
        return Self {
            token_vec: token_vec,
            ..Default::default()
        };
    }
    /// Start function. Use this function to analyse the syntax of the Vec<Token> provided in the constructor
    pub fn analyse_syntax(&mut self) -> bool {
        let x = self.rule_unit();
        return x;
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
        return false;
    }
    fn find_symbol_everywhere(&self, symbol_name: &str) -> Option<Symbol> {
        for context in self.symbol_tables.iter().rev() {
            match context.find_symbol(symbol_name) {
                Some(s) => return Some(s),
                None => continue,
            }
        }
        None
    }
    fn find_symbol_global(&self, symbol_name: &str) -> Option<Symbol> {
        if self.symbol_tables.len() > 0 {
            return self.symbol_tables[0].find_symbol(symbol_name);
        }
        None
    }

    fn add_var(&mut self, token: &Token, s_type: &mut SymbolType) {
        let token_name = token.token_type.get_id();
        match self.current_symbol {
            Some(Symbol {
                class: ClassType::ClsStruct,
                ..
            }) => {
                let mut symbol = Symbol {
                    name: token_name,
                    symbol_type: s_type.clone(),
                    class: ClassType::ClsVar,
                    storage: StorageType::MemStruct,
                    line: token.line,
                    depth: self.symbol_tables[self.current_table_idx].depth,
                    am: None,
                    table: self.current_table_idx,
                };
                if let Some(ref mut cs) = self.current_symbol {
                    cs.add_symbol(symbol);
                    // Update global table
                    self.symbol_tables[0].update_symbol(cs.clone());
                }

                //self.symbol_tables[self.current_table_idx].add_symbol(symbol);
            }
            Some(Symbol {
                class: ClassType::ClsFunc,
                ..
            }) => {
                let mut symbol = Symbol {
                    name: token_name,
                    symbol_type: s_type.clone(),
                    class: ClassType::ClsVar,
                    storage: StorageType::MemLocal,
                    line: token.line,
                    depth: self.symbol_tables[self.current_table_idx].depth,
                    am: None,
                    table: self.current_table_idx,
                };
                if self.is_function_context {
                    if let Some(ref mut cs) = self.current_symbol {
                        cs.add_symbol(symbol.clone());
                        self.symbol_tables[0].update_symbol(cs.clone());
                    }
                }
                self.symbol_tables[self.current_table_idx].add_symbol(symbol);
            }
            None => {
                let mut symbol = Symbol {
                    name: token_name,
                    symbol_type: s_type.clone(),
                    class: ClassType::ClsVar,
                    storage: StorageType::MemGlobal,
                    line: token.line,
                    depth: self.symbol_tables[self.current_table_idx].depth,
                    am: None,
                    table: self.current_table_idx,
                };
                self.symbol_tables[self.current_table_idx].add_symbol(symbol);
            }
            _ => {}
        }
    }
    /// Sets current token and idx to the given idx
    fn set_current_token(&mut self, idx: usize) {
        self.current_token_idx = idx;
    }

    /// unit: ( declStruct | declFunc | declVar )* END ;
    /// Checks structure, functions or variables
    fn rule_unit(&mut self) -> bool {
        self.symbol_tables.push(Context::default()); // create global context
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
        return false;
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
                // Save consumed token
                let token_temp = self.token_vec[self.current_token_idx - 1].clone();
                if self.consume(TokenType::Lacc.discriminant_value()) {
                    let token_name = token_temp.token_type.get_id();
                    let mut symbol = Symbol {
                        name: token_name,
                        symbol_type: SymbolType {
                            type_base: TypeName::TbStruct,
                            struct_symbol: None,
                            num_elements: -1,
                        },
                        class: ClassType::ClsStruct,
                        storage: StorageType::MemGlobal,
                        line: token_temp.line,
                        depth: self.symbol_tables[self.current_table_idx].depth,
                        am: Some(HashMap::new()),
                        table: 0,
                    };
                    // Add a new context
                    self.symbol_tables[self.current_table_idx].add_symbol(symbol.clone());
                    self.symbol_tables.push(Context::new(
                        StorageType::MemStruct,
                        self.current_table_idx + 1,
                    ));
                    self.current_table_idx += 1;
                    self.current_symbol = Some(symbol);

                    loop {
                        if self.rule_decl_var() {
                        } else {
                            //TODO  Should i reset self.current_token_idx here?
                            break;
                        }
                    }
                    if self.consume(TokenType::Racc.discriminant_value()) {
                        if self.consume(TokenType::Semicolon.discriminant_value()) {
                            // Exit struct, pop the context
                            self.symbol_tables.pop();
                            self.current_table_idx -= 1;
                            self.current_symbol = None;
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
        return false;
    }
    /// declVar:  typeBase ID arrayDecl? ( COMMA ID arrayDecl? )* SEMICOLON ;
    /// Examples:
    /// int x;
    /// int x, y[];
    fn rule_decl_var(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        let mut symbol_type = SymbolType::default();
        //let mut token_temp: Token;
        if self.rule_type_base(&mut symbol_type) {
            if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                let mut token_temp = self.token_vec[self.current_token_idx - 1].clone();
                let mut is_array = self.rule_array_decl(&mut symbol_type);
                if !is_array {
                    symbol_type.num_elements = -1;
                }
                self.add_var(&token_temp, &mut symbol_type);
                loop {
                    if self.consume(TokenType::Comma.discriminant_value()) {
                        is_array = true;
                        if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                            token_temp = self.token_vec[self.current_token_idx - 1].clone();
                            if !self.rule_array_decl(&mut symbol_type) {
                                symbol_type.num_elements = -1;
                            };
                        } else {
                            self.token_error("Expected variable identifier after comma `,` ");
                        }
                    } else {
                        break;
                    }
                    self.add_var(&token_temp, &mut symbol_type);
                }
                if self.consume(TokenType::Semicolon.discriminant_value()) {
                    return true;
                } else {
                    if is_array {
                        self.token_error("Expected semicolon `;` after the variable declaration");
                    } else {
                        self.token_error("Expected '=', ',', ';' or array declaration")
                    }
                }
            } else {
                self.token_error("Expected identifier");
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
    }
    /// typeBase: INT | DOUBLE | CHAR | STRUCT ID ;
    /// Type declaration
    fn rule_type_base(&mut self, symbol_type: &mut SymbolType) -> bool {
        //let start_token_idx = self.current_token_idx;
        if (self.consume(TokenType::Int.discriminant_value()) && {
            symbol_type.type_base = TypeName::TbInt;
            true
        }) || (self.consume(TokenType::Double.discriminant_value()) && {
            symbol_type.type_base = TypeName::TbDouble;
            true
        }) || (self.consume(TokenType::Char.discriminant_value()) && {
            symbol_type.type_base = TypeName::TbChar;
            true
        }) || (self.consume(TokenType::Struct.discriminant_value()) && {
            if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                let token_temp = self.token_vec[self.current_token_idx - 1].clone();
                let token_name = token_temp.token_type.get_id();
                // Search for struct in global context
                match self.find_symbol_global(&token_name) {
                    Some(s) => {
                        if s.class != ClassType::ClsStruct {
                            panic!("{} is not a struct", token_name);
                        } else {
                            symbol_type.type_base = TypeName::TbStruct;
                            symbol_type.struct_symbol = Some(Box::new(s));
                        }
                    }
                    None => self.token_error(&format!("{} is undefined", token_name)),
                }
                true
            } else {
                self.token_error("Missing / invalid struct identifier");
                false
            }
        }) {
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
        return false;
    }
    /// arrayDecl: LBRACKET expr? RBRACKET ;
    /// Examples:
    /// [23]
    fn rule_array_decl(&mut self, symbol_type: &mut SymbolType) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.consume(TokenType::Lbracket.discriminant_value()) {
            if !self.rule_expr() {
                symbol_type.num_elements = 0; // arrawy without size
            };
            if self.consume(TokenType::Rbracket.discriminant_value()) {
                return true;
            } else {
                self.token_error("Expected `]` at the end of array declaration");
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
    }
    /// typeName: typeBase arrayDecl? ;
    fn rule_type_name(&mut self, symbol_type: &mut SymbolType) -> bool {
        if self.rule_type_base(symbol_type) {
            if !self.rule_array_decl(symbol_type) {
                symbol_type.num_elements = -1;
            };
            return true;
        }
        return false;
    }

    fn decl_func_context(&mut self, token: &Token, symbol_type: &mut SymbolType) {
        let token_name = token.token_type.get_id();
        if self.current_table_idx != 0 {
            self.token_error("Functions must be declared on global level") // TODO is this necessary?
        }
        let mut symbol = Symbol {
            name: token_name,
            symbol_type: symbol_type.clone(),
            class: ClassType::ClsFunc,
            storage: StorageType::MemGlobal,
            line: token.line,
            depth: self.symbol_tables[self.current_table_idx].depth,
            am: Some(HashMap::new()), // Init func arguments
            table: 0,
        };
        // Add a new context
        self.symbol_tables[self.current_table_idx].add_symbol(symbol.clone());
        // Add function context
        self.symbol_tables.push(Context::new(
            StorageType::MemLocal,
            self.current_table_idx + 1,
        ));
        self.current_table_idx += 1;
        self.current_symbol = Some(symbol);
        self.is_function_context = true;
    }
    /// declFunc: ( typeBase MUL? | VOID ) ID
    ///                     LPAR ( funcArg ( COMMA funcArg )* )? RPAR
    ///                     stmCompound ;
    fn rule_decl_func(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        let mut symbol_type: SymbolType = SymbolType::default();

        let mut is_decl_func = false;
        let mut ok = false;
        let has_type = {
            if self.rule_type_base(&mut symbol_type) {
                is_decl_func = self.consume(TokenType::Mul.discriminant_value());
                if !is_decl_func {
                    symbol_type.num_elements = -1;
                } else {
                    symbol_type.num_elements = 0;
                }
                true
            } else {
                false
            }
        };
        if has_type
            || (self.consume(TokenType::Void.discriminant_value()) && {
                is_decl_func = true;
                symbol_type.type_base = TypeName::TbVoid;
                true
            })
        {
            if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                let token_temp = self.token_vec[self.current_token_idx - 1].clone();
                let token_name = token_temp.token_type.get_id();
                if self.consume(TokenType::Lpar.discriminant_value()) {
                    self.decl_func_context(&token_temp, &mut symbol_type);
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
                            // Pop function argument context
                            self.symbol_tables.pop();
                            self.current_table_idx -= 1;
                            self.current_symbol = None;
                            return true;
                        } else {
                            self.token_error("Expected statement after function declaration")
                        }
                    } else {
                        self.token_error("Expected `)` at the end of function declaration");
                    }
                }
            }
            // else {
            //     self.token_error("Expected function identifier");
            // }
        }
        self.current_token_idx = start_token_idx;
        return false;
    }

    fn add_func_arg(&mut self, token: &Token, symbol_type: &mut SymbolType) {
        let token_name = token.token_type.get_id();
        let mut symbol = Symbol {
            name: token_name,
            symbol_type: symbol_type.clone(),
            class: ClassType::ClsVar,
            storage: StorageType::MemArg,
            line: token.line,
            depth: self.symbol_tables[self.current_table_idx].depth,
            am: None, // Init func arguments
            table: 0,
        };
        // Add a new context
        self.symbol_tables[self.current_table_idx].add_symbol(symbol.clone());
        self.current_symbol
            .as_mut()
            .unwrap()
            .am
            .as_mut()
            .unwrap()
            .insert(String::from(&symbol.name), symbol.clone());
    }
    /// funcArg: typeBase ID arrayDecl? ;
    fn rule_func_arg(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        let mut symbol_type = SymbolType::default();
        if self.rule_type_base(&mut symbol_type) {
            if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
                let token_temp = self.token_vec[self.current_token_idx - 1].clone();
                if !self.rule_array_decl(&mut symbol_type) {
                    symbol_type.num_elements = -1;
                };
                self.add_func_arg(&token_temp, &mut symbol_type);
                return true;
            } else {
                self.token_error("Expected function argument identifier");
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
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
        return false;
    }
    /// stmCompound: LACC ( declVar | stm )* RACC ;
    fn rule_stm_compound(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        let mut is_function_context_after = false;
        if self.consume(TokenType::Lacc.discriminant_value()) {
            if !self.is_function_context {
                self.current_table_idx += 1;
                self.symbol_tables
                    .push(Context::new(StorageType::MemLocal, self.current_table_idx));
            } else {
                self.is_function_context = false;
                is_function_context_after = true;
            }
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
                if !is_function_context_after {
                    self.symbol_tables.pop();
                    self.current_table_idx -= 1;
                }
                return true;
            } else {
                self.token_error("Expected } at the end of the statement")
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
    }
    /// expr: exprAssign ;
    fn rule_expr(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_assign() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        return false;
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
        return false;
    }
    /// exprOr: exprOr OR exprAnd | exprAnd ;
    fn rule_expr_or(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_and() {
            if self.rule_expr_or1() {
                return true;
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
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
        return true;
    }

    /// exprAnd: exprAnd AND exprEq | exprEq ;
    fn rule_expr_and(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_eq() {
            if self.rule_expr_and1() {
                return true;
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
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
        return true;
    }
    /// exprEq: exprEq ( EQUAL | NOTEQ ) exprRel | exprRel ;
    fn rule_expr_eq(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_rel() {
            if self.rule_expr_eq1() {
                return true;
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
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
        return true;
    }

    /// exprRel: exprRel ( LESS | LESSEQ | GREATER | GREATEREQ ) exprAdd | exprAdd ;
    fn rule_expr_rel(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_add() {
            if self.rule_expr_rel1() {
                return true;
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
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
        return true;
    }
    /// exprAdd: exprAdd ( ADD | SUB ) exprMul | exprMul ;
    fn rule_expr_add(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_mul() {
            if self.rule_expr_add1() {
                return true;
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
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
        return true;
    }
    /// exprMul: exprMul ( MUL | DIV ) exprCast | exprCast ;
    fn rule_expr_mul(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_cast() {
            if self.rule_expr_mul1() {
                return true;
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
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
        return true;
    }
    /// exprCast: LPAR typeName RPAR exprCast | exprUnary ;
    /// Examples:
    /// (int)x;
    /// (int)(double)x;
    fn rule_expr_cast(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        let mut symbol_type = SymbolType::default();
        if self.consume(TokenType::Lpar.discriminant_value()) {
            if self.rule_type_name(&mut symbol_type) {
                if self.consume(TokenType::Rpar.discriminant_value()) {
                    if self.rule_expr_cast() {
                        return true;
                    } else {
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
        return false;
    }

    /// exprUnary: ( SUB | NOT ) exprUnary | exprPostfix ;
    /// Check if and expression starts with `-` or `!`
    fn rule_expr_unary(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.consume(TokenType::Sub.discriminant_value())
            || self.consume(TokenType::Not.discriminant_value())
        {
            if self.rule_expr_unary() {
                return true;
            }
        }

        //self.current_token_idx = start_token_idx;
        if self.rule_expr_postfix() {
            return true;
        }
        self.current_token_idx = start_token_idx;
        return false;
    }

    /// exprPostfix: exprPostfix LBRACKET expr RBRACKET
    /// | exprPostfix DOT ID
    /// | exprPrimary ;
    fn rule_expr_postfix(&mut self) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_primary() {
            if self.rule_expr_postfix1() {
                return true;
            }
        }
        self.current_token_idx = start_token_idx;
        return false;
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
                // let token_temp = self.token_vec[self.current_token_idx - 1].clone();
                // let token_name = token_temp.token_type.get_id();
                // dbg!(&self.current_dot_struct);
                // match &self.current_dot_struct {
                //     Some(s) => match s.find_symbol(&token_name) {
                //         Some(f) => {}
                //         None => self.token_error(&format!(
                //             "Undefined field `{}` in struct `{}`",
                //             token_name, s.name
                //         )),
                //     },
                //     None => {
                //         self.token_error(&format!("Undefined symbol: `{}`", token_name));
                //     }
                // }
                if self.rule_expr_postfix1() {
                    return true;
                }
            } else {
                self.token_error("Expected identifier after `.`");
            }
        }
        //self.current_token_idx = start_token_idx;
        return true;
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
            // let token_temp = self.token_vec[self.current_token_idx - 1].clone();
            // let token_name = token_temp.token_type.get_id();
            // match self.find_symbol_everywhere(&token_name) {
            //     Some(
            //         s
            //         @
            //         Symbol {
            //             class: ClassType::ClsStruct,
            //             ..
            //         },
            //     ) => {
            //         //dbg!(&self.current_dot_struct);
            //         self.current_dot_struct = Some(s)
            //     }
            //     Some(s) => {
            //         //dbg!(&s);
            //     }
            //     None => {
            //         self.token_error(&format!("Undefined symbol: `{}`", token_name));
            //     }
            // }
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
        if self.consume(TokenType::Lpar.discriminant_value()) {
            if self.rule_expr() {
                if self.consume(TokenType::Rpar.discriminant_value()) {
                    return true;
                } else {
                    self.token_error("Expected closing `)` after expression");
                }
            }
            // clashes with cast expression
            // else {
            //     self.token_error("Expected expression in primary rule");
            // }
        }
        self.current_token_idx = start_token_idx;
        return false;
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
