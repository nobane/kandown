use anyhow::bail;
// kandown-wasm/examples/generate_ts_types.rs
use kandown_wasm::KanbanViewData;
use reflect_to::ToTypescript;
// Import types DIRECTLY from the kandown_wasm crate itself
use std::{env, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        bail!(
            "Usage: {} <output_path.ts>",
            args.first().map_or("generate_ts_types", |s| s.as_str())
        )
    }

    println!("Initializing TypeScript generator...");
    let mut generator = ToTypescript::default();

    println!("Registering types from kandown_wasm...");
    generator.add_type::<KanbanViewData>()?;

    if args[1].as_str() == "--stdout" {
        let output = generator.generate()?;
        println!("{output}");
        Ok(())
    } else {
        let output_path = PathBuf::from(&args[1]);
        println!("Generating TypeScript code...");
        match generator.write_to_file(&output_path) {
            Ok(_) => {
                println!("Successfully generated TypeScript types to: {output_path:?}",);
                Ok(())
            }
            Err(e) => {
                bail!("Failed to write TypeScript file to {output_path:?}: {e}",);
            }
        }
    }
}
