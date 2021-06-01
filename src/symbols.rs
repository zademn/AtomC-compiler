use crate::lexer::Token;
use indexmap::map::IndexMap;

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
#[derive(Clone, Debug, PartialEq)]
pub enum CtVal {
    IntChar(isize),
    Double(f32),
    String(String),
}
impl CtVal {
    pub fn get_int(&self) -> isize {
        if let CtVal::IntChar(i) = self {
            return *i;
        }
        0
    }
    pub fn get_double(&self) -> f32 {
        if let CtVal::Double(d) = self {
            return *d;
        }
        0.
    }
    pub fn get_string(&self) -> String {
        if let CtVal::String(s) = self {
            return String::from(s);
        }
        String::from("")
    }
}
#[derive(Clone, Debug)]
pub struct RetVal {
    pub symbol_type: Option<SymbolType>,
    pub is_lval: bool,
    pub is_ctval: bool,
    pub ctval: Option<CtVal>,
}
impl Default for RetVal {
    fn default() -> Self {
        Self {
            symbol_type: None,
            is_lval: false,
            is_ctval: false,
            ctval: None,
        }
    }
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
            type_base,
            struct_symbol: None,
            num_elements,
        }
    }
    pub fn cast(&self, dst: SymbolType, token: &Token) {
        if self.num_elements > -1 {
            if dst.num_elements > -1 {
                if self.type_base != dst.type_base {
                    panic!("Error at line {}: An array cannot be converted to an array of another type", token.line);
                }
            } else {
                panic!(
                    "Error at line {}: An array cannot be converted to a non-array",
                    token.line
                );
            }
        } else if dst.num_elements > -1 {
            panic!(
                "Error at line {}: A non-array cannot be converted to an array",
                token.line
            );
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
                    return;
                }
            }
            _ => {}
        }
        panic!("Error at line {}: Incompatible types", &token.line);
    }
    pub fn get_arith_type(self, t: SymbolType) -> Option<SymbolType> {
        match t.type_base {
            TypeName::TbChar => match t.type_base {
                TypeName::TbChar | TypeName::TbDouble | TypeName::TbInt => Some(t),
                _ => None,
            },
            TypeName::TbInt => match t.type_base {
                TypeName::TbChar => Some(self),
                TypeName::TbDouble | TypeName::TbInt => Some(t),
                _ => None,
            },
            TypeName::TbDouble => match t.type_base {
                TypeName::TbChar | TypeName::TbDouble | TypeName::TbInt => Some(self),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ArgsMembers {
    Args(Vec<Symbol>),    // used for functions
    Members(Vec<Symbol>), // used for structs
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub enum AddrOffset {
    Addr(*const ()),
    Offset(isize),
}
impl AddrOffset {
    pub fn get_addr(&self) -> *const () {
        if let AddrOffset::Addr(addr) = self {
            return *addr;
        }
        std::ptr::null()
    }
    pub fn get_offset(&self) -> isize {
        if let AddrOffset::Offset(o) = self {
            return *o;
        }
        0
    }
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
    pub am: Option<IndexMap<String, Symbol>>,
    pub table: usize, // Index in the big table of the parent table
    //ao: AddrOffset,
    pub ao: AddrOffset,
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
            ao: AddrOffset::Offset(0),
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
    pub symbols: IndexMap<String, Symbol>,
    pub storage: StorageType,
    pub depth: usize,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            symbols: IndexMap::new(),
            storage: StorageType::MemGlobal,
            depth: 0,
        }
    }
}
impl Context {
    pub fn new(s: StorageType, d: usize) -> Self {
        Self {
            symbols: IndexMap::new(),
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

pub fn add_ext_func(
    name: &str,
    symbol_type: SymbolType,
    context: &mut Context,
    addr: *const (),
) -> Symbol {
    let s = Symbol {
        name: String::from(name),
        symbol_type,
        am: Some(IndexMap::new()),
        class: ClassType::ClsExtFunc,
        storage: StorageType::MemBuiltin,
        ao: AddrOffset::Addr(addr),

        ..Default::default()
    };
    context.add_symbol(s.clone());
    s
}
pub fn add_func_arg(func: &mut Symbol, name: &str, symbol_type: SymbolType) -> Symbol {
    let s = Symbol {
        name: String::from(name),
        symbol_type,
        class: ClassType::ClsVar,
        storage: StorageType::MemLocal, // MemArg?
        ..Default::default()
    };
    func.add_symbol(s.clone());
    s
}

pub fn add_ext_funcs(context: &mut Context) {
    let mut s: Symbol = add_ext_func(
        "put_s",
        SymbolType::new(TypeName::TbVoid, -1),
        context,
        put_s as *const (),
    );
    add_func_arg(&mut s, "s", SymbolType::new(TypeName::TbChar, 0));
    context.update_symbol(s);

    let mut s: Symbol = add_ext_func(
        "get_s",
        SymbolType::new(TypeName::TbVoid, -1),
        context,
        get_s as *const (),
    );
    add_func_arg(&mut s, "s", SymbolType::new(TypeName::TbChar, 0));
    context.update_symbol(s);

    let mut s: Symbol = add_ext_func(
        "put_i",
        SymbolType::new(TypeName::TbVoid, -1),
        context,
        put_i as *const (),
    );
    add_func_arg(&mut s, "i", SymbolType::new(TypeName::TbInt, -1));
    context.update_symbol(s);

    let _s: Symbol = add_ext_func(
        "get_i",
        SymbolType::new(TypeName::TbInt, -1),
        context,
        get_i as *const (),
    );
    context.update_symbol(_s);

    let mut s: Symbol = add_ext_func(
        "put_d",
        SymbolType::new(TypeName::TbVoid, -1),
        context,
        put_d as *const (),
    );
    add_func_arg(&mut s, "s", SymbolType::new(TypeName::TbDouble, -1));
    context.update_symbol(s);

    let _s: Symbol = add_ext_func(
        "get_d",
        SymbolType::new(TypeName::TbDouble, -1),
        context,
        get_d as *const (),
    );
    context.update_symbol(_s);

    let mut s: Symbol = add_ext_func(
        "put_c",
        SymbolType::new(TypeName::TbVoid, -1),
        context,
        put_c as *const (),
    );
    add_func_arg(&mut s, "c", SymbolType::new(TypeName::TbChar, -1));
    context.update_symbol(s);

    let _s: Symbol = add_ext_func(
        "get_c",
        SymbolType::new(TypeName::TbChar, -1),
        context,
        get_s as *const (),
    );
    context.update_symbol(_s);

    let _s: Symbol = add_ext_func(
        "seconds",
        SymbolType::new(TypeName::TbDouble, -1),
        context,
        seconds as *const (),
    );
    context.update_symbol(_s);
}
fn put_s() {}
fn get_s(s: &str) {}
fn put_i() {
    println!("hello from ext func");
}
fn get_i() -> i64 {
    return 0;
}
fn put_d() {}
fn get_d() -> f64 {
    return 0.;
}
fn put_c() {}
fn get_c() -> char {
    return 0 as char;
}
fn seconds() -> f64 {
    return 0.;
}
pub fn require_symbol(contexts: &Vec<Context>, name: &str) -> Symbol {
    for context in contexts.iter().rev() {
        match context.find_symbol(name) {
            Some(s) => return s,
            None => continue,
        }
    }
    panic!("Undefined symbol: {}", name);
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
            symbol_type: SymbolType {
                type_base: TypeName::TbChar,
                struct_symbol: None,
                num_elements: -1,
            },
            ..Default::default()
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
            symbol_type: SymbolType {
                type_base: TypeName::TbChar,
                struct_symbol: None,
                num_elements: -1,
            },
            ..Default::default()
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
