use std::borrow::Cow;

use core::fmt;

type Comment = Cow<'static, str>;
type Name = Cow<'static, str>;

/// Integer that'll be printed as a hexadecimal number in TypeScript.
///
/// This utility type ensures that values won't be sign-extended when converted
/// to TypeScript integers, even if negative signed integers are used. All values
/// are treated as unsigned integers for printing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BinaryInteger(pub u128);

macro_rules! impl_binary_integer {
    ($($ty:ty as $uty:ty),*) => {
        $(
            impl From<$ty> for BinaryInteger {
                #[inline(always)]
                fn from(value: $ty) -> Self {
                    BinaryInteger(value as $uty as u128)
                }
            }

            impl From<$uty> for BinaryInteger {
                #[inline(always)]
                fn from(value: $uty) -> Self {
                    BinaryInteger(value as u128)
                }
            }
        )*
    };
}

impl_binary_integer!(i8 as u8, i16 as u16, i32 as u32, i64 as u64, i128 as u128);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Discriminator {
    Simple(i64),
    BinaryHex(BinaryInteger),
    String(&'static str),
}

// JavaScript can't represent integers larger than or equal to 2^53 -1,
// so we need to represent them as strings.
pub const MAX_SAFE_NUMBER: u64 = (1 << 53) - 1;

impl fmt::Display for Discriminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Discriminator::Simple(i) => {
                if i.unsigned_abs() > MAX_SAFE_NUMBER {
                    write!(f, "\"{}\"", i)
                } else {
                    write!(f, "{}", i)
                }
            }
            Discriminator::BinaryHex(i) => {
                let i = i.0;

                if i > (MAX_SAFE_NUMBER as u128) {
                    write!(f, "\"0x{:x}\"", i)
                } else {
                    write!(f, "0x{:x}", i)
                }
            }
            Discriminator::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeScriptType {
    Null,
    Undefined,
    Number(Option<f64>),
    String(Option<Cow<'static, str>>),
    Boolean(Option<bool>),

    Array(Box<TypeScriptType>, Option<usize>),

    /// Array or Tuple literal
    ArrayLiteral(Vec<TypeScriptType>),

    Interface {
        members: Vec<(Name, TypeScriptType, Comment)>,
        extends: Vec<TypeScriptType>,
    },

    Union(Vec<TypeScriptType>),
    Intersection(Vec<TypeScriptType>),
    Enum(Vec<(Name, Option<Discriminator>, Comment)>),
    Tuple(Vec<(TypeScriptType, Comment)>),
    Partial(Box<TypeScriptType>),
    ReadOnly(Box<TypeScriptType>),

    /// [key: K]: T
    Map(Box<TypeScriptType>, Box<TypeScriptType>),

    /// `export const enum`
    ConstEnum(Vec<(Name, Option<Discriminator>, Comment)>),

    /// Field of an enum: ty.value
    EnumValue(&'static str, &'static str),

    /// Type that's been registered with the type registry under the given name.
    Named(&'static str),

    ApiDecl {
        // vec of EnumValue entries
        command_flags: Vec<TypeScriptType>,

        name: Name,
        method: Name,

        form_type: Box<TypeScriptType>,
        return_type: Box<TypeScriptType>,
        body_type: Option<Box<TypeScriptType>>,

        // if the path contains `${`, it'll be treated as a template string
        path: &'static str,
    },
}

impl TypeScriptType {
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            TypeScriptType::Null
                | TypeScriptType::Undefined
                | TypeScriptType::Number(_)
                | TypeScriptType::String(_)
                | TypeScriptType::Boolean(_)
        )
    }

    pub fn is_key_type(&self) -> bool {
        matches!(
            self,
            TypeScriptType::String(_) | TypeScriptType::Number(_) | TypeScriptType::Boolean(_)
        )
    }

    pub fn is_literal(&self) -> bool {
        match self {
            TypeScriptType::Number(Some(_))
            | TypeScriptType::String(Some(_))
            | TypeScriptType::Boolean(Some(_))
            | TypeScriptType::EnumValue(_, _)
            | TypeScriptType::ArrayLiteral(_) => true,

            TypeScriptType::ReadOnly(ty) => ty.is_literal(),
            _ => false,
        }
    }

    pub fn is_value(&self) -> bool {
        if self.is_literal() {
            return true;
        }

        matches!(
            self,
            TypeScriptType::ConstEnum(_) | TypeScriptType::Enum(_) | TypeScriptType::ApiDecl { .. }
        )
    }

    /// Performs simple cleanup of nested unions and intersections and removes duplicates.
    pub fn unify(&mut self) {
        let is_union = matches!(self, TypeScriptType::Union(_));

        match self {
            // unifying unions and intersections are very similar, but cannot be mixed.
            TypeScriptType::Union(types) | TypeScriptType::Intersection(types) => loop {
                let mut new_types = Vec::new();

                for mut ty in types.drain(..) {
                    ty.unify();

                    let inner_types = match ty {
                        TypeScriptType::Union(inner_types) if is_union => inner_types,
                        TypeScriptType::Intersection(inner_types) if !is_union => inner_types,
                        ty => {
                            if !new_types.contains(&ty) {
                                new_types.push(ty);
                            }

                            continue;
                        }
                    };

                    for ty in inner_types {
                        if !new_types.contains(&ty) {
                            new_types.push(ty);
                        }
                    }
                }

                *types = new_types;

                if !types.iter().any(|ty| match ty {
                    TypeScriptType::Union(_) if is_union => true,
                    TypeScriptType::Intersection(_) if !is_union => true,
                    _ => false,
                }) {
                    break;
                }
            },
            TypeScriptType::Array(ty, _) => ty.unify(),
            TypeScriptType::Map(key, value) => {
                key.unify();
                value.unify();
            }
            TypeScriptType::Tuple(types) => {
                for (ty, _) in types {
                    ty.unify();
                }
            }
            TypeScriptType::Interface { members, .. } => members.iter_mut().for_each(|(_, ty, _)| ty.unify()),

            TypeScriptType::Partial(ty) => {
                // Partial<T> should also remove the `undefined` type from T if union
                if let TypeScriptType::Union(types) = &mut **ty {
                    types.retain_mut(|ty| !ty.is_undefined());
                }

                ty.unify();

                // remove nested partials
                if let TypeScriptType::Partial(inner) = &**ty {
                    *ty = inner.clone();
                }
            }
            TypeScriptType::ReadOnly(ty) => {
                ty.unify();

                // remove nested readonlys
                if let TypeScriptType::ReadOnly(inner) = &**ty {
                    *ty = inner.clone();
                }
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
    pub fn interface(members: Vec<(Name, TypeScriptType, Comment)>, extend_hint: usize) -> TypeScriptType {
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

    pub fn string_value(value: impl Into<Cow<'static, str>>) -> TypeScriptType {
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
