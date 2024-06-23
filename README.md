# A simple txtar parser for Rust

A simple implementation of the go [txtar](https://github.com/golang/tools/blob/master/txtar/archive.go)
package: a trivial text-based file archive format.

Taken from the original go `txtar` docs:

> The goals for the format are:
>   - be trivial enough to create and edit by hand.
>   - be able to store trees of text files describing go command test cases.
>   - diff nicely in git history and code reviews.
>
> Non-goals include being a completely general archive format,
> storing binary data, storing file modes, storing special files like
> symbolic links, and so on.
>
> # Txtar format
>
> A txtar archive is zero or more comment lines and then a sequence of file entries.
> Each file entry begins with a file marker line of the form "-- FILENAME --"
> and is followed by zero or more file content lines making up the file data.
> The comment or file content ends at the next file marker line.
> The file marker line must begin with the three-byte sequence "-- "
> and end with the three-byte sequence " --", but the enclosed
> file name can be surrounding by additional white space,
> all of which is stripped.
>
> If the txtar file is missing a trailing newline on the final line,
> parsers should consider a final newline to be present anyway.
>
> There are no possible syntax errors in a txtar archive.


# Example usage

```rust
use simple_txtar::Archive;

let s = r#"All text before the first file entry is considered a comment.

Until we have the first file marker this is still part
of the comment.
-- example.json --
{
  "foo": 1,
  "bar": [ "baz" ]
}
-- example.txt --
Some example text in a separate file from the example json.
"#;

let a = Archive::from(s);

assert_eq!(
    a.comment().lines().next(),
    Some("All text before the first file entry is considered a comment.")
);
assert_eq!(a.comment().lines().last(), Some("of the comment."));

// Files can be accessed using indexing or the `get` method
assert_eq!(Some(&a[0]), a.get("example.json"));

// The Archive itself is also an iterator over the Files it contains
let mut it = a.iter();
assert_eq!(
    it.next().unwrap().name,
    "example.json"
);
assert_eq!(
    it.next().unwrap().content,
    "Some example text in a separate file from the example json.\n"
);
```
