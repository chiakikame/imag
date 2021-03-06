//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015, 2016 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

use std::fs::{File, OpenOptions, create_dir_all, remove_file, copy, rename};
use std::io::{Seek, SeekFrom, Read};
use std::path::{Path, PathBuf};

use error::{StoreError as SE, StoreErrorKind as SEK};
use error::ResultExt;

use super::FileAbstraction;
use super::FileAbstractionInstance;
use super::Drain;
use store::Entry;
use storeid::StoreId;
use file_abstraction::iter::PathIterator;

#[derive(Debug)]
pub enum FSFileAbstractionInstance {
    Absent(PathBuf),
    File(File, PathBuf)
}

impl FileAbstractionInstance for FSFileAbstractionInstance {

    /**
     * Get the content behind this file
     */
    fn get_file_content(&mut self, id: StoreId) -> Result<Entry, SE> {
        debug!("Getting lazy file: {:?}", self);
        let (file, path) = match *self {
            FSFileAbstractionInstance::File(ref mut f, _) => return {
                // We seek to the beginning of the file since we expect each
                // access to the file to be in a different context
                try!(f.seek(SeekFrom::Start(0))
                    .chain_err(|| SEK::FileNotSeeked));

                let mut s = String::new();
                f.read_to_string(&mut s)
                    .chain_err(|| SEK::IoError)
                    .map(|_| s)
                    .and_then(|s| Entry::from_str(id, &s))
            },
            FSFileAbstractionInstance::Absent(ref p) =>
                (try!(open_file(p).chain_err(|| SEK::FileNotFound)), p.clone()),
        };
        *self = FSFileAbstractionInstance::File(file, path);
        if let FSFileAbstractionInstance::File(ref mut f, _) = *self {
            let mut s = String::new();
            f.read_to_string(&mut s)
                .chain_err(|| SEK::IoError)
                .map(|_| s)
                .and_then(|s| Entry::from_str(id, &s))
        } else {
            unreachable!()
        }
    }

    /**
     * Write the content of this file
     */
    fn write_file_content(&mut self, buf: &Entry) -> Result<(), SE> {
        use std::io::Write;

        let buf = buf.to_str().into_bytes();

        let (file, path) = match *self {
            FSFileAbstractionInstance::File(ref mut f, _) => return {
                // We seek to the beginning of the file since we expect each
                // access to the file to be in a different context
                try!(f.seek(SeekFrom::Start(0))
                    .chain_err(|| SEK::FileNotCreated));

                try!(f.set_len(buf.len() as u64).chain_err(|| SEK::FileNotWritten));

                f.write_all(&buf).chain_err(|| SEK::FileNotWritten)
            },
            FSFileAbstractionInstance::Absent(ref p) =>
                (try!(create_file(p).chain_err(|| SEK::FileNotCreated)), p.clone()),
        };
        *self = FSFileAbstractionInstance::File(file, path);
        if let FSFileAbstractionInstance::File(ref mut f, _) = *self {
            return f.write_all(&buf).chain_err(|| SEK::FileNotWritten);
        }
        unreachable!();
    }
}

/// `FSFileAbstraction` state type
///
/// A lazy file is either absent, but a path to it is available, or it is present.
#[derive(Debug)]
pub struct FSFileAbstraction {
}

impl FSFileAbstraction {
    pub fn new() -> FSFileAbstraction {
        FSFileAbstraction { }
    }
}

impl FileAbstraction for FSFileAbstraction {

    fn remove_file(&self, path: &PathBuf) -> Result<(), SE> {
        remove_file(path).chain_err(|| SEK::FileNotRemoved)
    }

    fn copy(&self, from: &PathBuf, to: &PathBuf) -> Result<(), SE> {
        copy(from, to).chain_err(|| SEK::FileNotCopied).map(|_| ())
    }

    fn rename(&self, from: &PathBuf, to: &PathBuf) -> Result<(), SE> {
        rename(from, to).chain_err(|| SEK::FileNotRenamed)
    }

    fn create_dir_all(&self, path: &PathBuf) -> Result<(), SE> {
        create_dir_all(path).chain_err(|| SEK::DirNotCreated)
    }

    fn new_instance(&self, p: PathBuf) -> Box<FileAbstractionInstance> {
        Box::new(FSFileAbstractionInstance::Absent(p))
    }

    /// We return nothing from the FS here.
    fn drain(&self) -> Result<Drain, SE> {
        Ok(Drain::empty())
    }

    /// FileAbstraction::fill implementation that consumes the Drain and writes everything to the
    /// filesystem
    fn fill(&mut self, mut d: Drain) -> Result<(), SE> {
        d.iter()
            .fold(Ok(()), |acc, (path, element)| {
                acc.and_then(|_| self.new_instance(path).write_file_content(&element))
            })
    }

    fn pathes_recursively(&self, basepath: PathBuf) -> Result<PathIterator, SE> {
        use walkdir::WalkDir;

        let i : Result<Vec<PathBuf>, SE> = WalkDir::new(basepath)
            .into_iter()
            .map(|r| {
                r.map(|e| PathBuf::from(e.path()))
                    .chain_err(|| SE::from_kind(SEK::FileError))
            })
            .fold(Ok(vec![]), |acc, e| {
                acc.and_then(move |mut a| {
                    a.push(try!(e));
                    Ok(a)
                })
            });
        Ok(PathIterator::new(Box::new(try!(i).into_iter())))
    }
}

fn open_file<A: AsRef<Path>>(p: A) -> ::std::io::Result<File> {
    OpenOptions::new().write(true).read(true).open(p)
}

fn create_file<A: AsRef<Path>>(p: A) -> ::std::io::Result<File> {
    if let Some(parent) = p.as_ref().parent() {
        debug!("Implicitely creating directory: {:?}", parent);
        if let Err(e) = create_dir_all(parent) {
            return Err(e);
        }
    }
    OpenOptions::new().write(true).read(true).create(true).open(p)
}

