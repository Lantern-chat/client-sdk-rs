use std::collections::{HashMap, HashSet};

use super::{TypeRegistry, TypeScriptDef, TypeScriptType};

impl<K: TypeScriptDef, T: TypeScriptDef, H> TypeScriptDef for HashMap<K, T, H> {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        TypeScriptType::Map(K::register(registry).into(), T::register(registry).into())
    }
}

impl<T: TypeScriptDef, H> TypeScriptDef for HashSet<T, H> {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        TypeScriptType::Array(T::register(registry).into(), None)
    }
}
