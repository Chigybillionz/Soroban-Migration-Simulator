use stellar_xdr::curr::{ScSpecEntry, ReadXdr, Limits, Limited};
use wasmparser::{Parser, Payload};
use std::fs;

fn main() {
    let wasm = fs::read("c:/Users/User/soroban-migration-simulator/target/wasm32v1-none/release/migration_v1.wasm").unwrap();
    for payload in Parser::new(0).parse_all(&wasm) {
        let payload = payload.unwrap();
        if let Payload::CustomSection(section) = payload {
            if section.name() == "contractspecv0" {
                let data = section.data();
                let mut cursor = std::io::Cursor::new(data);
                let limits = Limits::none();
                let mut limited = Limited::new(&mut cursor, limits);
                while (limited.inner.position() as usize) < data.len() {
                    let entry = ScSpecEntry::read_xdr(&mut limited).unwrap();
                    match entry {
                        ScSpecEntry::UdtStructV0(s) => {
                            println!("Struct: {}", s.name.to_utf8_string_lossy());
                        },
                        ScSpecEntry::UdtUnionV0(u) => {
                            println!("Union: {}", u.name.to_utf8_string_lossy());
                        },
                        ScSpecEntry::UdtEnumV0(e) => {
                            println!("Enum: {}", e.name.to_utf8_string_lossy());
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}
