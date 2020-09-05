mod impl_queue_file;
mod impl_rustbreak;

use crate::impl_queue_file::QFQueue;
use crate::impl_rustbreak::RBQueue;
use bincode::{deserialize as de_binary, serialize as ser_binary};
use chrono::{DateTime, FixedOffset, TimeZone};
use core::convert::AsRef;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_yaml::to_string as to_yaml_string;
use std::fmt::Display;
use std::fs::{remove_file, OpenOptions};
use std::io::{Read, Result as IOResult, Write};
use std::path::Path;
use std::string::ToString;

const RBQ_FILE: &str = "person.rbq";
const QFQ_FILE: &str = "person.qfq";

pub trait IPersistentQueue<T, K = Self>
where
    T: DeserializeOwned + Serialize + Clone + Send,
    Self: Sized,
{
    fn new<S: AsRef<Path> + ToString + Display + Clone>(path: S) -> IOResult<Self>;
    fn enqueue(&mut self, data: T) -> IOResult<()>;
    fn dequeue(&mut self) -> IOResult<Option<T>>;
    fn count(&self) -> IOResult<usize>;
    fn get_filename(&self) -> String;
}

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

fn person_exp<S>(pq: &mut S)
where
    S: IPersistentQueue<Person>,
{
    println!(
        "\"Person\" persistent queue with filename: {}",
        pq.get_filename()
    );
    println!("=> Current PQ count is {}", pq.count().unwrap());
    println!("=> Adding 2 same person to queue...");
    let person_new = Person::new(
        "Aditya Kresna",
        FixedOffset::east(9 * 3600)
            .ymd(1984, 2, 10)
            .and_hms(7, 15, 0),
    );
    pq.enqueue(person_new.clone()).unwrap();
    pq.enqueue(person_new.clone()).unwrap();
    println!("=> Current PQ count is {}", pq.count().unwrap());
    println!("=> Removing 1 oldest person queue...");
    pq.dequeue().unwrap();
    println!("=> Current PQ count is {}", pq.count().unwrap());
}

fn main() {
    person_serde_exp();
    let mut person_rbq = RBQueue::<Person>::new(RBQ_FILE).unwrap();
    let mut person_qfq = QFQueue::<Person>::new(QFQ_FILE).unwrap();
    person_exp(&mut person_rbq);
    person_exp(&mut person_qfq);
}
