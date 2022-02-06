// Copyright (c) 2021, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use log::Level;
use std::fmt::Display;
use std::fmt::Formatter;

pub trait Backend
{
    type Error: Display;
    fn write(&mut self, target: &str, msg: &str, level: Level) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
}

pub struct DummyError();

impl Display for DummyError
{
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        todo!() // Panic (DummyError is by definition the error that never occurs)!
    }
}

pub struct StdBackend
{
    smart_stderr: bool
}

impl StdBackend
{
    pub fn new(smart_stderr: bool) -> StdBackend
    {
        StdBackend {
            smart_stderr
        }
    }
}

impl Backend for StdBackend
{
    type Error = DummyError;

    fn write(&mut self, target: &str, msg: &str, level: Level) -> Result<(), Self::Error>
    {
        if !self.smart_stderr {
            println!("<{}> [{}] {}", target, level, msg);
            return Ok(());
        }
        if level == Level::Error {
            eprintln!("<{}> [{}] {}", target, level, msg);
        } else {
            println!("<{}> [{}] {}", target, level, msg);
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error>
    {
        Ok(())
    }
}

pub struct FileBackend
{
    targets: HashMap<String, BufWriter<File>>,
    path: PathBuf
}

impl FileBackend
{
    pub fn new(path: PathBuf) -> FileBackend
    {
        FileBackend {
            targets: HashMap::new(),
            path
        }
    }

    fn get_create_open_file(&mut self, target: &str) -> Result<&mut BufWriter<File>, std::io::Error>
    {
        if let None = self.targets.get_mut(target) {
            let f = OpenOptions::new()
                .append(true)
                .create(true)
                .open(self.path.join(format!("{}.log", target)))?;
            self.targets.insert(target.into(), BufWriter::new(f));
        }
        unsafe { // This cannot never fail because None is captured and initialized by the if block.
            Ok(self.targets.get_mut(target).unwrap_unchecked())
        }
    }
}

impl Backend for FileBackend
{
    type Error = std::io::Error;

    fn write(&mut self, target: &str, msg: &str, level: Level) -> Result<(), Self::Error> {
        let log = format!("[{}] {}", level, msg);
        self.get_create_open_file(target)?.write_all(log.as_bytes())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        for (_, v) in &mut self.targets {
            v.flush()?;
        }
        Ok(())
    }
}
