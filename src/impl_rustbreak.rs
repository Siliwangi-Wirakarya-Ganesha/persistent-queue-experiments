use crate::IPersistentQueue;
use core::convert::AsRef;
use rustbreak::{backend::FileBackend, deser::Bincode, Database, FileDatabase};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Display;
use std::io::{Error as IOError, ErrorKind as IOErrorKind, Result as IOResult};
use std::path::Path;
use std::string::ToString;

pub struct RBQueue<T>
where
    T: DeserializeOwned + Serialize + Clone + Send,
{
    pbq: Database<Vec<T>, FileBackend, Bincode>,
    file_name: String,
}

impl<T> RBQueue<T>
where
    T: DeserializeOwned + Serialize + Clone + Send,
{
    fn write(&self, data: Option<T>) -> IOResult<Option<T>> {
        match self.pbq.write(|inner| match data {
            Some(data_to_push) => {
                inner.push(data_to_push);
                None
            }
            None => {
                if inner.is_empty() {
                    return None;
                }

                Some(inner.remove(0))
            }
        }) {
            Err(error_result) => Err(IOError::new(
                IOErrorKind::Other,
                format!("{:?}", error_result),
            )),
            Ok(ok_result) => Ok(ok_result),
        }
    }

    fn save(&self) -> IOResult<()> {
        if let Err(error_result) = self.pbq.save() {
            return Err(IOError::new(
                IOErrorKind::Other,
                format!("{:?}", error_result),
            ));
        }

        Ok(())
    }
}

impl<T> IPersistentQueue<T> for RBQueue<T>
where
    T: DeserializeOwned + Serialize + Clone + Send,
{
    fn new<S: AsRef<Path> + ToString + Display + Clone>(path: S) -> IOResult<Self> {
        let cloned_path = path.clone();
        let pbq = FileDatabase::<Vec<T>, Bincode>::load_from_path_or_default(path);

        if pbq.is_err() {
            return Err(IOError::new(
                IOErrorKind::PermissionDenied,
                "Unable to load or create RBQueue!",
            ));
        }

        let pbq = pbq.unwrap();

        Ok(Self {
            pbq,
            file_name: cloned_path.to_string(),
        })
    }

    fn enqueue(&mut self, data: T) -> IOResult<()> {
        self.write(Some(data))?;

        self.save()
    }

    fn dequeue(&mut self) -> IOResult<Option<T>> {
        let result = self.write(None);
        self.save()?;

        result
    }

    fn count(&self) -> IOResult<usize> {
        let result = self.pbq.read(|inner| inner.len());

        if let Err(error_result) = result {
            return Err(IOError::new(
                IOErrorKind::Other,
                format!("{:?}", error_result),
            ));
        }

        Ok(result.unwrap())
    }

    fn get_filename(&self) -> String {
        self.file_name.clone()
    }
}
