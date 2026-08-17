use std::{
    fs::File,
    io::{self, ErrorKind, Read},
    mem,
};

use crate::{
    assets::raw_asset::{Type, REPRESENTATION_BINARY, REPRESENTATION_TEXT},
    EngineError, EngineResult,
};

use super::{
    raw_asset::{RawAsset, Representation, TypeID},
    Data,
};

pub fn raw_assets_from_bundle(path: &str) -> EngineResult<Vec<RawAsset>> {
    let mut file = File::open(path).map_err(|e| {
        let msg = format!("failed to open asset bundle with error: {e}");
        EngineError::FileAccessError(msg)
    })?;
    let mut assets = Vec::new();
    loop {
        // EOF is a clean end of the bundle only at a record boundary,
        // i.e. on the very first read of the next record
        let raw_type = match read_type_id(&mut file) {
            Ok(value) => value,
            Err(e) if matches!(e.kind(), ErrorKind::UnexpectedEof) => break,
            Err(e) => return Err(damaged_bundle(e)),
        };
        // any failure past this point, including EOF, means a truncated
        // or corrupted record
        let asset = read_raw_asset(&mut file, raw_type).map_err(damaged_bundle)?;
        assets.push(asset);
    }
    Ok(assets)
}

fn damaged_bundle(e: io::Error) -> EngineError {
    let msg = format!("asset bundle looks like damaged. Error: {e}");
    EngineError::ResourceParseError(msg)
}

fn read_raw_asset(file: &mut File, raw_type: TypeID) -> io::Result<RawAsset> {
    let asset_type = Type::try_from(raw_type)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "can't parse asset type"))?;

    let id = {
        let len = read_len(file)?;
        let data = read_buffer(file, len)?;
        String::from_utf8(data).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
    }?;

    let representation = {
        let repr_type = read_type_id(file)?;
        let len = read_len(file)?;
        let value = read_buffer(file, len)?;
        match repr_type {
            REPRESENTATION_BINARY => Ok(Representation::Binary { value }),
            REPRESENTATION_TEXT => {
                let value = String::from_utf8(value)
                    .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
                Ok(Representation::Text { value })
            }
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                "unexpected representation type",
            )),
        }
    }?;

    let asset = RawAsset {
        asset_type,
        id,
        representation,
    };
    Ok(asset)
}

fn read_buffer(file: &mut File, size: usize) -> io::Result<Data> {
    let mut buf = vec![0u8; size];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

fn read_type_id(file: &mut File) -> io::Result<TypeID> {
    let out = read_buffer(file, mem::size_of::<TypeID>())?;
    let mut buf = [0u8; mem::size_of::<TypeID>()];
    buf.copy_from_slice(&out);
    Ok(TypeID::from_ne_bytes(buf))
}

fn read_len(file: &mut File) -> io::Result<usize> {
    let out = read_buffer(file, mem::size_of::<usize>())?;
    let mut buf = [0u8; mem::size_of::<usize>()];
    buf.copy_from_slice(&out);
    Ok(usize::from_ne_bytes(buf))
}
