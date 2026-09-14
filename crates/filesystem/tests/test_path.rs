use std::path::Path;

use inox_filesystem::NormalizedPath;

#[test]
fn normalizes_relative_and_absolute_paths() {
    assert_eq!(
        Path::new("foo/./bar/../baz").normalize(),
        Path::new("foo/baz")
    );
    assert_eq!(Path::new("./foo").normalize(), Path::new("foo"));
    assert_eq!(Path::new(".").normalize(), Path::new("."));
    assert_eq!(Path::new("foo/..").normalize(), Path::new("."));
    assert_eq!(Path::new("foo/../../bar").normalize(), Path::new("../bar"));
    assert_eq!(Path::new("foo/bar/").normalize(), Path::new("foo/bar"));
    assert_eq!(Path::new("/foo/../bar").normalize(), Path::new("/bar"));
}
