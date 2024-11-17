use std::io::Write as _;

use ts_bindgen::{TypeRegistry, TypeScriptDef};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = TypeRegistry::default();

    client_sdk::models::gateway::message::ServerMsg::register(&mut registry);
    client_sdk::models::gateway::message::ClientMsg::register(&mut registry);

    client_sdk::api::commands::register_routes(&mut registry);

    let mut out = std::fs::File::create("api.ts")?;

    write!(out, "import type {{ ")?;

    for (idx, name) in registry.external().iter().enumerate() {
        if idx > 0 {
            write!(out, ", ")?;
        }

        write!(out, "{name}")?;
    }

    write!(
        out,
        " }} from './models';\nimport {{ command }} from './api';\n\n{}",
        registry.display()
    )?;

    Ok(())
}
