use string_interner::StringInterner;

pub type Symbol = string_interner::symbol::SymbolU32;
pub type Interner = StringInterner<Symbol>;
