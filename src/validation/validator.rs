use std::collections::{HashMap, HashSet};

use crate::{
    ast::{
        expressions::{Expression, ExpressionKind, operations::BinaryOperator},
        function::{FunctionDeclaration, FunctionDefinition},
        project::Project,
        traits::Trait,
    },
    general::{
        naming::QualifiedName,
        types::{
            typeid::{FunctionType, TypeID},
            typeshape::TypeShape,
        },
    },
};

/**
 * bare bones type information, that only exists to make sure that the code is type safe
 * will be a pain in the ass later cuz I didn't take into account templates
 */
struct TypeData {
    shape: TypeShape,
    methods: HashMap<String, FunctionType>,
    traits: HashSet<QualifiedName>,
}

impl TypeData {
    pub fn new(shape: TypeShape) -> TypeData {
        return TypeData {
            shape,
            methods: HashMap::new(),
            traits: HashSet::new(),
        };
    }

    pub fn generate_arithmetic_traits() -> HashSet<QualifiedName> {
        return HashSet::from_iter([
            QualifiedName::from("core::ops::Add"),
            QualifiedName::from("core::ops::Sub"),
            QualifiedName::from("core::ops::Mul"),
            QualifiedName::from("core::ops::Div"),
            QualifiedName::from("core::ops::Mod"),
            QualifiedName::from("core::ops::Neg"),
            QualifiedName::from("core::ops::Eq"),
            QualifiedName::from("core::ops::Neq"),
            QualifiedName::from("core::ops::Leq"),
            QualifiedName::from("core::ops::Geq"),
            QualifiedName::from("core::ops::Le"),
            QualifiedName::from("core::ops::Ge"),
        ]);
    }

    pub fn generate_int(prec: usize) -> TypeData {
        return TypeData {
            shape: TypeShape::int_p(prec),
            methods: HashMap::new(),
            traits: Self::generate_arithmetic_traits(),
        };
    }

    pub fn generate_uint(prec: usize) -> TypeData {
        return TypeData {
            shape: TypeShape::uint_p(prec),
            methods: HashMap::new(),
            traits: Self::generate_arithmetic_traits(),
        };
    }

    pub fn generate_float(prec: usize) -> TypeData {
        return TypeData {
            shape: TypeShape::float_p(prec),
            methods: HashMap::new(),
            traits: Self::generate_arithmetic_traits(),
        };
    }
}

#[derive(Default)]
struct TypeRegistry {
    types: HashMap<TypeID, TypeData>,
}

#[derive(Default)]
struct Scope {
    global: HashMap<QualifiedName, TypeID>,
    local: Vec<HashMap<QualifiedName, TypeID>>,
}
#[derive(Default)]
pub struct Validator {
    ty_registry: TypeRegistry,
    scope: Scope,
}

impl Validator {
    fn generate_arithmetic_binary_operation<S: ToString>(name: S) -> Trait {
        let name = name.to_string();
        let mut t = Trait {
            functions: HashMap::new(),
        };
        t.functions.insert(
            name,
            FunctionDeclaration {
                constant: false,
                return_ty: Some(TypeID::self_()),
                parameters: vec![
                    ("self".to_string(), TypeID::self_()),
                    ("other".to_string(), TypeID::self_()),
                ],
            },
        );

        return t;
    }

    fn generate_binary_operations() -> Vec<Trait> {
        return vec![
            Self::generate_arithmetic_binary_operation("add"),
            Self::generate_arithmetic_binary_operation("sub"),
            Self::generate_arithmetic_binary_operation("mul"),
            Self::generate_arithmetic_binary_operation("div"),
            Self::generate_arithmetic_binary_operation("mod"),
        ];
    }

    pub fn new() -> Validator {
        let mut v = Validator::default();

        // register the primitive types
        // int register
        for bit_size in &[8, 16, 32, 64] {
            v.ty_registry
                .types
                .insert(TypeID::int_p(*bit_size), TypeData::generate_int(*bit_size));
            v.ty_registry.types.insert(
                TypeID::uint_p(*bit_size),
                TypeData::generate_uint(*bit_size),
            );
        }
        // float register
        for bit_size in &[32, 64] {
            v.ty_registry.types.insert(
                TypeID::float_p(*bit_size),
                TypeData::generate_float(*bit_size),
            );
        }
        // void register
        v.ty_registry
            .types
            .insert(TypeID::void(), TypeData::new(TypeShape::void()));
        // bool register
        v.ty_registry.types.insert(
            TypeID::bool(),
            TypeData {
                shape: TypeShape::bool(),
                methods: HashMap::new(),
                traits: HashSet::from_iter([]),
            },
        );

        return v;
    }

    pub fn process_project(project: Project) -> anyhow::Result<()> {
        return Ok(());
    }

    pub fn determine_type(&mut self, e: &mut Expression) {
        todo!()
    }
}
