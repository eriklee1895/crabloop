use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
};

pub struct Journal(File);
impl Journal {
    pub fn create(path: &Path) -> io::Result<Self> {
        Ok(Self(OpenOptions::new().write(true).create_new(true).open(path)?))
    }
    pub fn record(&mut self, kind: &str, data: &Value) -> io::Result<()> {
        writeln!(self.0, "{}", json!({"kind":kind,"data":data}))?;
        self.0.flush()
    }
}
