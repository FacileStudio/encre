use super::Assets;
use super::files::{lookup, shared};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

/// A Typst world held entirely in memory.
pub struct MemoryWorld<'a> {
    main: Source,
    assets: &'a Assets,
}

impl<'a> MemoryWorld<'a> {
    /// A world whose main file is `source`.
    pub fn new(source: &str, assets: &'a Assets) -> Self {
        let id = FileId::new(None, VirtualPath::new("/main.typ"));
        MemoryWorld { main: Source::new(id, source.to_owned()), assets }
    }
}

impl World for MemoryWorld<'_> {
    fn library(&self) -> &LazyHash<Library> {
        &shared().library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &shared().book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            return Ok(self.main.clone());
        }
        let text = std::str::from_utf8(lookup(self.assets, id)?).map_err(|_| FileError::InvalidUtf8)?;
        Ok(Source::new(id, text.to_owned()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        lookup(self.assets, id).map(|bytes| Bytes::new(bytes.to_vec()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        shared().fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
}
