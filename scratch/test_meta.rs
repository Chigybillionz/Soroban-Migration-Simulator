use stellar_xdr::curr::{ScMetaEntry, ScEnvMetaEntry, ReadXdr, Limits, Limited};
use wasmparser::{Parser, Payload};
use std::fs;

fn main() {
    let wasm = fs::read("c:/Users/User/soroban-migration-simulator/target/wasm32v1-none/release/migration_v1.wasm").unwrap();
    let mut has_meta = false;
    let mut has_env = false;

    for payload in Parser::new(0).parse_all(&wasm) {
        let payload = payload.unwrap();
        if let Payload::CustomSection(section) = payload {
            if section.name() == "contractmetav0" {
                has_meta = true;
                let data = section.data();
                let mut cursor = std::io::Cursor::new(data);
                let limits = Limits::none();
                let mut limited = Limited::new(&mut cursor, limits);
                while (limited.inner.position() as usize) < data.len() {
                    let entry = ScMetaEntry::read_xdr(&mut limited).unwrap();
                    println!("Meta: {:?}", entry);
                }
            } else if section.name() == "contractenvmetav0" {
                has_env = true;
                let data = section.data();
                let mut cursor = std::io::Cursor::new(data);
                let limits = Limits::none();
                let mut limited = Limited::new(&mut cursor, limits);
                while (limited.inner.position() as usize) < data.len() {
                    let entry = ScEnvMetaEntry::read_xdr(&mut limited).unwrap();
                    println!("EnvMeta: {:?}", entry);
                }
            }
        }
    }
    println!("Meta: {}, Env: {}", has_meta, has_env);
}
