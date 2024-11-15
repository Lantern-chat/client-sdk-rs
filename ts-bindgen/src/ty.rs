use std::rc::Rc;

use indexmap::IndexSet;

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Discriminator {
    Simple(i64),
    BinaryHex(u64),
    String(&'static str),
}

impl fmt::Display for Discriminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Discriminator::Simple(i) => write!(f, "{}", i),
            Discriminator::BinaryHex(i) => write!(f, "0x{:x}", i),
            Discriminator::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeScriptType {
    Null,
    Undefined,
    Number(Option<f64>),
    String(Option<Rc<str>>),
    Boolean(Option<bool>),

    Array(Box<TypeScriptType>, Option<usize>),

    Interface {
        members: Vec<(String, TypeScriptType)>,
        extends: Vec<TypeScriptType>,
    },

    Union(Vec<TypeScriptType>),
    Intersection(Vec<TypeScriptType>),
    Enum(Vec<(String, Option<Discriminator>)>),
    Tuple(Vec<TypeScriptType>),
    Partial(Box<TypeScriptType>),

    /// [key: K]: T
    Map(Box<TypeScriptType>, Box<TypeScriptType>),

    /// `export const enum`
    ConstEnum(Vec<(String, Option<Discriminator>)>),

    /// Field of an enum: ty.value
    EnumValue(&'static str, &'static str),

    /// Type that's been registered with the type registry under the given name.
    Named(&'static str),
}

impl TypeScriptType {
    /// Performs simply cleanup of nested unions and intersections.
    pub fn unify(&mut self) {
        match self {
            TypeScriptType::Union(types) => {
                while types.iter().any(|ty| matches!(ty, TypeScriptType::Union(_))) {
                    let mut new_types = Vec::new();

                    for mut ty in types.drain(..) {
                        ty.unify();

                        match ty {
                            TypeScriptType::Union(mut inner_types) => new_types.append(&mut inner_types),
                            ty => new_types.push(ty),
                        }
                    }

                    *types = new_types;
                }
            }
            TypeScriptType::Intersection(types) => {
                while types.iter().any(|ty| matches!(ty, TypeScriptType::Intersection(_))) {
                    let mut new_types = Vec::new();

                    for mut ty in types.drain(..) {
                        ty.unify();

                        match ty {
                            TypeScriptType::Intersection(mut inner_types) => new_types.append(&mut inner_types),
                            ty => new_types.push(ty),
                        }
                    }

                    *types = new_types;
                }
            }
            TypeScriptType::Array(ty, _) => ty.unify(),
            TypeScriptType::Map(key, value) => {
                key.unify();
                value.unify();
            }
            TypeScriptType::Tuple(types) => {
                for ty in types {
                    ty.unify();
                }
            }
            TypeScriptType::Interface { members, .. } => members.iter_mut().for_each(|(_, ty)| ty.unify()),

            TypeScriptType::Partial(ty) => {
                // Partial<T> should also remove the `undefined` type from T if union
                ty.unify()
            }
            _ => {}
        }
    }
}

impl TypeScriptType {
    pub fn is_optional(&self) -> bool {
        match self {
            TypeScriptType::Union(types) => types.iter().any(|t| t.is_undefined()),
            _ => false,
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            TypeScriptType::Union(types) => types.iter().any(|t| t.is_null()),
            _ => false,
        }
    }

    pub const fn is_null(&self) -> bool {
        matches!(self, TypeScriptType::Null)
    }

    pub const fn is_undefined(&self) -> bool {
        matches!(self, TypeScriptType::Undefined)
    }
}

impl TypeScriptType {
    fn boxed(self) -> Box<TypeScriptType> {
        Box::new(self)
    }
}

impl TypeScriptType {
    pub fn interface(members: Vec<(String, TypeScriptType)>, extend_hint: usize) -> TypeScriptType {
        TypeScriptType::Interface {
            members,
            extends: Vec::with_capacity(extend_hint),
        }
    }

    pub const fn undefined() -> TypeScriptType {
        TypeScriptType::Undefined
    }

    pub const fn null() -> TypeScriptType {
        TypeScriptType::Null
    }

    pub const fn boolean() -> TypeScriptType {
        TypeScriptType::Boolean(None)
    }

    pub const fn boolean_value(value: bool) -> TypeScriptType {
        TypeScriptType::Boolean(Some(value))
    }

    pub const fn number() -> TypeScriptType {
        TypeScriptType::Number(None)
    }

    pub const fn number_value(value: f64) -> TypeScriptType {
        TypeScriptType::Number(Some(value))
    }

    pub const fn string() -> TypeScriptType {
        TypeScriptType::String(None)
    }

    pub fn string_value(value: impl Into<Rc<str>>) -> TypeScriptType {
        TypeScriptType::String(Some(value.into()))
    }

    pub fn into_nullable(self) -> TypeScriptType {
        if self.is_nullable() {
            return self;
        }

        self.union(TypeScriptType::Null)
    }

    pub fn into_optional(self) -> TypeScriptType {
        if self.is_optional() {
            return self;
        }

        self.union(TypeScriptType::Undefined)
    }

    #[doc(hidden)]
    pub fn _into_optional_internal(self, is_option: bool) -> TypeScriptType {
        match (self, is_option) {
            (TypeScriptType::Union(mut types), true) => {
                types.pop();
                types.push(TypeScriptType::Undefined);
                TypeScriptType::Union(types)
            }
            (a, _) => a.into_optional(),
        }
    }

    pub fn take_optional(&self) -> Result<TypeScriptType, &TypeScriptType> {
        match self {
            TypeScriptType::Union(types) if types.iter().any(|t| t.is_undefined()) => {
                let mut new_types = Vec::new();

                for ty in types {
                    if !ty.is_undefined() {
                        new_types.push(ty.clone());
                    }
                }

                Ok(TypeScriptType::Union(new_types))
            }
            _ => Err(self),
        }
    }

    pub fn into_array(self) -> TypeScriptType {
        TypeScriptType::Array(self.boxed(), None)
    }

    pub fn into_sized_array(self, size: usize) -> TypeScriptType {
        TypeScriptType::Array(self.boxed(), Some(size))
    }

    pub fn union(self, other: TypeScriptType) -> TypeScriptType {
        match (self, other) {
            (TypeScriptType::Union(mut types), TypeScriptType::Union(other_types)) => {
                types.extend(other_types);
                TypeScriptType::Union(types)
            }
            (TypeScriptType::Union(mut types), other) => {
                types.push(other);
                TypeScriptType::Union(types)
            }
            (a, TypeScriptType::Union(mut types)) => {
                types.insert(0, a);
                TypeScriptType::Union(types)
            }
            (a, other) => TypeScriptType::Union(vec![a, other]),
        }
    }

    pub fn intersection(self, other: TypeScriptType) -> TypeScriptType {
        match (self, other) {
            (TypeScriptType::Intersection(mut types), TypeScriptType::Intersection(other_types)) => {
                types.extend(other_types);
                TypeScriptType::Intersection(types)
            }
            (TypeScriptType::Intersection(mut types), other) => {
                types.push(other);
                TypeScriptType::Intersection(types)
            }
            (a, TypeScriptType::Intersection(mut types)) => {
                types.insert(0, a);
                TypeScriptType::Intersection(types)
            }
            (a, other) => TypeScriptType::Intersection(vec![a, other]),
        }
    }

    /// Merge `field` into this type.
    pub fn flatten(self, field: TypeScriptType) -> TypeScriptType {
        match (self, field) {
            (TypeScriptType::Named(a), TypeScriptType::Named(b)) => {
                if a == b {
                    return TypeScriptType::Named(a);
                }

                TypeScriptType::Union(vec![TypeScriptType::Named(a), TypeScriptType::Named(b)])
            }
            (TypeScriptType::Interface { members, mut extends }, TypeScriptType::Named(name)) => {
                extends.push(TypeScriptType::Named(name));

                TypeScriptType::Interface { members, extends }
            }
            (
                TypeScriptType::Interface {
                    members: mut am,
                    extends: mut ae,
                },
                TypeScriptType::Interface {
                    members: bm,
                    extends: be,
                },
            ) => {
                am.extend(bm);
                ae.extend(be);

                TypeScriptType::Interface {
                    members: am,
                    extends: ae,
                }
            }
            (a @ TypeScriptType::Interface { .. }, TypeScriptType::Null | TypeScriptType::Undefined) => a,

            // #[serde(flatten, default, skip_serializing_if = "...")]
            (TypeScriptType::Interface { members, extends }, TypeScriptType::Union(types))
                if matches!(&types[..], &[TypeScriptType::Named(_), TypeScriptType::Undefined]) =>
            {
                let mut extends = extends.clone();
                extends.push(TypeScriptType::Partial(types[0].clone().boxed()));

                TypeScriptType::Interface { members, extends }
            }

            (s, f) => unreachable!("flatten called with invalid types: {s:?}, {f:?}"),
        }
    }
}
