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