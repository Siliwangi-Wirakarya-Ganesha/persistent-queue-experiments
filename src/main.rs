use bincode::{deserialize as de_binary, serialize as ser_binary};
use chrono::{DateTime, FixedOffset, TimeZone};
use rustbreak::{backend::FileBackend, deser::Bincode, Database, FileDatabase};
use serde::{Deserialize, Serialize};
use serde_yaml::to_string as to_yaml_string;
use std::fs::{remove_file, OpenOptions};
use std::io::{Read, Write};

const PERSISTENT_FILE: &str = "person.pq";

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

fn person_serde_exp() {
    println!("\"Person\" struct serde...");
    let person_new = Person::new(
        "Aditya Kresna",
        FixedOffset::east(9 * 3600)
            .ymd(1984, 2, 10)
            .and_hms(7, 15, 0),
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

fn get_pq_count(db: &Database<Vec<Person>, FileBackend, Bincode>) -> usize {
    db.read(|db| db.len()).unwrap()
}

fn person_pq_exp() {
    println!("\"Person\" persistent queue...");
    let person_pq =
        FileDatabase::<Vec<Person>, Bincode>::load_from_path_or_default(PERSISTENT_FILE).unwrap();
    println!("=> Current PQ count is {}", get_pq_count(&person_pq));
    println!("=> Adding 2 same person to queue...");
    person_pq
        .write_safe(|db| {
            let person_new = Person::new(
                "Aditya Kresna",
                FixedOffset::east(9 * 3600)
                    .ymd(1984, 2, 10)
                    .and_hms(7, 15, 0),
            );
            db.push(person_new.clone());
            db.push(person_new.clone());
        })
        .unwrap();
    person_pq.save().unwrap();
    println!("=> Current PQ count is {}", get_pq_count(&person_pq));
    println!("=> Removing 1 oldest person queue...");
    person_pq
        .write_safe(|db| {
            db.remove(0);
        })
        .unwrap();
    person_pq.save().unwrap();
    println!("=> Current PQ count is {}", get_pq_count(&person_pq));
}

fn main() {
    person_serde_exp();
    person_pq_exp();
}
