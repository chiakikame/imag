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

use std::io::Read;
use std::path::PathBuf;
use std::result::Result as RResult;

use crypto::sha1::Sha1;
use crypto::digest::Digest;

use hasher::Hasher;
use error::Result;
use error::RefErrorKind as REK;
use error::ResultExt;

pub struct NBytesHasher {
    hasher: Sha1,
    n: usize,
}

impl NBytesHasher {

    pub fn new(n: usize) -> NBytesHasher {
        NBytesHasher {
            hasher: Sha1::new(),
            n: n,
        }
    }

}

impl Hasher for NBytesHasher {

    fn hash_name(&self) -> &'static str {
        "n-bytes-hasher"
    }

    fn create_hash<R: Read>(&mut self, _: &PathBuf, contents: &mut R) -> Result<String> {
        let s = try!(contents
            .bytes()
            .take(self.n)
            .collect::<RResult<Vec<u8>, _>>()
            .chain_err(|| REK::IOError)
            .and_then(|v| String::from_utf8(v).chain_err(|| REK::UTF8Error)));
        self.hasher.input_str(&s[..]);
        Ok(self.hasher.result_str())
    }

}

