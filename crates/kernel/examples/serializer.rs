use kernel::serializer::System;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Data {
    name: String,
    value: i32,
}

fn main() {
    let system = System::new();
    let data = Data { name: "test".to_string(), value: 42 };
    // JSON
    let json = system.json(&data).unwrap();
    let parsed: Data = system.parse(&json).unwrap();
    println!("JSON roundtrip: {:?} == {:?} => {}", data, parsed, data == parsed);
    // Bincode
    let bin = system.encode(&data).unwrap();
    let decoded: Data = system.decode(&bin).unwrap();
    println!("Bincode roundtrip: {:?} == {:?} => {}", data, decoded, data == decoded);
} 