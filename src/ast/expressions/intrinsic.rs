use std::fmt::Display;
use std::str::FromStr;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntrinsicKind {
    Copy,

    // ix art intrinsics
    NegI { prec: usize },
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
    NegU { prec: usize },
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
    NegF { prec: usize },
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
        macro_rules! build_match {
            ( $({ $num:tt : $(($name: literal, $intrinsic: tt)),* $(,)? }),* $(,)?) => {
                match self {
                    $($(IntrinsicKind::$intrinsic { prec }=> format_binary_intrinsic(f, $name, NumericKind::$num, *prec),)*)*
                    IntrinsicKind::Copy => write!(f, "copy"),
                    IntrinsicKind::IToU { src_prec, out_prec } => {
                        write!(f, "i{src_prec}_to_u{out_prec}")
                    }
                    IntrinsicKind::IToF { src_prec, out_prec } => {
                        write!(f, "i{src_prec}_to_f{out_prec}")
                    }
                    IntrinsicKind::UToI { src_prec, out_prec } => {
                        write!(f, "u{src_prec}_to_i{out_prec}")
                    }
                    IntrinsicKind::UToF { src_prec, out_prec } => {
                        write!(f, "u{src_prec}_to_f{out_prec}")
                    }
                    IntrinsicKind::FToI { src_prec, out_prec } => {
                        write!(f, "f{src_prec}_to_i{out_prec}")
                    }
                    IntrinsicKind::FToU { src_prec, out_prec } => {
                        write!(f, "f{src_prec}_to_u{out_prec}")
                    }
                    #[allow(unreachable_patterns)]
                    _ => unreachable!()
                }
            };
        }

        build_match!(
            {Int:
                ("neg", NegI),
                ("add", AddI), ("sub", SubI),
                ("mul", MulI), ("div", DivI),
                ("eq", EqI), ("neq", NEqI),
                ("leq", LEqI), ("geq", GEqI),
                ("les", LesI), ("gre", GreI)},
            {Uint:
                ("neg", NegU),
                ("add", AddU), ("sub", SubU),
                ("mul", MulU), ("div", DivU),
                ("eq", EqU), ("neq", NEqU),
                ("leq", LEqU), ("geq", GEqU),
                ("les", LesU), ("gre", GreU)},
            {Float:
                ("neg", NegF),
                ("add", AddF), ("sub", SubF),
                ("mul", MulF), ("div", DivF),
                ("eq", EqF), ("neq", NEqF),
                ("leq", LEqF), ("geq", GEqF),
                ("les", LesF), ("gre", GreF)},
        )
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

        macro_rules! build_match {
            ( $({ $num:tt : $(($name: literal, $intrinsic: tt)),* $(,)? }),* $(,)?) => {
                match (op, operand_kind) {
                    $($(($name, NumericKind::$num) => Ok(Self::$intrinsic { prec: operand_prec }),)*)*
                    _ => anyhow::bail!("unknown intrinsic: {value}"),
                }
            };
        }

        build_match!(
            {Int:
                ("neg", NegI),
                ("add", AddI), ("sub", SubI),
                ("mul", MulI), ("div", DivI),
                ("eq", EqI), ("neq", NEqI),
                ("leq", LEqI), ("geq", GEqI),
                ("les", LesI), ("gre", GreI)},
            {Uint:
                ("neg", NegU),
                ("add", AddU), ("sub", SubU),
                ("mul", MulU), ("div", DivU),
                ("eq", EqU), ("neq", NEqU),
                ("leq", LEqU), ("geq", GEqU),
                ("les", LesU), ("gre", GreU)},
            {Float:
                ("neg", NegF),
                ("add", AddF), ("sub", SubF),
                ("mul", MulF), ("div", DivF),
                ("eq", EqF), ("neq", NEqF),
                ("leq", LEqF), ("geq", GEqF),
                ("les", LesF), ("gre", GreF)},
        )
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
