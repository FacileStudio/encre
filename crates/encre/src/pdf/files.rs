use super::Assets;
use include_dir::{Dir, include_dir};
use std::sync::OnceLock;
use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::FileId;
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};

static MUSE_TYPST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/typst/muse-typst");
static FONTS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/typst/fonts");

/// Where muse-typst lives inside the in-memory world.
const MUSE_ROOT: &str = "/muse-typst/";

/// The standard library and the embedded fonts, loaded once.
pub struct Shared {
    pub library: LazyHash<Library>,
    pub book: LazyHash<FontBook>,
    pub fonts: Vec<Font>,
}

/// The process-wide [`Shared`].
pub fn shared() -> &'static Shared {
    static SHARED: OnceLock<Shared> = OnceLock::new();
    SHARED.get_or_init(|| {
        let fonts: Vec<Font> = FONTS
            .files()
            .filter(|file| file.path().extension().is_some_and(|ext| ext == "ttf"))
            .flat_map(|file| Font::iter(Bytes::new(file.contents())))
            .collect();
        Shared { library: LazyHash::new(Library::default()), book: LazyHash::new(FontBook::from_fonts(&fonts)), fonts }
    })
}

/// The bytes of `id`: muse-typst under `/muse-typst/`, otherwise the caller's assets. Packages never resolve.
pub fn lookup(assets: &Assets, id: FileId) -> FileResult<&[u8]> {
    let path = id.vpath().as_rootless_path().to_string_lossy().replace('\\', "/");
    let absolute = format!("/{path}");
    let found = match (id.package(), absolute.strip_prefix(MUSE_ROOT)) {
        (Some(_), _) => None,
        (None, Some(inner)) => MUSE_TYPST.get_file(inner).map(|file| file.contents()),
        (None, None) => assets.get(&absolute),
    };
    found.ok_or_else(|| FileError::NotFound(absolute.into()))
}
