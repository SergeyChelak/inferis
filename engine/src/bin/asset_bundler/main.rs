use std::{
    env,
    fs::File,
    io::{self, Write},
};

use engine::{
    assets::{
        raw_asset::{self, AssetTypeID, RawAsset},
        raw_assets_from_text,
    },
    EngineError, EngineResult,
};

fn main() -> EngineResult<()> {
    let args = env::args().collect::<Vec<String>>();
    let Some(input) = args.get(1) else {
        return show_usage();
    };
    let Some(output) = args.get(2) else {
        return show_usage();
    };

    let raw_assets = raw_assets_from_text(input)?;
    write_assets(output, &raw_assets).map_err(|e| {
        let msg = format!("Failed to create bundle with error {}", e);
        EngineError::FileAccessError(msg)
    })?;
    println!("Done");
    Ok(())
}

fn write_assets(filename: &str, assets: &[RawAsset]) -> io::Result<()> {
    let mut file = File::create(filename)?;
    for asset in assets {
        write_asset(&mut file, asset)?;
    }
    file.flush()
}

fn write_asset(file: &mut File, asset: &RawAsset) -> io::Result<()> {
    let type_int = asset.asset_type as u8;
    write_buffer(file, type_int, asset.id.as_bytes())?;
    let id = asset.representation.id();
    use raw_asset::Representation::*;
    match &asset.representation {
        Text { value } => write_buffer(file, id, value.as_bytes())?,
        Binary { value } => write_buffer(file, id, value)?,
    };
    Ok(())
}

fn write_buffer(file: &mut File, id: AssetTypeID, buffer: &[u8]) -> io::Result<()> {
    file.write_all(&id.to_ne_bytes())?;
    let len = buffer.len();
    file.write_all(&len.to_ne_bytes())?;
    file.write_all(buffer)
}

fn show_usage() -> EngineResult<()> {
    let message = r#"
Asset bundle generator for Inferis project
Usage:
cargo run --bin asset_bundler <input> <output>
    <input>     asset registry filename
    <output>    output bundle filename
    "#;
    println!("{message}");
    Ok(())
}
