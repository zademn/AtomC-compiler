use crate::lexer::{Token, TokenType};
use crate::symbols::*;
use indexmap::map::IndexMap;

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
        if token_vec.is_empty() {
            Default::default()
        }
        Self {
            token_vec,
            ..Default::default()
        }
    }
    /// Start function. Use this function to analyse the syntax of the Vec<Token> provided in the constructor
    pub fn analyse_syntax(&mut self) -> bool {
        let x = self.rule_unit();
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
        if !self.symbol_tables.is_empty() {
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
                let symbol = Symbol {
                    name: token_name,
                    symbol_type: s_type.clone(),
                    class: ClassType::ClsVar,
                    storage: StorageType::MemStruct,
                    line: token.line,
                    depth: self.symbol_tables[self.current_table_idx].depth,
                    am: None,
                    table: self.current_table_idx,
                    ..Default::default()
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
                let symbol = Symbol {
                    name: token_name,
                    symbol_type: s_type.clone(),
                    class: ClassType::ClsVar,
                    storage: StorageType::MemLocal,
                    line: token.line,
                    depth: self.symbol_tables[self.current_table_idx].depth,
                    am: None,
                    table: self.current_table_idx,
                    ..Default::default()
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
                let symbol = Symbol {
                    name: token_name,
                    symbol_type: s_type.clone(),
                    class: ClassType::ClsVar,
                    storage: StorageType::MemGlobal,
                    line: token.line,
                    depth: self.symbol_tables[self.current_table_idx].depth,
                    am: None,
                    table: self.current_table_idx,
                    ..Default::default()
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
        add_ext_funcs(&mut self.symbol_tables[0]);
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
                // Save consumed token
                //let token_temp = self.token_vec[self.current_token_idx - 1].clone();
                let token_temp = self.consumed_token.as_ref().unwrap().clone();
                if self.consume(TokenType::Lacc.discriminant_value()) {
                    let token_name = token_temp.token_type.get_id();
                    let symbol = Symbol {
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
                        am: Some(IndexMap::new()),
                        table: 0,
                        ..Default::default()
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
        false
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
                } else if is_array {
                    self.token_error("Expected semicolon `;` after the variable declaration");
                } else {
                    self.token_error("Expected '=', ',', ';' or array declaration")
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
        false
    }
    /// arrayDecl: LBRACKET expr? RBRACKET ;
    /// Examples:
    /// [23]
    fn rule_array_decl(&mut self, symbol_type: &mut SymbolType) -> bool {
        let start_token_idx = self.current_token_idx;
        let mut rv = RetVal::default();
        if self.consume(TokenType::Lbracket.discriminant_value()) {
            if self.rule_expr(&mut rv) {
                if !rv.is_ctval {
                    self.token_error("the array size is not a constant value")
                }
                if rv.symbol_type.unwrap().type_base != TypeName::TbInt {
                    self.token_error("the array size is not an integer")
                }
                symbol_type.num_elements = rv.ctval.unwrap().get_int();
            } else {
                symbol_type.num_elements = 0; // arrawy without size
            };
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
    fn rule_type_name(&mut self, symbol_type: &mut SymbolType) -> bool {
        if self.rule_type_base(symbol_type) {
            if !self.rule_array_decl(symbol_type) {
                symbol_type.num_elements = -1;
            };
            return true;
        }
        false
    }

    fn decl_func_context(&mut self, token: &Token, symbol_type: &mut SymbolType) {
        let token_name = token.token_type.get_id();
        if self.current_table_idx != 0 {
            self.token_error("Functions must be declared on global level") // TODO is this necessary?
        }
        let symbol = Symbol {
            name: token_name,
            symbol_type: symbol_type.clone(),
            class: ClassType::ClsFunc,
            storage: StorageType::MemGlobal,
            line: token.line,
            depth: self.symbol_tables[self.current_table_idx].depth,
            am: Some(IndexMap::new()), // Init func arguments
            table: 0,
            ..Default::default()
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

        let mut is_decl_func: bool;
        let _ok = false;
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
        if (has_type
            || (self.consume(TokenType::Void.discriminant_value()) && {
                is_decl_func = true;
                symbol_type.type_base = TypeName::TbVoid;
                true
            }))
            && self.consume(TokenType::Id("".to_string()).discriminant_value())
        {
            let token_temp = self.token_vec[self.current_token_idx - 1].clone();
            let _token_name = token_temp.token_type.get_id();
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
        self.current_token_idx = start_token_idx;
        false
    }

    fn add_func_arg(&mut self, token: &Token, symbol_type: &mut SymbolType) {
        let token_name = token.token_type.get_id();
        let symbol = Symbol {
            name: token_name,
            symbol_type: symbol_type.clone(),
            class: ClassType::ClsVar,
            storage: StorageType::MemArg,
            line: token.line,
            depth: self.symbol_tables[self.current_table_idx].depth,
            am: None, // Init func arguments
            table: 0,
            ..Default::default()
        };
        // Add a new context
        self.symbol_tables[self.current_table_idx].add_symbol(symbol.clone());
        // self.current_symbol
        //     .as_mut()
        //     .unwrap()
        //     .am
        //     .as_mut()
        //     .unwrap()
        //     .insert(String::from(&symbol.name), symbol);
        if self.is_function_context {
            if let Some(ref mut cs) = self.current_symbol {
                cs.add_symbol(symbol);
                self.symbol_tables[0].update_symbol(cs.clone());
            }
        }
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
        let mut rv = RetVal::default();
        if self.rule_stm_compound() {
            return true;
        }

        // If condition
        if self.consume(TokenType::If.discriminant_value()) {
            if self.consume(TokenType::Lpar.discriminant_value()) {
                if self.rule_expr(&mut rv) {
                    if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct {
                        self.token_error("a structure cannot be logically tested");
                    }
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
                if self.rule_expr(&mut rv) {
                    if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct {
                        self.token_error("a structure cannot be logically tested");
                    }
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
        let mut rv1 = RetVal::default();
        let mut rv2 = RetVal::default();
        let mut rv3 = RetVal::default();
        if self.consume(TokenType::For.discriminant_value()) {
            if self.consume(TokenType::Lpar.discriminant_value()) {
                if self.rule_expr(&mut rv1) {
                    // instructions
                } // TODO should i reset if this fails?
                if self.consume(TokenType::Semicolon.discriminant_value()) {
                    if self.rule_expr(&mut rv2)
                        && rv2.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct
                    {
                        self.token_error("a structure cannot be logically tested");
                    }; // TODO should i reset if this fails?
                    if self.consume(TokenType::Semicolon.discriminant_value()) {
                        if self.rule_expr(&mut rv3) {
                            // instructions
                        }; // TODO should i reset if this fails?
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
            if self.rule_expr(&mut rv) {
                if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbVoid {
                    self.token_error("a void function cannot return a value");
                }
                self.current_symbol.as_ref().unwrap().symbol_type.cast(
                    rv.symbol_type.as_ref().unwrap().clone(),
                    &self.consumed_token.as_ref().unwrap(),
                );
            };
            if self.consume(TokenType::Semicolon.discriminant_value()) {
                return true;
            } else {
                self.token_error("Expected semicolon `;` at the end of the `return` statement")
            }
        }
        if self.rule_expr(&mut rv) {
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
        false
    }
    /// expr: exprAssign ;
    fn rule_expr(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_assign(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    /// exprAssign: exprUnary ASSIGN exprAssign | exprOr ;
    fn rule_expr_assign(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        let mut rve = RetVal::default();
        if self.rule_expr_unary(rv) {
            if self.consume(TokenType::Assign.discriminant_value()) {
                if self.rule_expr_assign(&mut rve) {
                    if !rv.is_lval {
                        self.token_error("cannot assign to a non-lval");
                    }
                    if rv.symbol_type.as_ref().unwrap().num_elements > -1
                        || rve.symbol_type.as_ref().unwrap().num_elements > -1
                    {
                        self.token_error("The arrays cannot be assigned");
                    }
                    rv.symbol_type.as_ref().unwrap().cast(
                        rve.symbol_type.unwrap(),
                        &self.token_vec[self.current_token_idx],
                    );
                    rv.is_ctval = false;
                    rv.is_lval = false;
                    return true;
                } else {
                    self.token_error("Missing right operand after `=` in assign operation")
                }
            } // No need to expect assign operator
            self.current_token_idx = start_token_idx;
        }

        // Reset before or variable
        //self.current_token_idx = start_token_idx;
        if self.rule_expr_or(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }
    /// exprOr: exprOr OR exprAnd | exprAnd ;
    fn rule_expr_or(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_and(rv) && self.rule_expr_or1(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    /// exprOr1: (OR exprAnd exprOr1)?
    fn rule_expr_or1(&mut self, rv: &mut RetVal) -> bool {
        //let start_token_idx = self.current_token_idx;
        let mut rve = RetVal::default();
        if self.consume(TokenType::Or.discriminant_value()) {
            if self.rule_expr_and(&mut rve) {
                if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct
                    || rve.symbol_type.unwrap().type_base == TypeName::TbStruct
                {
                    self.token_error("A structure cannot be loically tested");
                }
                rv.symbol_type = Some(SymbolType::new(TypeName::TbInt, -1));
                rv.is_ctval = false;
                rv.is_lval = false;
                if self.rule_expr_or1(rv) {
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
    fn rule_expr_and(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_eq(rv) && self.rule_expr_and1(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }
    /// exprAnd1:  (AND exprEq | exprAnd1)? ;
    fn rule_expr_and1(&mut self, rv: &mut RetVal) -> bool {
        let mut rve = RetVal::default();
        if self.consume(TokenType::And.discriminant_value()) {
            if self.rule_expr_eq(&mut rve) {
                if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct
                    || rve.symbol_type.unwrap().type_base == TypeName::TbStruct
                {
                    self.token_error("A structure cannot be loically tested");
                }
                rv.symbol_type = Some(SymbolType::new(TypeName::TbInt, -1));
                rv.is_ctval = false;
                rv.is_lval = false;
                if self.rule_expr_and1(rv) {}
            } else {
                self.token_error("Expected operand in `and` expression body");
            }
        };
        true
    }
    /// exprEq: exprEq ( EQUAL | NOTEQ ) exprRel | exprRel ;
    fn rule_expr_eq(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_rel(rv) && self.rule_expr_eq1(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }
    /// exprEq1: (( EQUAL | NOTEQ ) exprRel exprEq1)?' ;
    fn rule_expr_eq1(&mut self, rv: &mut RetVal) -> bool {
        let mut rve = RetVal::default();
        if self.consume(TokenType::Equal.discriminant_value())
            || self.consume(TokenType::NotEq.discriminant_value())
        {
            //let token_temp = self.token_vec[self.current_token_idx - 1].clone();
            if self.rule_expr_rel(&mut rve) {
                if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct
                    || rve.symbol_type.unwrap().type_base == TypeName::TbStruct
                {
                    self.token_error("A structure cannot be compared");
                }
                rv.symbol_type = Some(SymbolType::new(TypeName::TbInt, -1));
                rv.is_ctval = false;
                rv.is_lval = false;
                if self.rule_expr_eq1(rv) {
                    return true;
                }
            } else {
                self.token_error("Expected operand in `equals` expression body");
            }
        }
        true
    }

    /// exprRel: exprRel ( LESS | LESSEQ | GREATER | GREATEREQ ) exprAdd | exprAdd ;
    fn rule_expr_rel(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_add(rv) && self.rule_expr_rel1(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    fn rule_expr_rel1(&mut self, rv: &mut RetVal) -> bool {
        let mut rve = RetVal::default();
        if self.consume(TokenType::Less.discriminant_value())
            || self.consume(TokenType::LessEq.discriminant_value())
            || self.consume(TokenType::Greater.discriminant_value())
            || self.consume(TokenType::GreaterEq.discriminant_value())
        {
            if self.rule_expr_add(&mut rve) {
                if rv.symbol_type.as_ref().unwrap().num_elements > -1
                    || rve.symbol_type.as_ref().unwrap().num_elements > -1
                {
                    self.token_error("An array cannot be compared");
                }
                if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct
                    || rve.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct
                {
                    self.token_error("A structure cannot be compared");
                }
                rv.symbol_type = Some(SymbolType::new(TypeName::TbInt, -1));
                rv.is_ctval = false;
                rv.is_lval = false;
                if self.rule_expr_rel1(rv) {
                    return true;
                }
            } else {
                self.token_error("Expected operand in `relation` expression body");
            }
        }
        true
    }
    /// exprAdd: exprAdd ( ADD | SUB ) exprMul | exprMul ;
    fn rule_expr_add(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_mul(rv) && self.rule_expr_add1(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }
    fn rule_expr_add1(&mut self, rv: &mut RetVal) -> bool {
        let mut rve = RetVal::default();
        if self.consume(TokenType::Add.discriminant_value())
            || self.consume(TokenType::Sub.discriminant_value())
        {
            if self.rule_expr_mul(&mut rve) {
                if rv.symbol_type.as_ref().unwrap().num_elements > -1
                    || rve.symbol_type.as_ref().unwrap().num_elements > -1
                {
                    self.token_error("An array cannot be added / subtracted");
                }
                if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct
                    || rve.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct
                {
                    self.token_error("A structure cannot be added / subtracted");
                }
                rv.symbol_type = rv
                    .symbol_type
                    .as_ref()
                    .unwrap()
                    .clone()
                    .get_arith_type(rve.symbol_type.unwrap());
                rv.is_ctval = false;
                rv.is_lval = false;
                if self.rule_expr_add1(rv) {
                    return true;
                }
            } else {
                self.token_error("Expected operand in `addition / subtraction` expression body");
            }
        }
        true
    }
    /// exprMul: exprMul ( MUL | DIV ) exprCast | exprCast ;
    fn rule_expr_mul(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_cast(rv) && self.rule_expr_mul1(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    fn rule_expr_mul1(&mut self, rv: &mut RetVal) -> bool {
        let mut rve = RetVal::default();
        if self.consume(TokenType::Mul.discriminant_value())
            || self.consume(TokenType::Div.discriminant_value())
        {
            if self.rule_expr_cast(&mut rve) {
                if rv.symbol_type.as_ref().unwrap().num_elements > -1
                    || rve.symbol_type.as_ref().unwrap().num_elements > -1
                {
                    self.token_error("An array cannot be multiplied / divided");
                }
                if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct
                    || rve.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct
                {
                    self.token_error("A structure cannot be multiplied / divided");
                }
                rv.symbol_type = rv
                    .symbol_type
                    .as_ref()
                    .unwrap()
                    .clone()
                    .get_arith_type(rve.symbol_type.unwrap());
                rv.is_ctval = false;
                rv.is_lval = false;
                if self.rule_expr_mul1(rv) {
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
    fn rule_expr_cast(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        let mut symbol_type = SymbolType::default();
        let mut rve = RetVal::default();
        if self.consume(TokenType::Lpar.discriminant_value()) {
            if self.rule_type_name(&mut symbol_type) {
                if self.consume(TokenType::Rpar.discriminant_value()) {
                    if self.rule_expr_cast(&mut rve) {
                        dbg!(&symbol_type);
                        dbg!(&rve.symbol_type);
                        symbol_type.cast(
                            rve.symbol_type.unwrap(),
                            &self.token_vec[self.current_token_idx - 1],
                        );
                        rv.symbol_type = Some(symbol_type);
                        rv.is_ctval = false;
                        rv.is_lval = false;
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
        if self.rule_expr_unary(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    /// exprUnary: ( SUB | NOT ) exprUnary | exprPostfix ;
    /// Check if and expression starts with `-` or `!`
    fn rule_expr_unary(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.consume(TokenType::Sub.discriminant_value())
            || self.consume(TokenType::Not.discriminant_value())
        {
            let token_temp = self.token_vec[self.current_token_idx - 1].clone();
            if self.rule_expr_unary(rv) {
                match token_temp.token_type {
                    TokenType::Sub => {
                        if rv.symbol_type.as_ref().unwrap().num_elements > -1 {
                            self.token_error("unary `-` cannot be applied to arrays")
                        }
                        if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct {
                            self.token_error("unary `-` cannot be applied to structures")
                        }
                    }
                    TokenType::Not => {
                        if rv.symbol_type.as_ref().unwrap().type_base == TypeName::TbStruct {
                            self.token_error("unary `!` cannot be applied to structures")
                        }
                    }
                    _ => {}
                }
                rv.is_ctval = false;
                rv.is_lval = false;
                return true;
            } else {
                self.token_error("Invalid unary expression");
            }
        }
        //self.current_token_idx = start_token_idx;
        if self.rule_expr_postfix(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }

    /// exprPostfix: exprPostfix LBRACKET expr RBRACKET
    /// | exprPostfix DOT ID
    /// | exprPrimary ;
    fn rule_expr_postfix(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        if self.rule_expr_primary(rv) && self.rule_expr_postfix1(rv) {
            return true;
        }
        self.current_token_idx = start_token_idx;
        false
    }
    fn rule_expr_postfix1(&mut self, rv: &mut RetVal) -> bool {
        let mut rve = RetVal::default();
        if self.consume(TokenType::Lbracket.discriminant_value()) {
            if self.rule_expr(&mut rve) {
                if rv.symbol_type.as_ref().unwrap().num_elements < 0 {
                    self.token_error("Only an array can be indexed");
                }
                let type_int = SymbolType::new(TypeName::TbInt, -1);
                type_int.cast(
                    rve.symbol_type.unwrap(),
                    &self.token_vec[self.current_token_idx],
                );
                // rv.symbol_type = Some(SymbolType::new(
                //     rv.symbol_type.as_ref().unwrap().type_base.clone(),
                //     -1,
                // ));
                //rv.symbol_type = rv.symbol_type.clone(); // shoulb be rve here?

                rv.symbol_type = Some(SymbolType {
                    num_elements: -1,
                    ..rv.symbol_type.as_ref().unwrap().clone()
                });
                rv.is_lval = true;
                rv.is_ctval = false;

                if self.consume(TokenType::Rbracket.discriminant_value()) {
                    if self.rule_expr_postfix1(rv) {
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
                let token_temp = self.token_vec[self.current_token_idx - 1].clone();
                let token_name = token_temp.token_type.get_id();
                let s_struct = rv.symbol_type.as_ref().unwrap();
                if s_struct.struct_symbol.is_none() {
                    self.token_error(&format!("`{}`'s parent is not a struct", token_name));
                }
                let s_struct = s_struct.struct_symbol.as_ref().unwrap();
                //let s_struct: Symbol = (*rv.symbol_type.as_ref().unwrap().struct_symbol).unwrap();
                let s_member = s_struct.find_symbol(&token_name);
                match s_member {
                    Some(s) => {
                        rv.symbol_type = Some(s.symbol_type);
                        rv.is_lval = true;
                        rv.is_ctval = false;
                    }
                    None => {
                        self.token_error(&format!(
                            "struct {} does not have the member {}",
                            s_struct.name, token_name
                        ));
                    }
                }

                if self.rule_expr_postfix1(rv) {
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
    fn rule_expr_primary(&mut self, rv: &mut RetVal) -> bool {
        let start_token_idx = self.current_token_idx;
        let mut is_func = false;
        if self.consume(TokenType::Id("".to_string()).discriminant_value()) {
            let token_temp = self.token_vec[self.current_token_idx - 1].clone();
            let token_name = token_temp.token_type.get_id();
            let mut arg = RetVal::default();
            let ss = self.find_symbol_everywhere(&token_name);
            match &ss {
                Some(s) => {
                    rv.symbol_type = Some(s.symbol_type.clone());
                    rv.is_ctval = false;
                    rv.is_lval = true;
                    if s.class == ClassType::ClsFunc || s.class == ClassType::ClsExtFunc {
                        rv.is_lval = false;
                        is_func = true;
                    }
                }
                None => self.token_error(&format!("undefined symbol: `{}`", token_name)),
            }
            let s = ss.unwrap();
            // Optional
            if self.consume(TokenType::Lpar.discriminant_value()) {
                if s.class != ClassType::ClsFunc && s.class != ClassType::ClsExtFunc {
                    self.token_error(&format!("`{}` is not a function", token_name))
                }
                let mut num_args = 0;
                let defined_args =
                    s.am.as_ref()
                        .unwrap()
                        .values()
                        .cloned()
                        .collect::<Vec<Symbol>>();
                if self.rule_expr(&mut arg) {
                    // this passes if we have 1 arg => we use `>=`
                    if num_args >= defined_args.len() {
                        self.token_error(&format!(
                            "Too many arguments in function `{}` call",
                            token_name
                        ));
                    }
                    defined_args[num_args].symbol_type.cast(
                        arg.symbol_type.as_ref().unwrap().clone(),
                        &self.token_vec[self.current_token_idx],
                    );
                    num_args += 1;
                }
                loop {
                    if self.consume(TokenType::Comma.discriminant_value()) {
                        if self.rule_expr(&mut arg) {
                            if num_args >= defined_args.len() {
                                self.token_error(&format!(
                                    "Too many arguments in function `{}` call",
                                    token_name
                                ));
                            }
                            defined_args[num_args].symbol_type.cast(
                                arg.symbol_type.as_ref().unwrap().clone(),
                                &self.token_vec[self.current_token_idx],
                            );
                            num_args += 1;
                        } else {
                            self.token_error("expected `expression` after `comma` in primary rule");
                        }
                    } else {
                        break;
                    }
                }
                // no else because it's optional
                if self.consume(TokenType::Rpar.discriminant_value()) {
                    if num_args < defined_args.len() {
                        self.token_error(&format!(
                            "Too few arguments in function `{}` call",
                            token_name
                        ));
                    }
                    rv.symbol_type = Some(s.symbol_type);
                    rv.is_ctval = false;
                    rv.is_lval = false;
                } else {
                    dbg!(&self.token_vec[self.current_token_idx]);
                    dbg!(&s);
                    if s.class == ClassType::ClsFunc || s.class == ClassType::ClsExtFunc {
                        self.token_error(&format!("Missing call for function `{}`", s.name));
                    }
                    self.token_error("Expected closing `)` after expression body");
                }
            }
            return true;
        }

        //self.current_token_idx = start_token_idx;
        if self.consume(TokenType::CtInt(0).discriminant_value()) {
            let i = self.token_vec[self.current_token_idx - 1]
                .token_type
                .get_int();
            rv.symbol_type = Some(SymbolType::new(TypeName::TbInt, -1));
            rv.ctval = Some(CtVal::IntChar(i));
            rv.is_ctval = true;
            rv.is_lval = false;
            return true;
        }
        if self.consume(TokenType::CtChar('a').discriminant_value()) {
            let i = self.token_vec[self.current_token_idx - 1]
                .token_type
                .get_char();
            rv.symbol_type = Some(SymbolType::new(TypeName::TbChar, -1));
            rv.ctval = Some(CtVal::IntChar(i as isize));
            rv.is_ctval = true;
            rv.is_lval = false;
            return true;
        }
        if self.consume(TokenType::CtReal(0.).discriminant_value()) {
            let i = self.token_vec[self.current_token_idx - 1]
                .token_type
                .get_double();
            rv.symbol_type = Some(SymbolType::new(TypeName::TbDouble, -1));
            rv.ctval = Some(CtVal::Double(i));
            rv.is_ctval = true;
            rv.is_lval = false;
            return true;
        }
        if self.consume(TokenType::CtString("".to_string()).discriminant_value()) {
            let i = self
                .consumed_token
                .as_ref()
                .unwrap()
                .token_type
                .get_string();
            rv.symbol_type = Some(SymbolType::new(TypeName::TbChar, 0));
            rv.ctval = Some(CtVal::String(i));
            rv.is_ctval = true;
            rv.is_lval = false;
            return true;
        }
        if self.consume(TokenType::Lpar.discriminant_value()) && self.rule_expr(rv) {
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
