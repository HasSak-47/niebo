use std::fmt::Display;
use std::str::FromStr;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntrinsicKind {
    Copy,

    // ix art intrinsics
    AddI { prec: usize },
    SubI { prec: usize },
    MulI { prec: usize },
    DivI { prec: usize },

    // ix cmp intrinsics
    EqI { prec: usize },
    NEqI { prec: usize },
    LEqI { prec: usize },
    GEqI { prec: usize },

    LesI { prec: usize },
    GreI { prec: usize },
    IToU { src_prec: usize, out_prec: usize },
    IToF { src_prec: usize, out_prec: usize },

    // ux art intrinsics
    AddU { prec: usize },
    SubU { prec: usize },
    MulU { prec: usize },
    DivU { prec: usize },

    // ux cmp intrinsics
    EqU { prec: usize },
    NEqU { prec: usize },
    LEqU { prec: usize },
    GEqU { prec: usize },

    LesU { prec: usize },
    GreU { prec: usize },
    UToI { src_prec: usize, out_prec: usize },
    UToF { src_prec: usize, out_prec: usize },

    // fx intrinsics
    AddF { prec: usize },
    SubF { prec: usize },
    MulF { prec: usize },
    DivF { prec: usize },

    // fx cmp intrinsics
    EqF { prec: usize },
    NEqF { prec: usize },
    LEqF { prec: usize },
    GEqF { prec: usize },

    LesF { prec: usize },
    GreF { prec: usize },
    FToI { src_prec: usize, out_prec: usize },
    FToU { src_prec: usize, out_prec: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericKind {
    Int,
    Uint,
    Float,
}

impl Display for NumericKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericKind::Int => write!(f, "i"),
            NumericKind::Uint => write!(f, "u"),
            NumericKind::Float => write!(f, "f"),
        }
    }
}

fn parse_numeric_kind(value: &str) -> Option<(NumericKind, usize)> {
    let (kind, prec) = value.split_at(1);
    let prec = prec.parse().ok()?;
    match kind {
        "i" => Some((NumericKind::Int, prec)),
        "u" => Some((NumericKind::Uint, prec)),
        "f" => Some((NumericKind::Float, prec)),
        _ => None,
    }
}

fn format_binary_intrinsic(
    f: &mut std::fmt::Formatter<'_>,
    op: &str,
    kind: NumericKind,
    prec: usize,
) -> std::fmt::Result {
    write!(f, "{op}_{kind}{prec}")
}

impl Display for IntrinsicKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntrinsicKind::Copy => write!(f, "copy"),

            IntrinsicKind::AddI { prec } => {
                format_binary_intrinsic(f, "add", NumericKind::Int, *prec)
            }
            IntrinsicKind::SubI { prec } => {
                format_binary_intrinsic(f, "sub", NumericKind::Int, *prec)
            }
            IntrinsicKind::MulI { prec } => {
                format_binary_intrinsic(f, "mul", NumericKind::Int, *prec)
            }
            IntrinsicKind::DivI { prec } => {
                format_binary_intrinsic(f, "div", NumericKind::Int, *prec)
            }
            IntrinsicKind::EqI { prec } => {
                format_binary_intrinsic(f, "eq", NumericKind::Int, *prec)
            }
            IntrinsicKind::NEqI { prec } => {
                format_binary_intrinsic(f, "neq", NumericKind::Int, *prec)
            }
            IntrinsicKind::LEqI { prec } => {
                format_binary_intrinsic(f, "leq", NumericKind::Int, *prec)
            }
            IntrinsicKind::GEqI { prec } => {
                format_binary_intrinsic(f, "geq", NumericKind::Int, *prec)
            }
            IntrinsicKind::LesI { prec } => {
                format_binary_intrinsic(f, "les", NumericKind::Int, *prec)
            }
            IntrinsicKind::GreI { prec } => {
                format_binary_intrinsic(f, "gre", NumericKind::Int, *prec)
            }
            IntrinsicKind::IToU { src_prec, out_prec } => {
                write!(f, "i{src_prec}_to_u{out_prec}")
            }
            IntrinsicKind::IToF { src_prec, out_prec } => {
                write!(f, "i{src_prec}_to_f{out_prec}")
            }

            IntrinsicKind::AddU { prec } => {
                format_binary_intrinsic(f, "add", NumericKind::Uint, *prec)
            }
            IntrinsicKind::SubU { prec } => {
                format_binary_intrinsic(f, "sub", NumericKind::Uint, *prec)
            }
            IntrinsicKind::MulU { prec } => {
                format_binary_intrinsic(f, "mul", NumericKind::Uint, *prec)
            }
            IntrinsicKind::DivU { prec } => {
                format_binary_intrinsic(f, "div", NumericKind::Uint, *prec)
            }
            IntrinsicKind::EqU { prec } => {
                format_binary_intrinsic(f, "eq", NumericKind::Uint, *prec)
            }
            IntrinsicKind::NEqU { prec } => {
                format_binary_intrinsic(f, "neq", NumericKind::Uint, *prec)
            }
            IntrinsicKind::LEqU { prec } => {
                format_binary_intrinsic(f, "leq", NumericKind::Uint, *prec)
            }
            IntrinsicKind::GEqU { prec } => {
                format_binary_intrinsic(f, "geq", NumericKind::Uint, *prec)
            }
            IntrinsicKind::LesU { prec } => {
                format_binary_intrinsic(f, "les", NumericKind::Uint, *prec)
            }
            IntrinsicKind::GreU { prec } => {
                format_binary_intrinsic(f, "gre", NumericKind::Uint, *prec)
            }
            IntrinsicKind::UToI { src_prec, out_prec } => {
                write!(f, "u{src_prec}_to_i{out_prec}")
            }
            IntrinsicKind::UToF { src_prec, out_prec } => {
                write!(f, "u{src_prec}_to_f{out_prec}")
            }

            IntrinsicKind::AddF { prec } => {
                format_binary_intrinsic(f, "add", NumericKind::Float, *prec)
            }
            IntrinsicKind::SubF { prec } => {
                format_binary_intrinsic(f, "sub", NumericKind::Float, *prec)
            }
            IntrinsicKind::MulF { prec } => {
                format_binary_intrinsic(f, "mul", NumericKind::Float, *prec)
            }
            IntrinsicKind::DivF { prec } => {
                format_binary_intrinsic(f, "div", NumericKind::Float, *prec)
            }
            IntrinsicKind::EqF { prec } => {
                format_binary_intrinsic(f, "eq", NumericKind::Float, *prec)
            }
            IntrinsicKind::NEqF { prec } => {
                format_binary_intrinsic(f, "neq", NumericKind::Float, *prec)
            }
            IntrinsicKind::LEqF { prec } => {
                format_binary_intrinsic(f, "leq", NumericKind::Float, *prec)
            }
            IntrinsicKind::GEqF { prec } => {
                format_binary_intrinsic(f, "geq", NumericKind::Float, *prec)
            }
            IntrinsicKind::LesF { prec } => {
                format_binary_intrinsic(f, "les", NumericKind::Float, *prec)
            }
            IntrinsicKind::GreF { prec } => {
                format_binary_intrinsic(f, "gre", NumericKind::Float, *prec)
            }
            IntrinsicKind::FToI { src_prec, out_prec } => {
                write!(f, "f{src_prec}_to_i{out_prec}")
            }
            IntrinsicKind::FToU { src_prec, out_prec } => {
                write!(f, "f{src_prec}_to_u{out_prec}")
            }
        }
    }
}

impl FromStr for IntrinsicKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "copy" {
            return Ok(Self::Copy);
        }

        if let Some((src, out)) = value.split_once("_to_") {
            let (src_kind, src_prec) = parse_numeric_kind(src)
                .ok_or_else(|| anyhow::anyhow!("invalid intrinsic source type: {src}"))?;
            let (out_kind, out_prec) = parse_numeric_kind(out)
                .ok_or_else(|| anyhow::anyhow!("invalid intrinsic output type: {out}"))?;

            return match (src_kind, out_kind) {
                (NumericKind::Int, NumericKind::Uint) => Ok(Self::IToU { src_prec, out_prec }),
                (NumericKind::Int, NumericKind::Float) => Ok(Self::IToF { src_prec, out_prec }),
                (NumericKind::Uint, NumericKind::Int) => Ok(Self::UToI { src_prec, out_prec }),
                (NumericKind::Uint, NumericKind::Float) => Ok(Self::UToF { src_prec, out_prec }),
                (NumericKind::Float, NumericKind::Int) => Ok(Self::FToI { src_prec, out_prec }),
                (NumericKind::Float, NumericKind::Uint) => Ok(Self::FToU { src_prec, out_prec }),
                _ => anyhow::bail!("unsupported intrinsic conversion: {value}"),
            };
        }

        let mut parts = value.split('_');
        let op = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing intrinsic operation"))?;
        let operand = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing intrinsic operand type"))?;
        if parts.next().is_some() {
            anyhow::bail!("invalid intrinsic name: {value}");
        }

        let (operand_kind, operand_prec) = parse_numeric_kind(operand)
            .ok_or_else(|| anyhow::anyhow!("invalid intrinsic operand type: {operand}"))?;

        match (op, operand_kind) {
            ("add", NumericKind::Int) => Ok(Self::AddI { prec: operand_prec }),
            ("sub", NumericKind::Int) => Ok(Self::SubI { prec: operand_prec }),
            ("mul", NumericKind::Int) => Ok(Self::MulI { prec: operand_prec }),
            ("div", NumericKind::Int) => Ok(Self::DivI { prec: operand_prec }),
            ("eq", NumericKind::Int) => Ok(Self::EqI { prec: operand_prec }),
            ("neq", NumericKind::Int) => Ok(Self::NEqI { prec: operand_prec }),
            ("leq", NumericKind::Int) => Ok(Self::LEqI { prec: operand_prec }),
            ("geq", NumericKind::Int) => Ok(Self::GEqI { prec: operand_prec }),
            ("les", NumericKind::Int) => Ok(Self::LesI { prec: operand_prec }),
            ("gre", NumericKind::Int) => Ok(Self::GreI { prec: operand_prec }),

            ("add", NumericKind::Uint) => Ok(Self::AddU { prec: operand_prec }),
            ("sub", NumericKind::Uint) => Ok(Self::SubU { prec: operand_prec }),
            ("mul", NumericKind::Uint) => Ok(Self::MulU { prec: operand_prec }),
            ("div", NumericKind::Uint) => Ok(Self::DivU { prec: operand_prec }),
            ("eq", NumericKind::Uint) => Ok(Self::EqU { prec: operand_prec }),
            ("neq", NumericKind::Uint) => Ok(Self::NEqU { prec: operand_prec }),
            ("leq", NumericKind::Uint) => Ok(Self::LEqU { prec: operand_prec }),
            ("geq", NumericKind::Uint) => Ok(Self::GEqU { prec: operand_prec }),
            ("les", NumericKind::Uint) => Ok(Self::LesU { prec: operand_prec }),
            ("gre", NumericKind::Uint) => Ok(Self::GreU { prec: operand_prec }),

            ("add", NumericKind::Float) => Ok(Self::AddF { prec: operand_prec }),
            ("sub", NumericKind::Float) => Ok(Self::SubF { prec: operand_prec }),
            ("mul", NumericKind::Float) => Ok(Self::MulF { prec: operand_prec }),
            ("div", NumericKind::Float) => Ok(Self::DivF { prec: operand_prec }),
            ("eq", NumericKind::Float) => Ok(Self::EqF { prec: operand_prec }),
            ("neq", NumericKind::Float) => Ok(Self::NEqF { prec: operand_prec }),
            ("leq", NumericKind::Float) => Ok(Self::LEqF { prec: operand_prec }),
            ("geq", NumericKind::Float) => Ok(Self::GEqF { prec: operand_prec }),
            ("les", NumericKind::Float) => Ok(Self::LesF { prec: operand_prec }),
            ("gre", NumericKind::Float) => Ok(Self::GreF { prec: operand_prec }),
            _ => anyhow::bail!("unknown intrinsic: {value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Intrinsic {
    pub kind: IntrinsicKind,
    pub parameters: Vec<Expression>,
}

impl Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}({:?})", self.kind, self.parameters)
    }
}

impl Intrinsic {
    pub fn new(kind: IntrinsicKind, parameters: Vec<Expression>) -> Self {
        Self { kind, parameters }
    }
}
