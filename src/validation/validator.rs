use std::collections::{HashMap, HashSet};

use crate::{
    ast::{
        Definition,
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
    trait_registry: HashMap<QualifiedName, Trait>,
    scope: Scope,
}

impl Validator {
    fn generate_unary_operation<S: ToString>(name: S, return_ty: TypeID) -> Trait {
        let name = name.to_string();
        let mut t = Trait {
            functions: HashMap::new(),
        };
        t.functions.insert(
            name,
            FunctionDeclaration {
                constant: false,
                return_ty: Some(return_ty),
                parameters: vec![("self".to_string(), TypeID::self_())],
            },
        );

        return t;
    }

    fn generate_binary_operation<S: ToString>(name: S, return_ty: TypeID) -> Trait {
        let name = name.to_string();
        let mut t = Trait {
            functions: HashMap::new(),
        };
        t.functions.insert(
            name,
            FunctionDeclaration {
                constant: false,
                return_ty: Some(return_ty),
                parameters: vec![
                    ("self".to_string(), TypeID::self_()),
                    ("other".to_string(), TypeID::self_()),
                ],
            },
        );

        return t;
    }

    fn generate_unary_arithmetic_operation<S: ToString>(name: S) -> Trait {
        return Self::generate_unary_operation(name, TypeID::bool());
    }

    fn generate_binary_arithmetic_operation<S: ToString>(name: S) -> Trait {
        return Self::generate_binary_operation(name, TypeID::self_());
    }

    fn generate_binary_arithmetic_operations() -> HashMap<String, Trait> {
        return HashMap::from_iter([
            (
                "Add".to_string(),
                Self::generate_binary_arithmetic_operation("add"),
            ),
            (
                "Sub".to_string(),
                Self::generate_binary_arithmetic_operation("sub"),
            ),
            (
                "Mul".to_string(),
                Self::generate_binary_arithmetic_operation("mul"),
            ),
            (
                "Div".to_string(),
                Self::generate_binary_arithmetic_operation("div"),
            ),
            (
                "Mod".to_string(),
                Self::generate_binary_arithmetic_operation("mod"),
            ),
        ]);
    }

    fn generate_unary_arithmetic_operations() -> HashMap<String, Trait> {
        return HashMap::from_iter([
            (
                "Neg".to_string(),
                Self::generate_unary_arithmetic_operation("neg"),
            ),
            (
                "Inc".to_string(),
                Self::generate_unary_arithmetic_operation("inc"),
            ),
            (
                "Dec".to_string(),
                Self::generate_unary_arithmetic_operation("dec"),
            ),
        ]);
    }

    fn generate_binary_boolean_operations() -> HashMap<String, Trait> {
        return HashMap::from_iter([
            (
                "Eq".to_string(),
                Self::generate_binary_operation("add", TypeID::bool()),
            ),
            (
                "LEq".to_string(),
                Self::generate_binary_operation("sub", TypeID::bool()),
            ),
            (
                "GEq".to_string(),
                Self::generate_binary_operation("mul", TypeID::bool()),
            ),
            (
                "Les".to_string(),
                Self::generate_binary_operation("les", TypeID::bool()),
            ),
            (
                "Gre".to_string(),
                Self::generate_binary_operation("gre", TypeID::bool()),
            ),
        ]);
    }

    fn generate_unary_boolean_operations() -> HashMap<String, Trait> {
        return HashMap::from_iter([(
            "Not".to_string(),
            Self::generate_unary_operation("not", TypeID::bool()),
        )]);
    }

    pub fn new() -> Validator {
        let mut v = Validator::default();
        // register the operator traits
        for (name, trait_) in Self::generate_unary_arithmetic_operations() {
            v.trait_registry
                .insert(QualifiedName::from(format!("core::ops::{name}")), trait_);
        }

        for (name, trait_) in Self::generate_binary_arithmetic_operations() {
            v.trait_registry
                .insert(QualifiedName::from(format!("core::ops::{name}")), trait_);
        }

        for (name, trait_) in Self::generate_unary_boolean_operations() {
            v.trait_registry
                .insert(QualifiedName::from(format!("core::ops::{name}")), trait_);
        }

        for (name, trait_) in Self::generate_binary_boolean_operations() {
            v.trait_registry
                .insert(QualifiedName::from(format!("core::ops::{name}")), trait_);
        }

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

    fn process_definition(&mut self, def: &mut Definition) {
        use crate::ast::DefinitionKind as DK;
        match &mut def.kind {
            DK::FunctionDefinition(func) => {
                self.scope.local.push(HashMap::new());
                let local_scope = self.scope.local.last_mut().unwrap();
                for (name, ty) in &func.decl.parameters {
                    local_scope.insert(name.into(), ty.clone());
                }

                use crate::ast::statements::Statement as ST;
                for stmt in &mut func.body.statements {
                    match stmt {
                        ST::Return(Some(exp)) => self.determine_type(exp),
                        ST::Return(None) => {}
                        ST::Definition(def) => {
                            self.process_definition(def);
                        }
                        _ => todo!(),
                    }
                }

                self.scope.local.pop();
            }
            DK::Variable(var) => {
                todo!()
            }
            _ => {}
        }
    }

    pub fn process_project(&mut self, mut project: Project) -> anyhow::Result<()> {
        for def in &mut project.root_module.definitions {
            self.process_definition(def);
        }
        return Ok(());
    }

    pub fn determine_type(&mut self, e: &mut Expression) {
        todo!()
    }
}
