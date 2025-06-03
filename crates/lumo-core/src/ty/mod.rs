#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct SimpleTypeRef(pub usize);

impl SimpleTypeRef {
    pub const UNIT: SimpleTypeRef = SimpleTypeRef(0);
}

#[derive(Debug, Clone)]
pub enum SimpleType {
    Variable(VariableState),
    Primitive(String),
    Function(Vec<SimpleTypeRef>, SimpleTypeRef),

    Todo,
}

impl SimpleType {
    pub fn variable() -> SimpleType {
        SimpleType::Variable(VariableState::default())
    }
}

#[derive(Clone, Debug, Default)]
pub struct VariableState {
    pub lower_bounds: Vec<SimpleTypeRef>,
    pub upper_bounds: Vec<SimpleTypeRef>,
}
