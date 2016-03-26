use std::io::stdout;
use std::io::Write;
use std::ops::Deref;

use lister::Lister;
use result::Result;

use libimagstore::store::FileLockEntry;
use libimagstore::store::Entry;

pub struct LineLister<'a> {
    lister: &'a Fn(&Entry) -> String,
}

impl<'a> LineLister<'a> {

    pub fn new(lister: &'a Fn(&Entry) -> String) -> LineLister<'a> {
        LineLister {
            lister: lister,
        }
    }

}

impl<'a> Lister for LineLister<'a> {

    fn list<'b, I: Iterator<Item = FileLockEntry<'b>>>(&self, entries: I) -> Result<()> {
        use error::ListError as LE;
        use error::ListErrorKind as LEK;

        entries.fold(Ok(()), |accu, entry| {
            accu.and_then(|_| {
                    write!(stdout(), "{:?}\n", (self.lister)(entry.deref()))
                        .map_err(|e| LE::new(LEK::FormatError, Some(Box::new(e))))
                })
            })
    }

}
