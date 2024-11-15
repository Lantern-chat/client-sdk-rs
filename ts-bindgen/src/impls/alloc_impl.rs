extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::{collections::VecDeque, vec::Vec};

use super::{TypeRegistry, TypeScriptDef, TypeScriptType};

impl TypeScriptDef for String {
    fn register(_registry: &mut TypeRegistry) -> TypeScriptType {
        TypeScriptType::String(None)
    }
}

impl<T: TypeScriptDef> TypeScriptDef for Box<T> {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        T::register(registry)
    }
}

impl<T: TypeScriptDef> TypeScriptDef for Rc<T> {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        T::register(registry)
    }
}

impl<T: TypeScriptDef> TypeScriptDef for Arc<T> {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        T::register(registry)
    }
}

impl<T: TypeScriptDef> TypeScriptDef for Vec<T> {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        T::register(registry).into_array()
    }
}

impl<T: TypeScriptDef> TypeScriptDef for VecDeque<T> {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        T::register(registry).into_array()
    }
}
