use crate::lexer::Token;
use std::collections::HashMap;
/// Data types enum.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeName {
    TbInt,
    TbDouble,
    TbChar,
    TbStruct,
    TbFunc,
    TbVoid,
}

/// ClassType tells us if it's a variable, function struct, parameter etc
#[derive(Clone, Debug, PartialEq)]
pub enum ClassType {
    ClsVar,
    ClsFunc,
    ClsExtFunc,
    ClsStruct,
}

/// Tells us where the symbol is stored
#[derive(Clone, Debug, PartialEq)]
pub enum StorageType {
    MemGlobal,
    MemLocal,
    MemArg,
    MemStruct,
    MemDeclaration,
    MemBuiltin,
}

#[derive(Clone, Debug)]
pub struct SymbolType {
    pub type_base: TypeName,                // Tb*
    pub struct_symbol: Option<Box<Symbol>>, // for TbStruct
    pub num_elements: isize, //  >0 for an array of given size, 0 for an array without size, <0 if it's not an array
}
impl Default for SymbolType {
    fn default() -> Self {
        Self {
            type_base: TypeName::TbVoid,
            struct_symbol: None,
            num_elements: -1,
        }
    }
}
/// For types
impl SymbolType {
    pub fn new(type_base: TypeName, num_elements: isize) -> Self {
        Self {
            type_base: type_base,
            struct_symbol: None,
            num_elements: num_elements,
        }
    }
    pub fn cast(&self, dst: SymbolType, token: Token) {
        if self.num_elements > -1 {
            if dst.num_elements > -1 {
                if self.type_base != dst.type_base {
                    panic!(format!("Error at line {}: An array cannot be converted to an array of another type", token.line));
                } else {
                    panic!(format!(
                        "Error at line {}: An array cannot be converted to a non-array",
                        token.line
                    ));
                }
            }
        } else {
            if dst.num_elements > -1 {
                panic!(format!(
                    "Error at line {}: A non-array cannot be converted to an array",
                    &token.line
                ));
            }
        }
        match self.type_base {
            TypeName::TbChar | TypeName::TbInt | TypeName::TbDouble => match dst.type_base {
                TypeName::TbChar | TypeName::TbInt | TypeName::TbDouble => return,
                _ => {}
            },
            TypeName::TbStruct => {
                if dst.type_base == TypeName::TbStruct {
                    // TODO check for None
                    if self.struct_symbol.as_ref().unwrap().name != dst.struct_symbol.unwrap().name
                    {
                        panic!(
                            "Error at line {}: A structure cannot be converted to another one",
                            token.line
                        );
                    }
                }
            }
            _ => {
                panic!(format!("Error at line {}: Incompatible types", &token.line));
            }
        }
    }
    pub fn get_arith_type(self, t: SymbolType) -> Option<SymbolType> {
        match t.type_base {
            TypeName::TbChar => match t.type_base {
                TypeName::TbChar | TypeName::TbDouble | TypeName::TbInt => return Some(t),
                _ => return None,
            },
            TypeName::TbInt => match t.type_base {
                TypeName::TbChar => return Some(self),
                TypeName::TbDouble | TypeName::TbInt => return Some(t),
                _ => return None,
            },
            TypeName::TbDouble => match t.type_base {
                TypeName::TbChar | TypeName::TbDouble | TypeName::TbInt => return Some(self),
                _ => return None,
            },
            _ => return None,
        }
    }
}
pub fn add_ext_func(name: &str, symbol_type: SymbolType, context: &mut Context) -> Symbol {
    let s = Symbol {
        name: String::from(name),
        symbol_type: symbol_type.clone(),
        am: Some(HashMap::new()),
        class: ClassType::ClsExtFunc,
        storage: StorageType::MemBuiltin,
        depth: 0,
        line: 0,
        table: 0,
    };
    context.add_symbol(s.clone());
    return s;
}
pub fn add_func_arg(func: &mut Symbol, name: &str, symbol_type: SymbolType) -> Symbol {
    let s = Symbol {
        name: String::from(name),
        symbol_type: symbol_type.clone(),
        am: None,
        class: ClassType::ClsVar,
        storage: StorageType::MemLocal, // MemArg?
        depth: 0,
        line: 0,
        table: 0,
    };
    func.add_symbol(s.clone());
    return s;
}

pub fn add_ext_funcs(context: &mut Context) {
    let mut s: Symbol = add_ext_func("put_s", SymbolType::new(TypeName::TbVoid, -1), context);
    add_func_arg(&mut s, "s", SymbolType::new(TypeName::TbChar, 0));

    let mut s: Symbol = add_ext_func("get_s", SymbolType::new(TypeName::TbVoid, -1), context);
    add_func_arg(&mut s, "s", SymbolType::new(TypeName::TbChar, 0));

    let mut s: Symbol = add_ext_func("put_i", SymbolType::new(TypeName::TbVoid, -1), context);
    add_func_arg(&mut s, "i", SymbolType::new(TypeName::TbInt, -1));

    let mut s: Symbol = add_ext_func("get_i", SymbolType::new(TypeName::TbInt, -1), context);

    let mut s: Symbol = add_ext_func("put_d", SymbolType::new(TypeName::TbVoid, -1), context);
    add_func_arg(&mut s, "s", SymbolType::new(TypeName::TbDouble, -1));

    let mut s: Symbol = add_ext_func("get_d", SymbolType::new(TypeName::TbDouble, -1), context);

    let mut s: Symbol = add_ext_func("put_c", SymbolType::new(TypeName::TbVoid, -1), context);
    add_func_arg(&mut s, "c", SymbolType::new(TypeName::TbChar, -1));

    let mut s: Symbol = add_ext_func("get_c", SymbolType::new(TypeName::TbChar, -1), context);

    let mut s: Symbol = add_ext_func("seconds", SymbolType::new(TypeName::TbDouble, -1), context);
}
#[derive(Clone, Debug)]
pub enum ArgsMembers {
    Args(Vec<Symbol>),    // used for functions
    Members(Vec<Symbol>), // used for structs
}
pub enum AddrOffset {
    Addr(()),      // vm: memory for global symbols
    Offset(isize), // vm: stack offset for local symbols
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Symbol {
    pub name: String,
    pub class: ClassType,
    pub storage: StorageType,
    pub symbol_type: SymbolType, // Tells us the data type and bonus info if it's a struct
    pub depth: usize,            // 0-global, 1-in function, 2... - nested blocks in function
    pub line: usize,
    pub am: Option<HashMap<String, Symbol>>,
    pub table: usize, // Index in the big table of the parent table
                      //ao: AddrOffset,
}
impl Default for Symbol {
    fn default() -> Self {
        Self {
            name: String::from(""),
            class: ClassType::ClsVar,
            storage: StorageType::MemGlobal,
            symbol_type: SymbolType::default(),
            depth: 0,
            line: 0,
            am: None,
            table: 0,
        }
    }
}
impl Symbol {
    pub fn find_symbol(&self, symbol_name: &str) -> Option<Symbol> {
        match &self.am {
            Some(am) => am.get(symbol_name).cloned(),
            None => None,
        }
    }
    pub fn add_symbol(&mut self, symbol: Symbol) {
        let symbol = symbol;
        match self.find_symbol(&symbol.name) {
            Some(s) => panic!(
                "Error at line {}: Symbol `{}` is already defined on line {}",
                symbol.line, s.name, s.line
            ),
            None => {
                //symbol.table = Some(Rc::new(self.clone())); // Make this a pointer?
                if let Some(am) = &mut self.am {
                    am.insert(String::from(&symbol.name), symbol);
                }
            }
        }
    }
    pub fn update_symbol(&mut self, symbol: Symbol) {
        if let Some(am) = &mut self.am {
            am.insert(String::from(&symbol.name), symbol);
        }
    }
}
#[derive(Debug)]
pub struct Context {
    pub symbols: HashMap<String, Symbol>,
    pub storage: StorageType,
    pub depth: usize,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            symbols: HashMap::new(),
            storage: StorageType::MemGlobal,
            depth: 0,
        }
    }
}
impl Context {
    pub fn new(s: StorageType, d: usize) -> Self {
        Self {
            symbols: HashMap::new(),
            storage: s,
            depth: d,
        }
    }
    pub fn add_symbol(&mut self, symbol: Symbol) {
        let symbol = symbol;
        match self.find_symbol(&symbol.name) {
            Some(s) => panic!(
                "Error at line {}: Symbol `{}` is already defined on line {}",
                symbol.line, s.name, s.line
            ),
            None => {
                //symbol.table = Some(Rc::new(self.clone())); // Make this a pointer?
                self.symbols.insert(String::from(&symbol.name), symbol);
            }
        };
    }
    /// Searches for symbol in this table.
    ///  If not found searches in the parent table
    pub fn find_symbol(&self, symbol_name: &str) -> Option<Symbol> {
        self.symbols.get(symbol_name).cloned()
    }
    pub fn update_symbol(&mut self, symbol: Symbol) {
        self.symbols.insert(String::from(&symbol.name), symbol);
    }
    pub fn clear(&mut self) {
        self.symbols.clear();
    }
}

#[cfg(test)]
pub mod tests {
    use crate::symbols::*;
    #[test]
    fn symbol_add_two() {
        let s1 = Symbol {
            name: String::from("x"),
            class: ClassType::ClsVar,
            storage: StorageType::MemArg,
            depth: 0,
            line: 0,
            symbol_type: SymbolType {
                type_base: TypeName::TbChar,
                struct_symbol: None,
                num_elements: -1,
            },
            am: None,
            table: 0,
        };
        let s2 = Symbol {
            name: String::from("y"),
            ..s1.clone()
        };
        let mut st = Context::default();
        let _all_tables = vec![&st];
        st.add_symbol(s1);
        st.add_symbol(s2);
        dbg!(&st);
        assert_eq!((&st.symbols).len(), 2);
    }
    #[test]
    #[should_panic]
    fn symbol_add_twice() {
        let s1 = Symbol {
            name: String::from("x"),
            class: ClassType::ClsVar,
            storage: StorageType::MemArg,
            depth: 0,
            line: 0,
            symbol_type: SymbolType {
                type_base: TypeName::TbChar,
                struct_symbol: None,
                num_elements: -1,
            },
            am: None,
            table: 0,
        };
        let s2 = Symbol {
            name: String::from("x"),
            ..s1.clone()
        };
        let mut st = Context::default();
        st.add_symbol(s1);
        st.add_symbol(s2);
        dbg!(&st);
    }
}
