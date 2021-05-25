pub enum TypeName {
    TbInt,
    TbDouble,
    TbChar,
    TbStruct,
    TbFunc,
    TbVoid,
}

pub enum ClassType {
    ClsVar,
    ClsFunc,
    ClsExtFunc,
    ClsStruct,
}
pub enum StorageType {
    MemGlobal,
    MemLocal,
    MemArg,
    MemStruct,
    MemDeclaration,
    MemBuiltin,
}

struct SymbolType {
    type_base: TypeName,
    num_elements: usize,
}

struct Symbol {
    name: String,
    class: ClassType,
    storage: StorageType,
    depth: usize, // 0-global, 1-in function, 2... - nested blocks in function
    line: usize,
    offset: usize,
    table: SymbolTable,
}

struct SymbolTable {}
