use ts_bindgen_macros::TypeScriptDef;

#[derive(Default, TypeScriptDef)]
pub struct Inner {
    a: i32,
}

#[derive(Default, TypeScriptDef)]
pub struct TestEmpty();

#[derive(Default, TypeScriptDef)]
#[serde(tag = "tag", content = "content")]
pub enum TestEnum {
    A(i32),
    B {
        a: i32,
    },
    #[default]
    C,
}

#[derive(Default, TypeScriptDef)]
pub enum TestEnum2 {
    A = 1,
    #[default]
    B,
    C = 3,
}

#[derive(Default, TypeScriptDef)]
#[serde(default, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Test {
    #[serde(skip_serializing)]
    a_small_field: i32,

    /// Bananas?
    bananas: i32,

    #[serde(flatten)]
    inner: Inner,
}

fn main() {
    use ts_bindgen::{TypeRegistry, TypeScriptDef};

    let mut registry = TypeRegistry::default();

    Test::register(&mut registry);
    TestEnum::register(&mut registry);
    TestEnum2::register(&mut registry);

    println!("{}", registry.fmt_to_string().unwrap());
}
