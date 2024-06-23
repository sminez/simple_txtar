//! A simple implementation of the go [txtar](https://github.com/golang/tools/blob/master/txtar/archive.go)
//! package: a trivial text-based file archive format.
//!
//! Taken from the original `txtar` docs:
//! > The goals for the format are:
//! >   - be trivial enough to create and edit by hand.
//! >   - be able to store trees of text files describing go command test cases.
//! >   - diff nicely in git history and code reviews.
//! >
//! > Non-goals include being a completely general archive format,
//! > storing binary data, storing file modes, storing special files like
//! > symbolic links, and so on.
//! >
//! > # Txtar format
//! >
//! > A txtar archive is zero or more comment lines and then a sequence of file entries.
//! > Each file entry begins with a file marker line of the form "-- FILENAME --"
//! > and is followed by zero or more file content lines making up the file data.
//! > The comment or file content ends at the next file marker line.
//! > The file marker line must begin with the three-byte sequence "-- "
//! > and end with the three-byte sequence " --", but the enclosed
//! > file name can be surrounding by additional white space,
//! > all of which is stripped.
//! >
//! > If the txtar file is missing a trailing newline on the final line,
//! > parsers should consider a final newline to be present anyway.
//! >
//! > There are no possible syntax errors in a txtar archive.
//!
//! # Example usage
//!
//! ```rust
//! use simple_txtar::Archive;
//!
//! let s = r#"All text before the first file entry is considered a comment.
//!
//! Until we have the first file marker this is still part
//! of the comment.
//! -- example.json --
//! {
//!   "foo": 1,
//!   "bar": [ "baz" ]
//! }
//! -- example.txt --
//! Some example text in a separate file from the example json.
//! "#;
//!
//! let a = Archive::from(s);
//!
//! assert_eq!(
//!     a.comment().lines().next(),
//!     Some("All text before the first file entry is considered a comment.")
//! );
//! assert_eq!(a.comment().lines().last(), Some("of the comment."));
//!
//! // Files can be accessed using indexing or the `get` method
//! assert_eq!(Some(&a[0]), a.get("example.json"));
//!
//! // The Archive itself is also an iterator over the Files it contains
//! let mut it = a.iter();
//! assert_eq!(
//!     it.next().unwrap().name,
//!     "example.json"
//! );
//! assert_eq!(
//!     it.next().unwrap().content,
//!     "Some example text in a separate file from the example json.\n"
//! );
//! ```
#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::style,
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rustdoc::all,
    clippy::undocumented_unsafe_blocks
)]
use std::{fmt, fs, io, iter::IntoIterator, ops::Index, slice::Iter};

const NEWLINE_MARKER: &str = "\n-- ";
const MARKER: &str = "-- ";
const MARKER_END: &str = " --";
const MARKER_LEN: usize = MARKER.len() + MARKER_END.len();

/// An Archive is a collection of [File]s that have been read from a `txtar` file.
///
/// Archives can be created from a file on disk via the [Archive::from_file] method or directly
/// from a `String` or `&str` using [Archive::from]. Once you have an Archive you can access the
/// files by name using [Archive::get], index into the archive in the order that the contained
/// [File]s were defined in the original `txtar` file, or iterate over the files in order.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Archive {
    comment: String,
    files: Vec<File>,
}

impl fmt::Display for Archive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", fix_trailing_newline(&self.comment))?;
        for file in self.files.iter() {
            write!(f, "{file}")?;
        }

        Ok(())
    }
}

impl Archive {
    /// Parse a `txtar` archive from the file at the specified path.
    ///
    /// This will error if there are any issues with reading the file. To construct an [Archive]
    /// directly from a `String` or `&str` that you already have in scope, use the `from` method.
    ///
    /// ## Example
    /// ```no_run
    /// use simple_txtar::Archive;
    ///
    /// let res = Archive::from_file("my_txtar_archive");
    /// ```
    pub fn from_file(path: &str) -> io::Result<Self> {
        let raw = fs::read_to_string(path)?;

        Ok(Self::from(raw.as_str()))
    }

    /// The optional comment at the top of the `txtar` archive.
    ///
    /// If no comment was provided this will return an empty string.
    ///
    /// ## Example
    /// ```rust
    /// use simple_txtar::Archive;
    ///
    /// let a = Archive::from("comment line\n-- file1 --\nfoo");
    /// assert_eq!(a.comment(), "comment line\n");
    /// ```
    pub fn comment(&self) -> &str {
        &self.comment
    }

    /// Attempt to get a file by name from the archive.
    ///
    /// ## Example
    /// ```rust
    /// use simple_txtar::{Archive, File};
    ///
    /// let a = Archive::from("-- file1.txt --\nfoo");
    /// assert_eq!(
    ///     a.get("file1.txt"),
    ///     Some(&File {
    ///         name: "file1.txt".to_string(),
    ///         content: "foo\n".to_string()
    ///     })
    /// );
    ///
    /// assert!(a.get("bar").is_none());
    /// ```
    pub fn get(&self, filename: &str) -> Option<&File> {
        self.files.iter().find(|f| f.name == filename)
    }

    /// Iterate over the [File]s contained in this archive in the order they were specified in the
    /// original `txtar` file.
    ///
    /// ## Example
    /// ```rust
    /// use simple_txtar::{Archive, File};
    ///
    /// let a = Archive::from("-- file1.txt --\nfoo\n-- file2.txt --\nbar");
    /// let mut it = a.iter();
    /// assert_eq!(
    ///     it.next(),
    ///     Some(&File {
    ///         name: "file1.txt".to_string(),
    ///         content: "foo\n".to_string()
    ///     })
    /// );
    ///
    /// assert_eq!(
    ///     it.next(),
    ///     Some(&File {
    ///         name: "file2.txt".to_string(),
    ///         content: "bar\n".to_string()
    ///     })
    /// );
    ///
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<'_, File> {
        self.files.iter()
    }
}

impl Index<usize> for Archive {
    type Output = File;

    fn index(&self, index: usize) -> &Self::Output {
        &self.files[index]
    }
}

impl IntoIterator for Archive {
    type Item = File;
    type IntoIter = std::vec::IntoIter<File>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}

impl From<&str> for Archive {
    fn from(s: &str) -> Self {
        let (comment, mut name_after) = find_file_marker(s);
        let mut a = Archive {
            comment,
            files: Vec::new(),
        };

        let mut content;
        while let Some((name, after)) = name_after {
            (content, name_after) = find_file_marker(after);
            a.files.push(File::new(name, content));
        }

        a
    }
}

impl From<String> for Archive {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<&String> for Archive {
    fn from(s: &String) -> Self {
        Self::from(s.as_str())
    }
}

/// A File is a single file within an [Archive].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct File {
    /// The name of the file within the archive
    pub name: String,
    /// The contents of the file
    pub content: String,
}

impl File {
    fn new(name: &str, content: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            content: content.into(),
        }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "-- {} --", self.name)?;
        write!(f, "{}", fix_trailing_newline(&self.content))
    }
}

fn fix_trailing_newline(s: &str) -> String {
    let mut s = s.to_string();
    if !(s.is_empty() || s.ends_with('\n')) {
        s.push('\n');
    }

    s
}

fn find_file_marker(s: &str) -> (String, Option<(&str, &str)>) {
    let mut i = 0;

    loop {
        let (before, after) = s.split_at(i);
        let name_after = try_parse_marker(after);
        if name_after.is_some() {
            return (before.to_string(), name_after);
        }

        match after.find(NEWLINE_MARKER) {
            Some(j) => i += j + 1,
            None => return (fix_trailing_newline(s), None),
        };
    }
}

fn try_parse_marker(s: &str) -> Option<(&str, &str)> {
    if !s.starts_with(MARKER) {
        return None;
    }

    let (s, after) = match s.find('\n') {
        Some(i) => {
            let (s, after) = s.split_at(i);
            (s, after.split_at(1).1) // consume the newline
        }
        None => (s, ""),
    };

    if !(s.ends_with(MARKER_END) && s.len() >= MARKER_LEN) {
        return None;
    }

    let (_, s) = s.split_at(MARKER.len());
    let (s, _) = s.split_at(s.len() - MARKER_END.len());

    Some((s.trim(), after))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_ARCHIVE: &str = "\
comment1
comment2
-- file1 --
File 1 text.
-- foo ---
More file 1 text.
-- file 2 --
File 2 text.
-- empty --
-- noNL --
hello world
-- empty filename line --
some content
-- --";

    const SIMPLE_FORMAT_OUTPUT: &str = "\
comment1
comment2
-- file1 --
File 1 text.
-- foo ---
More file 1 text.
-- file 2 --
File 2 text.
-- empty --
-- noNL --
hello world
";

    // This is the TestParse test case from https://github.com/golang/tools/blob/master/txtar/archive_test.go
    #[test]
    fn simple_parse() {
        let expected = Archive {
            comment: "comment1\ncomment2\n".to_string(),
            files: vec![
                File::new("file1", "File 1 text.\n-- foo ---\nMore file 1 text.\n"),
                File::new("file 2", "File 2 text.\n"),
                File::new("empty", ""),
                File::new("noNL", "hello world\n"),
                File::new("empty filename line", "some content\n-- --\n"),
            ],
        };

        let parsed = Archive::from(SIMPLE_ARCHIVE);
        assert_eq!(parsed, expected);
    }

    // This is the TestFormat test case from https://github.com/golang/tools/blob/master/txtar/archive_test.go
    #[test]
    fn simple_format() {
        let a = Archive {
            comment: "comment1\ncomment2\n".to_string(),
            files: vec![
                File::new("file1", "File 1 text.\n-- foo ---\nMore file 1 text.\n"),
                File::new("file 2", "File 2 text.\n"),
                File::new("empty", ""),
                File::new("noNL", "hello world"),
            ],
        };

        assert_eq!(a.to_string(), SIMPLE_FORMAT_OUTPUT); // trailing newline is enforced
    }
}
