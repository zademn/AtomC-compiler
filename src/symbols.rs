use std::collections::HashMap;
use std::rc::{Rc, Weak};
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
    pub am: Option<Vec<Symbol>>,
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
        let mut symbol = symbol;
        match self.find_symbol(&symbol.name) {
            Some(s) => panic!("Symbol {} is already defined on line {}", s.name, s.line),
            None => {
                //symbol.table = Some(Rc::new(self.clone())); // Make this a pointer?
                self.symbols.insert(String::from(&symbol.name), symbol);
            }
        };
    }
    /// Searches for symbol in this table.
    ///  If not found searches in the parent table
    pub fn find_symbol(&self, symbol_name: &str) -> Option<Symbol> {
        match self.symbols.get(symbol_name) {
            Some(s) => return Some(s.clone()),
            None => return None,
        }
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
        let mut s1 = Symbol {
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
        let mut s2 = Symbol {
            name: String::from("y"),
            ..s1.clone()
        };
        let mut st = Context::default();
        let mut all_tables = vec![&st];
        st.add_symbol(s1);
        st.add_symbol(s2);
        dbg!(&st);
        assert_eq!((&st.symbols).len(), 2);
    }
    #[test]
    #[should_panic]
    fn symbol_add_twice() {
        let mut s1 = Symbol {
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
        let mut s2 = Symbol {
            name: String::from("x"),
            ..s1.clone()
        };
        let mut st = Context::default();
        st.add_symbol(s1);
        st.add_symbol(s2);
        dbg!(&st);
    }
}
