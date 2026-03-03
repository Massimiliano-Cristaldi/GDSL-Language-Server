use std::{collections::HashMap, sync::LazyLock};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataType {
    Unknown,
    Int(IntType),
    Float(FloatType),
    Vec(VecType),
    Mat(MatType),
    Sampler(SamplerType),
    Bool,
    Function,
    Void
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IntType {
    I8,
    I16,
    I32
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatType {
    F8,
    F16,
    F32
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VecType {
    FVec2,
    FVec3,
    FVec4,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MatType {
    Mat2,
    Mat3,
    Mat4
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SamplerType {
    F2D,
    I2D,
    U2D,
    F2DArr,
    I2DArr,
    U2DArr,
    F3D,
    I3D,
    U3D,
    Cube,
    CubeArr,
    EOES
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
    args: Option<i32>
}

pub const OPERATORS: [&str; 4] = [
    "+",
    "-",
    "/",
    "*"
];

pub const SYMBOLS: [&str; 10] = [
    "(",
    ")",
    "[",
    "]",
    "{",
    "}",
    "=",
    ";",
    ",",
    ".",
];

//TODO
pub const MISC_KEYWORDS: [&str; 8] = [
    "shader_type",
    "render_mode",
    "canvas_item",
    "uniform",
    "for",
    "return",
    "flat",
    "smooth"
];

//TODO
pub const TYPE_KEYWORDS: LazyLock<HashMap<&'static str, DataType>> = LazyLock::new(|| {
    return HashMap::from([
        ( "void", DataType::Void ),
        ( "bool", DataType::Bool ),
        ( "int", DataType::Int(IntType::I32) ),
        ( "float", DataType::Float(FloatType::F32) ),
        ( "vec2", DataType::Vec(VecType::FVec2) ),
        ( "vec3", DataType::Vec(VecType::FVec3) ),
        ( "vec4", DataType::Vec(VecType::FVec4) ),
        ( "ivec2", DataType::Vec(VecType::IVec2) ),
        ( "ivec3", DataType::Vec(VecType::IVec3) ),
        ( "ivec4", DataType::Vec(VecType::IVec4) ),
        ( "uvec2", DataType::Vec(VecType::UVec2) ),
        ( "uvec3", DataType::Vec(VecType::UVec3) ),
        ( "uvec4", DataType::Vec(VecType::UVec4) ),
        ( "mat2", DataType::Mat(MatType::Mat2) ),
        ( "mat3", DataType::Mat(MatType::Mat3) ),
        ( "mat4", DataType::Mat(MatType::Mat4) ),
        ( "sampler2D", DataType::Sampler(SamplerType::F2D) ),
        ( "isampler2D", DataType::Sampler(SamplerType::I2D) ),
        ( "usampler2D", DataType::Sampler(SamplerType::U2D) ),
        ( "sampler2DArray", DataType::Sampler(SamplerType::F2DArr) ),
        ( "isampler2DArray", DataType::Sampler(SamplerType::I2DArr) ),
        ( "usampler2DArray", DataType::Sampler(SamplerType::U2DArr) ),
        ( "sampler3D", DataType::Sampler(SamplerType::F3D) ),
        ( "isampler3D", DataType::Sampler(SamplerType::I3D) ),
        ( "usampler3D", DataType::Sampler(SamplerType::U3D) ),
        ( "samplerCube", DataType::Sampler(SamplerType::Cube) ),
        ( "samplerCubeArray", DataType::Sampler(SamplerType::CubeArr) ),
        ( "samplerExternalOES", DataType::Sampler(SamplerType::EOES) ),
    ]);
});

pub const PRECISION_KEYWORDS: [&str; 3] = [
    "highp",
    "mediump",
    "lowp"
];

pub const GLOBALS: LazyLock<HashMap<&'static str, DataType>> = LazyLock::new(|| {
    return HashMap::from([
        ( "COLOR", DataType::Vec(VecType::FVec4) ),
        ( "TEXTURE", DataType::Sampler(SamplerType::F2D) ),
        ( "TEXTURE_PIXEL_SIZE", DataType::Vec(VecType::FVec2) ),
        ( "SCREEN_PIXEL_SIZE", DataType::Vec(VecType::FVec2) ),
        ( "UV", DataType::Vec(VecType::FVec2) ),
        ( "SCREEN_UV", DataType::Vec(VecType::FVec2) ),
        ( "NORMAL", DataType::Vec(VecType::FVec3) ),
    ]);
});

pub const BUILT_IN_FUNCTIONS: LazyLock<HashMap<&'static str, Function>> = LazyLock::new(|| {
    return HashMap::from([
        ( "radians", Function { args: None } ),
        ( "degrees", Function { args: None } ),
        ( "sin", Function { args: None } ),
        ( "cos", Function { args: None } ),
        ( "tan", Function { args: None } ),
        ( "asin", Function { args: None } ),
        ( "acos", Function { args: None } ),
        ( "atan", Function { args: None } ),
        ( "atan", Function { args: None } ),
        ( "sinh", Function { args: None } ),
        ( "cosh", Function { args: None } ),
        ( "tanh", Function { args: None } ),
        ( "asinh", Function { args: None } ),
        ( "acosh", Function { args: None } ),
        ( "atanh", Function { args: None } ),
        ( "pow", Function { args: None } ),
        ( "exp", Function { args: None } ),
        ( "exp2", Function { args: None } ),
        ( "log", Function { args: None } ),
        ( "log2", Function { args: None } ),
        ( "sqrt", Function { args: None } ),
        ( "inversesqrt", Function { args: None } ),
        ( "abs", Function { args: None } ),
        ( "sign", Function { args: None } ),
        ( "sign", Function { args: None } ),
        ( "floor", Function { args: None } ),
        ( "round", Function { args: None } ),
        ( "roundEven", Function { args: None } ),
        ( "trunc", Function { args: None } ),
        ( "ceil", Function { args: None } ),
        ( "fract", Function { args: None } ),
        ( "mod", Function { args: None } ),
        ( "mod", Function { args: None } ),
        ( "min", Function { args: None } ),
        ( "min", Function { args: None } ),
        ( "min", Function { args: None } ),
        ( "min", Function { args: None } ),
        ( "min", Function { args: None } ),
        ( "min", Function { args: None } ),
        ( "max", Function { args: None } ),
        ( "max", Function { args: None } ),
        ( "max", Function { args: None } ),
        ( "max", Function { args: None } ),
        ( "max", Function { args: None } ),
        ( "max", Function { args: None } ),
        ( "clamp", Function { args: None } ),
        ( "clamp", Function { args: None } ),
        ( "clamp", Function { args: None } ),
        ( "clamp", Function { args: None } ),
        ( "clamp", Function { args: None } ),
        ( "clamp", Function { args: None } ),
        ( "mix", Function { args: None } ),
        ( "mix", Function { args: None } ),
        ( "mix", Function { args: None } ),
        ( "fma", Function { args: None } ),
        ( "step", Function { args: None } ),
        ( "smoothstep", Function { args: None } ),
        ( "isnan", Function { args: None } ),
        ( "isinf", Function { args: None } ),
        ( "floatBitToInt", Function { args: None } ),
        ( "floatBitToUint", Function { args: None } ),
        ( "intBitsToFloat", Function { args: None } ),
        ( "uintBitsToFloat", Function { args: None } ),
    ]);
});