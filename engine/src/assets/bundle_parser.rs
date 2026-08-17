use std::{
    fs::File,
    io::{self, ErrorKind, Read, Seek},
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
        let result = read_raw_asset(&mut file);
        if let Err(e) = &result {
            if matches!(e.kind(), ErrorKind::UnexpectedEof) {
                break;
            }
        }
        let asset = result.map_err(|e| {
            let msg = format!("asset bundle looks like damaged. Error: {e}");
            EngineError::ResourceParseError(msg)
        })?;
        assets.push(asset);
    }
    Ok(assets)
}

fn read_raw_asset(file: &mut File) -> io::Result<RawAsset> {
    let asset_type = {
        let raw = read_type_id(file)?;
        Type::try_from(raw)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "can't parse asset type"))
    }?;

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
    // sanity check before allocating: a corrupted length field
    // must not trigger a giant allocation
    let remaining = file
        .metadata()?
        .len()
        .saturating_sub(file.stream_position()?);
    if size as u64 > remaining {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "record length exceeds remaining bundle size",
        ));
    }
    let mut buf = vec![0u8; size];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

fn read_type_id(file: &mut File) -> io::Result<TypeID> {
    let mut buf = [0u8; mem::size_of::<TypeID>()];
    file.read_exact(&mut buf)?;
    Ok(TypeID::from_le_bytes(buf))
}

// lengths are stored as fixed-width little-endian u64 so bundles
// are readable regardless of the platform they were built on
fn read_len(file: &mut File) -> io::Result<usize> {
    let mut buf = [0u8; mem::size_of::<u64>()];
    file.read_exact(&mut buf)?;
    usize::try_from(u64::from_le_bytes(buf)).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{io::Write, path::PathBuf};

    fn temp_bundle(name: &str, data: &[u8]) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("inferis_fmt_test_{}_{}", std::process::id(), name));
        let mut file = File::create(&path).expect("failed to create temp bundle");
        file.write_all(data).expect("failed to write temp bundle");
        path
    }

    #[test]
    fn little_endian_bundle_parses() {
        let mut data = Vec::new();
        data.push(3u8); // color asset
        data.extend_from_slice(&4u64.to_le_bytes());
        data.extend_from_slice(b"clr1");
        data.push(REPRESENTATION_TEXT);
        data.extend_from_slice(&5u64.to_le_bytes());
        data.extend_from_slice(b"1,2,3");
        let path = temp_bundle("le.bin", &data);
        let assets = raw_assets_from_bundle(path.to_str().unwrap()).expect("bundle must parse");
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].id, "clr1");
        _ = std::fs::remove_file(path);
    }

    #[test]
    fn oversized_length_is_rejected_without_allocation() {
        let mut data = Vec::new();
        data.push(3u8); // color asset
        data.extend_from_slice(&u64::MAX.to_le_bytes()); // absurd id length
        data.extend_from_slice(b"clr1");
        let path = temp_bundle("oversized.bin", &data);
        let result = raw_assets_from_bundle(path.to_str().unwrap());
        assert!(result.is_err(), "corrupted length must be rejected");
        _ = std::fs::remove_file(path);
    }
}
