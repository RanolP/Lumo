#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub body: Vec<Stmt>,
}

impl Program {
    pub fn new(body: Vec<Stmt>) -> Self {
        Self { body }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self { stmts }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<TsType>,
}

impl Param {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_ann: None,
        }
    }

    pub fn with_type(mut self, ty: TsType) -> Self {
        self.type_ann = Some(ty);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub export: bool,
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<TsType>,
    pub body: FunctionBody,
    /// `#[inline(always)]` — definition will be substituted at call sites and
    /// the declaration removed from the final output.
    pub inline_always: bool,
}

impl FunctionDecl {
    pub fn new(name: impl Into<String>, body: FunctionBody) -> Self {
        Self {
            export: false,
            name: name.into(),
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: None,
            body,
            inline_always: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    Expr(Box<Expr>),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDecl {
    pub export: bool,
    pub name: String,
    pub type_ann: Option<TsType>,
    pub init: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub export: bool,
    pub name: String,
    pub type_params: Vec<String>,
    pub ty: TsType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDecl {
    pub export: bool,
    pub name: String,
    pub members: Vec<InterfaceMember>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceMember {
    pub name: String,
    pub ty: TsType,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Return(Option<Expr>),
    Const(ConstDecl),
    /// `let name = init;` or `let name;` (for IIFE flattening with reassignment)
    Let {
        name: String,
        export: bool,
        type_ann: Option<TsType>,
        init: Option<Expr>,
    },
    /// `name = expr;` (assignment expression as statement)
    Assign { name: String, value: Expr },
    If {
        cond: Expr,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    Block(Block),
    Function(FunctionDecl),
    TypeAlias(TypeAlias),
    Interface(InterfaceDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Undefined,
    Void(Box<Expr>),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Member {
        object: Box<Expr>,
        property: String,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Array(Vec<Expr>),
    Object(Vec<ObjectProp>),
    Arrow {
        params: Vec<Param>,
        return_type: Option<TsType>,
        body: Box<FunctionBody>,
    },
    IfElse {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProp {
    pub key: ObjectKey,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectKey {
    Ident(String),
    String(String),
    Computed(Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
    BitNot,
}

impl UnaryOp {
    pub fn as_str(self) -> &'static str {
        match self {
            UnaryOp::Plus => "+",
            UnaryOp::Minus => "-",
            UnaryOp::Not => "!",
            UnaryOp::BitNot => "~",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
    EqEqEq,
    NotEqEq,
    Lt,
    Lte,
    Gt,
    Gte,
    AndAnd,
    OrOr,
}

impl BinaryOp {
    pub fn as_str(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Exp => "**",
            BinaryOp::EqEqEq => "===",
            BinaryOp::NotEqEq => "!==",
            BinaryOp::Lt => "<",
            BinaryOp::Lte => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Gte => ">=",
            BinaryOp::AndAnd => "&&",
            BinaryOp::OrOr => "||",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TsType {
    Any,
    Unknown,
    Never,
    Void,
    Boolean,
    Number,
    String,
    Null,
    Undefined,
    TypeRef(String),
    Array(Box<TsType>),
    Union(Vec<TsType>),
    Func {
        params: Vec<Param>,
        ret: Box<TsType>,
    },
    Raw(String),
}
