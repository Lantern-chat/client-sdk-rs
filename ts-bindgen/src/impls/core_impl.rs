use super::{TypeRegistry, TypeScriptDef, TypeScriptType};

impl<T: TypeScriptDef> TypeScriptDef for Option<T> {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        T::register(registry).into_nullable()
    }

    const _IS_OPTION: bool = true;
}

impl TypeScriptDef for () {
    fn register(_registry: &mut TypeRegistry) -> TypeScriptType {
        TypeScriptType::Null
    }
}

impl TypeScriptDef for str {
    fn register(_registry: &mut TypeRegistry) -> TypeScriptType {
        TypeScriptType::String(None)
    }
}

impl<T: TypeScriptDef> TypeScriptDef for &'_ T {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        T::register(registry)
    }
}

impl TypeScriptDef for bool {
    fn register(_registry: &mut TypeRegistry) -> TypeScriptType {
        TypeScriptType::Boolean(None)
    }
}

macro_rules! impl_numbers {
    ($($num:ty),*) => {$(
        impl TypeScriptDef for $num {
            fn register(_registry: &mut TypeRegistry) -> TypeScriptType {
                TypeScriptType::number()
            }
        }
    )*}
}

impl_numbers!(
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    core::num::NonZeroU8,
    core::num::NonZeroU16,
    core::num::NonZeroU32,
    core::num::NonZeroU64,
    core::num::NonZeroU128,
    core::num::NonZeroUsize,
    core::num::NonZeroI8,
    core::num::NonZeroI16,
    core::num::NonZeroI32,
    core::num::NonZeroI64,
    core::num::NonZeroI128,
    core::num::NonZeroIsize
);
