use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataType {
    Unknown,
    I8,
    I16,
    I32,
    F8,
    F16,
    F32,
    FVec2,
    FVec3,
    FVec4,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
    Mat2,
    Mat3,
    Mat4,
    FSamp2D,
    ISamp2D,
    USamp2D,
    FSamp2DArr,
    ISamp2DArr,
    USamp2DArr,
    FSamp3D,
    ISamp3D,
    USamp3D,
    SampCube,
    SampCubeArr,
    EOES,
    Bool,
    Fn,
    Void
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DataType::Unknown => "unknown",
            DataType::I8 => "int (lowp)",
            DataType::I16 => "int (mediump)",
            DataType::I32 => "int (highp)",
            DataType::F8 => "float (lowp)",
            DataType::F16 => "float (mediump)",
            DataType::F32 => "float (highp)",
            DataType::FVec2 => "vec2",
            DataType::FVec3 => "vec3",
            DataType::FVec4 => "vec4",
            DataType::IVec2 => "ivec2",
            DataType::IVec3 => "ivec3",
            DataType::IVec4 => "ivec4",
            DataType::UVec2 => "uvec2",
            DataType::UVec3 => "uvec3",
            DataType::UVec4 => "uvec4",
            DataType::Mat2 => "mat2",
            DataType::Mat3 => "mat3",
            DataType::Mat4 => "mat4",
            DataType::FSamp2D => "sampler2D",
            DataType::ISamp2D => "isampler2D",
            DataType::USamp2D => "usampler2D",
            DataType::FSamp2DArr => "sampler2DArray",
            DataType::ISamp2DArr => "isampler2DArray",
            DataType::USamp2DArr => "usampler2DArray",
            DataType::FSamp3D => "sampler3D",
            DataType::ISamp3D => "isampler3D",
            DataType::USamp3D => "usampler3D",
            DataType::SampCube => "samplerCube",
            DataType::SampCubeArr => "samplerCubeArray",
            DataType::EOES => "samplerExternalOES",
            DataType::Bool => "bool",
            DataType::Fn => "function",
            DataType::Void => "void"
        };

        write!(f, "{}", s)
    }
}

// TODO: create a TokenKind for Vecs and Mats maybe?
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TokenKind {
    Symbol,
    Operator,
    MiscKeyword,
    TypeKeyword,
    Ident(DataType),
    Global(DataType),
    IntLit,
    FloatLit,
    Comment,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TokenKind::Symbol => "symbol",
            TokenKind::Operator => "operator",
            TokenKind::MiscKeyword => "keyword",
            TokenKind::TypeKeyword => "type keyword",
            TokenKind::Ident(data_type) => &data_type.to_string(),
            TokenKind::Global(data_type) => &data_type.to_string(),
            TokenKind::IntLit => "int literal",
            TokenKind::FloatLit => "float literal",
            TokenKind::Comment => "comment"
        };

        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Token<'a> {
    pub value: &'a str,
    pub kind: TokenKind,
    pub line: usize,
    pub tail: usize,
    pub is_mut: bool,
}

impl<'a> Token<'a> {
    pub fn is_ident(&'a self) -> bool {
        return matches!(self.kind, TokenKind::Ident(_));
    }

    pub fn len(&'a self) -> usize {
        return self.value.chars().count();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SymbolType {
    ParenOpen,
    ParenClosed,
    SquareOpen,
    SquareClosed,
    CurlyOpen,
    CurlyClosed
}

pub struct Function {
    //TODO
    pub args: Option<i32>
}