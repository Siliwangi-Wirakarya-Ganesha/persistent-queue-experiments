use bincode::{deserialize as de_binary, serialize as ser_binary};
use chrono::{DateTime, FixedOffset, TimeZone};
use serde::{Deserialize, Serialize};
use serde_yaml::to_string as to_yaml_string;
use std::fs::{remove_file, OpenOptions};
use std::io::{Read, Write};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Person {
    pub name: String,
    pub birthdate: DateTime<FixedOffset>,
}

impl Person {
    pub fn new(name: &str, birthdate: DateTime<FixedOffset>) -> Self {
        Self {
            name: name.into(),
            birthdate,
        }
    }
}

fn main() {
    let person_new = Person::new(
        "Aditya Kresna",
        FixedOffset::east(9 * 3600)
            .ymd(1984, 2, 10)
            .and_hms(5, 15, 0),
    );
    let person_bin = ser_binary(&person_new).unwrap();
    OpenOptions::new()
        .write(true)
        .create(true)
        .open("person.bin")
        .unwrap()
        .write(&person_bin[..])
        .unwrap();
    let mut person_bin_loaded = Vec::new();
    OpenOptions::new()
        .read(true)
        .open("person.bin")
        .unwrap()
        .read_to_end(&mut person_bin_loaded)
        .unwrap();
    let person_bin_loaded = person_bin_loaded;
    let person_de = de_binary::<Person>(&person_bin_loaded[..]).unwrap();
    let person_yaml = to_yaml_string(&person_de).unwrap();
    println!("{}", person_yaml);
    remove_file("person.bin").unwrap();
}
