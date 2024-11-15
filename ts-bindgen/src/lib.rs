mod impls;

pub mod registry;
pub mod ty;

pub use ts_bindgen_macros::TypeScriptDef;

pub use registry::TypeRegistry;
pub use ty::{Discriminator, TypeScriptType};

pub trait TypeScriptDef {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType;

    #[doc(hidden)]
    const _IS_OPTION: bool = false;
}
