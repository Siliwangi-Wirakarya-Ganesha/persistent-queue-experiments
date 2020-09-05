use crate::IPersistentQueue;
use bincode::{deserialize as de_binary, serialize as ser_binary};
use core::convert::AsRef;
use queue_file::QueueFile;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Display;
use std::io::{Error as IOError, ErrorKind as IOErrorKind, Result as IOResult};
use std::marker::PhantomData;
use std::path::Path;
use std::string::ToString;

pub struct QFQueue<T>
where
    T: DeserializeOwned + Serialize + Clone + Send,
{
    qfq: QueueFile,
    file_name: String,
    marker: PhantomData<T>,
}

impl<T> IPersistentQueue<T> for QFQueue<T>
where
    T: DeserializeOwned + Serialize + Clone + Send,
{
    fn new<S: AsRef<Path> + ToString + Display + Clone>(path: S) -> IOResult<Self> {
        let cloned_path = path.clone();

        match QueueFile::open(path) {
            Err(error_result) => Err(IOError::new(IOErrorKind::Other, error_result.to_string())),
            Ok(qfq) => Ok(Self {
                qfq,
                file_name: cloned_path.to_string(),
                marker: Default::default(),
            }),
        }
    }

    fn enqueue(&mut self, data: T) -> IOResult<()> {
        match ser_binary(&data) {
            Err(error_result) => Err(IOError::new(IOErrorKind::Other, error_result.to_string())),
            Ok(data_bin) => match self.qfq.add(&data_bin[..]) {
                Err(error_result) => {
                    Err(IOError::new(IOErrorKind::Other, error_result.to_string()))
                }
                Ok(_) => Ok(()),
            },
        }
    }

    fn dequeue(&mut self) -> IOResult<Option<T>> {
        match self.qfq.iter().next() {
            None => Ok(None),
            Some(data_bin) => match de_binary(&data_bin) {
                Err(error_result) => {
                    Err(IOError::new(IOErrorKind::Other, error_result.to_string()))
                }
                Ok(data) => {
                    if let Err(error_result) = self.qfq.remove() {
                        return Err(IOError::new(IOErrorKind::Other, error_result.to_string()));
                    }

                    Ok(Some(data))
                }
            },
        }
    }

    fn count(&self) -> IOResult<usize> {
        Ok(self.qfq.size())
    }

    fn get_filename(&self) -> String {
        self.file_name.clone()
    }
}
