use std::{fs, io::Write, path::PathBuf};

use engine::assets::raw_assets_from_bundle;

const TYPE_ID_COLOR: u8 = 3;
const REPRESENTATION_TEXT: u8 = 1;

fn write_record(out: &mut Vec<u8>, asset_type: u8, id: &str, repr: u8, value: &[u8]) {
    out.push(asset_type);
    out.extend_from_slice(&id.len().to_ne_bytes());
    out.extend_from_slice(id.as_bytes());
    out.push(repr);
    out.extend_from_slice(&value.len().to_ne_bytes());
    out.extend_from_slice(value);
}

fn temp_bundle(name: &str, data: &[u8]) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("inferis_test_{}_{}", std::process::id(), name));
    let mut file = fs::File::create(&path).expect("failed to create temp bundle");
    file.write_all(data).expect("failed to write temp bundle");
    path
}

#[test]
fn empty_bundle_is_valid() {
    let path = temp_bundle("empty.bin", &[]);
    let assets = raw_assets_from_bundle(path.to_str().unwrap()).expect("empty bundle must parse");
    assert!(assets.is_empty());
    _ = fs::remove_file(path);
}

#[test]
fn complete_bundle_parses() {
    let mut data = Vec::new();
    write_record(&mut data, TYPE_ID_COLOR, "clr1", REPRESENTATION_TEXT, b"1,2,3");
    write_record(&mut data, TYPE_ID_COLOR, "clr2", REPRESENTATION_TEXT, b"4,5,6");
    let path = temp_bundle("complete.bin", &data);
    let assets = raw_assets_from_bundle(path.to_str().unwrap()).expect("bundle must parse");
    assert_eq!(assets.len(), 2);
    _ = fs::remove_file(path);
}

#[test]
fn truncated_record_is_rejected() {
    let mut data = Vec::new();
    write_record(&mut data, TYPE_ID_COLOR, "clr1", REPRESENTATION_TEXT, b"1,2,3");
    write_record(&mut data, TYPE_ID_COLOR, "clr2", REPRESENTATION_TEXT, b"4,5,6");
    // cut the last record in half
    data.truncate(data.len() - 3);
    let path = temp_bundle("truncated.bin", &data);
    let result = raw_assets_from_bundle(path.to_str().unwrap());
    assert!(result.is_err(), "truncated bundle must be rejected");
    _ = fs::remove_file(path);
}

#[test]
fn record_cut_at_length_field_is_rejected() {
    let mut data = Vec::new();
    write_record(&mut data, TYPE_ID_COLOR, "clr1", REPRESENTATION_TEXT, b"1,2,3");
    // a new record starts (type id present) but EOF hits inside the id length
    data.push(TYPE_ID_COLOR);
    data.extend_from_slice(&[0u8; 3]);
    let path = temp_bundle("cut_len.bin", &data);
    let result = raw_assets_from_bundle(path.to_str().unwrap());
    assert!(result.is_err(), "partially written record must be rejected");
    _ = fs::remove_file(path);
}
