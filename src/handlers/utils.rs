use std::ffi::OsStr;
use std::path::Path;

/// Returns the file stem and file extension parts of `file_name`. This internally uses
/// [`std::path::Path::file_stem`] and [`std::path::Path::extension`]. See their document for how
/// each item would be extracted.
///
/// # Examples
/// ```ignore
/// // This API is supposed to be accessed by the modules defined under module `crate::handlers`
///
/// use crate::handlers::utils::split_file_name;
/// use std::ffi::OsStr;
///
/// assert_eq!(split_file_name("/tmp/foo.rs"), (Some(OsStr::new("foo")), Some(OsStr::new("rs"))));
/// assert_eq!(split_file_name("/.tmp123ABC"), (Some(OsStr::new(".tmp123ABC")), None));
/// assert_eq!(split_file_name("/tmp/no_ext"), (Some(OsStr::new("no_ext")), None));
/// assert_eq!(split_file_name("secret.tar.gz"), (Some(OsStr::new("secret.tar")), Some(OsStr::new("gz"))));
/// ```
pub fn split_file_name<P: AsRef<Path> + ?Sized>(file_name: &P) -> (Option<&OsStr>, Option<&OsStr>) {
    let path = file_name.as_ref();
    (path.file_stem(), path.extension())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_file_name() {
        assert_eq!(
            split_file_name("/tmp/foo.rs"),
            (Some(OsStr::new("foo")), Some(OsStr::new("rs")))
        );
        assert_eq!(
            split_file_name("./.tmp123ABC"),
            (Some(OsStr::new(".tmp123ABC")), None)
        );
        assert_eq!(
            split_file_name("/tmp/no_ext"),
            (Some(OsStr::new("no_ext")), None)
        );
        assert_eq!(
            split_file_name("secret.tar.gz"),
            (Some(OsStr::new("secret.tar")), Some(OsStr::new("gz")))
        );
    }
}
