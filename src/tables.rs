use std::{collections::HashMap};
use std::sync::LazyLock;

use crate::common::{DataType, Function};

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

pub const PRECISION_KEYWORDS: [&str; 3] = [
    "highp",
    "mediump",
    "lowp"
];

//TODO
pub const TYPE_KEYWORDS: LazyLock<HashMap<&'static str, DataType>> = LazyLock::new(|| {
    return HashMap::from([
        ( "void", DataType::Void ),
        ( "bool", DataType::Bool ),
        ( "int", DataType::I32 ),
        ( "float", DataType::F32 ),
        ( "vec2", DataType::FVec2 ),
        ( "vec3", DataType::FVec3 ),
        ( "vec4", DataType::FVec4 ),
        ( "ivec2", DataType::IVec2 ),
        ( "ivec3", DataType::IVec3 ),
        ( "ivec4", DataType::IVec4 ),
        ( "uvec2", DataType::UVec2 ),
        ( "uvec3", DataType::UVec3 ),
        ( "uvec4", DataType::UVec4 ),
        ( "mat2", DataType::Mat2 ),
        ( "mat3", DataType::Mat3 ),
        ( "mat4", DataType::Mat4 ),
        ( "sampler2D", DataType::FSamp2D ),
        ( "isampler2D", DataType::ISamp2D ),
        ( "usampler2D", DataType::USamp2D ),
        ( "sampler2DArray", DataType::FSamp2DArr ),
        ( "isampler2DArray", DataType::ISamp2DArr ),
        ( "usampler2DArray", DataType::USamp2DArr ),
        ( "sampler3D", DataType::FSamp3D ),
        ( "isampler3D", DataType::ISamp3D ),
        ( "usampler3D", DataType::USamp3D ),
        ( "samplerCube", DataType::SampCube ),
        ( "samplerCubeArray", DataType::SampCubeArr ),
        ( "samplerExternalOES", DataType::EOES ),
    ]);
});

//TODO
pub const GLOBALS: LazyLock<HashMap<&'static str, DataType>> = LazyLock::new(|| {
    return HashMap::from([
        ( "COLOR", DataType::FVec4 ),
        ( "TEXTURE", DataType::FSamp2D ),
        ( "TEXTURE_PIXEL_SIZE", DataType::FVec2 ),
        ( "SCREEN_PIXEL_SIZE", DataType::FVec2 ),
        ( "UV", DataType::FVec2 ),
        ( "SCREEN_UV", DataType::FVec2 ),
        ( "NORMAL", DataType::FVec3 ),
    ]);
});

//TODO
pub const BUILT_IN_FUNCTIONS: LazyLock<HashMap<&'static str, Function>> = LazyLock::new(|| {
    return HashMap::from([
        ( "radians", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "degrees", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "sin", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "cos", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "tan", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "asin", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "acos", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "atan", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "atan", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "sinh", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "cosh", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "tanh", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "asinh", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "acosh", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "atanh", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "pow", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "exp", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "exp2", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "log", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "log2", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "sqrt", Function { args: vec![], ret_type: DataType::F32 } ),
        ( "inversesqrt", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "abs", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "sign", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "sign", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "floor", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "round", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "roundEven", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "trunc", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "ceil", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "fract", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "mod", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "mod", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "min", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "min", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "min", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "min", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "min", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "min", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "max", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "max", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "max", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "max", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "max", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "max", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "clamp", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "clamp", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "clamp", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "clamp", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "clamp", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "clamp", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "mix", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "mix", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "mix", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "fma", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "step", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "smoothstep", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "isnan", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "isinf", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "floatBitToInt", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "floatBitToUint", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "intBitsToFloat", Function { args: vec![], ret_type: DataType::Unknown } ),
        ( "uintBitsToFloat", Function { args: vec![], ret_type: DataType::Unknown } ),
    ]);
});