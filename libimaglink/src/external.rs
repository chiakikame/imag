/// External linking is a complex implementation to be able to serve a clean and easy-to-use
/// interface.
///
/// Internally, there are no such things as "external links" (plural). Each Entry in the store can
/// only have _one_ external link.
///
/// This library does the following therefor: It allows you to have several external links with one
/// entry, which are internally one file in the store for each link, linked with "internal
/// linking".
///
/// This helps us greatly with deduplication of URLs.
///

use std::convert::Into;
use std::ops::Deref;
use std::ops::DerefMut;
use std::collections::BTreeMap;

use libimagstore::store::Entry;
use libimagstore::store::EntryHeader;
use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagstore::storeid::StoreId;
use libimagstore::storeid::IntoStoreId;

use error::LinkError as LE;
use error::LinkErrorKind as LEK;
use result::Result;
use internal::InternalLinker;
use module_path::ModuleEntryPath;

use toml::Value;
use toml::Table;
use url::Url;
use crypto::sha1::Sha1;
use crypto::digest::Digest;

/// "Link" Type, just an abstraction over FileLockEntry to have some convenience internally.
struct Link<'a> {
    link: FileLockEntry<'a>
}

impl<'a> Link<'a> {

    pub fn new(fle: FileLockEntry<'a>) -> Link<'a> {
        Link { link: fle }
    }

    /// For interal use only. Load an Link from a store id, if this is actually a Link
    fn retrieve(store: &'a Store, id: StoreId) -> Result<Option<Link<'a>>> {
        store.retrieve(id)
            .map(|fle| {
                if let Some(_) = Link::get_link_uri_from_filelockentry(&fle) {
                    Some(Link {
                        link: fle
                    })
                } else {
                    None
                }
            })
            .map_err(|e| LE::new(LEK::StoreReadError, Some(Box::new(e))))
    }

    /// Get a link Url object from a FileLockEntry, ignore errors.
    fn get_link_uri_from_filelockentry(file: &FileLockEntry<'a>) -> Option<Url> {
        file.deref()
            .get_header()
            .read("imag.content.uri")
            .ok()
            .and_then(|opt| {
                match opt {
                    Some(Value::String(s)) => Url::parse(&s[..]).ok(),
                    _ => None
                }
            })
    }

    pub fn get_url(&self) -> Result<Option<Url>> {
        let opt = self.link
            .deref()
            .get_header()
            .read("imag.content.uri");

        match opt {
            Ok(Some(Value::String(s))) => {
                Url::parse(&s[..])
                     .map(|s| Some(s))
                     .map_err(|e| LE::new(LEK::EntryHeaderReadError, Some(Box::new(e))))
            },
            Ok(None) => Ok(None),
            _ => Err(LE::new(LEK::EntryHeaderReadError, None))
        }
    }

}

pub trait ExternalLinker : InternalLinker {

    /// Get the external links from the implementor object
    fn get_external_links(&self, store: &Store) -> Result<Vec<Url>>;

    /// Set the external links for the implementor object
    fn set_external_links(&mut self, store: &Store, links: Vec<Url>) -> Result<()>;

    /// Add an external link to the implementor object
    fn add_external_link(&mut self, store: &Store, link: Url) -> Result<()>;

    /// Remove an external link from the implementor object
    fn remove_external_link(&mut self, store: &Store, link: Url) -> Result<()>;

}

/// Check whether the StoreId starts with `/link/external/`
fn is_link_store_id(id: &StoreId) -> bool {
    debug!("Checking whether this is a /link/external/*: '{:?}'", id);
    id.starts_with("/link/external/")
}

fn get_external_link_from_file(entry: &FileLockEntry) -> Result<Url> {
    Link::get_link_uri_from_filelockentry(entry) // TODO: Do not hide error by using this function
        .ok_or(LE::new(LEK::StoreReadError, None))
}

/// Implement ExternalLinker for Entry, hiding the fact that there is no such thing as an external
/// link in an entry, but internal links to other entries which serve as external links, as one
/// entry in the store can only have one external link.
impl ExternalLinker for Entry {

    /// Get the external links from the implementor object
    fn get_external_links(&self, store: &Store) -> Result<Vec<Url>> {
        // Iterate through all internal links and filter for FileLockEntries which live in
        // /link/external/<SHA> -> load these files and get the external link from their headers,
        // put them into the return vector.
        self.get_internal_links()
            .map(|vect| {
                debug!("Getting external links");
                vect.into_iter()
                    .filter(is_link_store_id)
                    .map(|id| {
                        debug!("Retrieving entry for id: '{:?}'", id);
                        match store.retrieve(id.clone()) {
                            Ok(f) => get_external_link_from_file(&f),
                            Err(e) => {
                                debug!("Retrieving entry for id: '{:?}' failed", id);
                                Err(LE::new(LEK::StoreReadError, Some(Box::new(e))))
                            }
                        }
                    })
                    .filter_map(|x| x.ok()) // TODO: Do not ignore error here
                    .collect()
            })
            .map_err(|e| LE::new(LEK::StoreReadError, Some(Box::new(e))))
    }

    /// Set the external links for the implementor object
    fn set_external_links(&mut self, store: &Store, links: Vec<Url>) -> Result<()> {
        // Take all the links, generate a SHA sum out of each one, filter out the already existing
        // store entries and store the other URIs in the header of one FileLockEntry each, in
        // the path /link/external/<SHA of the URL>

        debug!("Iterating {} links = {:?}", links.len(), links);
        for link in links { // for all links
            let hash = {
                let mut s = Sha1::new();
                s.input_str(&link.serialize()[..]);
                s.result_str()
            };
            let file_id = ModuleEntryPath::new(format!("external/{}", hash)).into_storeid();

            debug!("Link    = '{:?}'", link);
            debug!("Hash    = '{:?}'", hash);
            debug!("StoreId = '{:?}'", file_id);

            let mut file = {
                if let Ok(mut file) = store.retrieve(file_id.clone()) { // retrieve the file from the store
                    debug!("Woha, there is already a file for this link: '{:?}'", link);
                    file
                } else { // or
                    debug!("Generating file for link = '{:?}' on id = {:?}", link, file_id);
                    let res = store.create(file_id) // create it
                        .and_then(|mut file| {
                            {
                                debug!("Generating header content!");
                                let mut hdr = file.deref_mut().get_header_mut();

                                // Write the URI into the header
                                match hdr.set("imag.content", Value::Table(BTreeMap::new())) {
                                    Ok(_) => {
                                        let v = Value::String(link.serialize());
                                        debug!("setting URL = '{:?}", v);
                                        hdr.set("imag.content.uri", v);
                                        Ok(())
                                    },
                                    Err(e) => {
                                        debug!("Failed to generate a table in header at 'imag.content'");
                                        Err(e)
                                    },
                                }
                            }.map(|_| file)
                        })
                        .map_err(|e| LE::new(LEK::StoreWriteError, Some(Box::new(e))));

                    // And if that fails we can error
                    if let Err(e) = res {
                        debug!("Failed to create or retrieve an file for this link '{:?}'", link);
                        return Err(e);
                    }
                    debug!("Success creating or retrieving an file for this link '{:?}'", link);
                    res.unwrap()
                }
            };

            // then add an internal link to the new file or return an error if this fails
            if let Err(e) = self.add_internal_link(file.deref_mut()) {
                debug!("Error adding internal link");
                return Err(LE::new(LEK::StoreWriteError, Some(Box::new(e))));
            }
        }
        debug!("Ready iterating");
        Ok(())
    }

    /// Add an external link to the implementor object
    fn add_external_link(&mut self, store: &Store, link: Url) -> Result<()> {
        // get external links, add this one, save them
        debug!("Getting links");
        self.get_external_links(store)
            .and_then(|mut links| {
                debug!("Adding link = '{:?}' to links = {:?}", link, links);
                links.push(link);
                debug!("Setting {} links = {:?}", links.len(), links);
                self.set_external_links(store, links)
            })
    }

    /// Remove an external link from the implementor object
    fn remove_external_link(&mut self, store: &Store, link: Url) -> Result<()> {
        // get external links, remove this one, save them
        self.get_external_links(store)
            .and_then(|mut links| {
                debug!("Removing link = '{:?}' from links = {:?}", link, links);
                let links = links.into_iter()
                    .filter(|l| l.serialize() != link.serialize())
                    .collect();
                self.set_external_links(store, links)
            })
    }

}

