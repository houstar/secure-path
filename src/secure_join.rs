use std::path::Path;
use std::path::PathBuf;

// This function constructs a canonicalized path by combining the `rootfs` and `unsafe_path` elements.
// The resulting path is guaranteed to be ("below" / "in a directory under") the `rootfs` directory.
//
// Parameters:
//
// - `rootfs` is the absolute path to the root of the containers root filesystem directory.
// - `unsafe_path` is path inside a container. It is unsafe since it may try to "escape" from the containers
//    rootfs by using one or more "../" path elements or is its a symlink to path.
pub fn secure_join(rootfs: &str, unsafe_path: &str) -> String {
    let mut path = PathBuf::from(format!("{}/", rootfs));
    let unsafe_p = Path::new(&unsafe_path);

    for it in unsafe_p.iter() {
        let it_p = Path::new(&it);

        // if it_p leads with "/", path.push(it) will be replace as it, so ignore "/"
        if it_p.has_root() {
            continue;
        };

        path.push(it);
        if let Ok(v) = path.read_link() {
            if v.is_absolute() {
                path = PathBuf::from(format!("{}{}", rootfs, v.to_str().unwrap().to_string()));
            } else {
                path.pop();
                for it in v.iter() {
                    path.push(it);
                    if path.exists() {
                        path = path.canonicalize().unwrap();
                        if !path.starts_with(rootfs) {
                            path = PathBuf::from(rootfs.to_string());
                        }
                    }
                }
            }
        }
        // skip any ".."
        if path.ends_with("..") {
            path.pop();
        }
    }

    path.to_str().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs;
    use tempfile::tempdir;
    #[test]
    fn test_secure_join() {
        #[derive(Debug)]
        struct TestData<'a> {
            name: &'a str,
            rootfs: &'a str,
            unsafe_path: &'a str,
            symlink_path: &'a str,
            result: &'a str,
        }

        // create tempory directory to simulate container rootfs with symlink
        let rootfs_dir = tempdir().expect("failed to create tmpdir");
        let rootfs_path = rootfs_dir.path().to_str().unwrap();

        let tests = &[
            TestData {
                name: "rootfs_not_exist",
                rootfs: "/home/rootfs",
                unsafe_path: "a/b/c",
                symlink_path: "",
                result: "/home/rootfs/a/b/c",
            },
            TestData {
                name: "relative_path",
                rootfs: "/home/rootfs",
                unsafe_path: "../../../a/b/c",
                symlink_path: "",
                result: "/home/rootfs/a/b/c",
            },
            TestData {
                name: "skip any ..",
                rootfs: "/home/rootfs",
                unsafe_path: "../../../a/../../b/../../c",
                symlink_path: "",
                result: "/home/rootfs/a/b/c",
            },
            TestData {
                name: "rootfs is null",
                rootfs: "",
                unsafe_path: "",
                symlink_path: "",
                result: "/",
            },
            TestData {
                name: "relative softlink beyond container rootfs",
                rootfs: rootfs_path,
                unsafe_path: "1",
                symlink_path: "../../../",
                result: rootfs_path,
            },
            TestData {
                name: "abs softlink points to the non-exist directory",
                rootfs: rootfs_path,
                unsafe_path: "2",
                symlink_path: "/dddd",
                result: &format!("{}/dddd", rootfs_path).as_str().to_owned(),
            },
            TestData {
                name: "abs softlink points to the root",
                rootfs: rootfs_path,
                unsafe_path: "3",
                symlink_path: "/",
                result: &format!("{}/", rootfs_path).as_str().to_owned(),
            },
        ];

        for (i, t) in tests.iter().enumerate() {
            // Create a string containing details of the test
            let msg = format!("test[{}]: {:?}", i, t);

            // if is_symlink, then should be prepare the softlink environment
            if t.symlink_path != "" {
                fs::symlink(t.symlink_path, format!("{}/{}", t.rootfs, t.unsafe_path)).unwrap();
            }
            let result = secure_join(t.rootfs, t.unsafe_path);

            // Update the test details string with the results of the call
            let msg = format!("{}, result: {:?}", msg, result);

            // Perform the checks
            assert!(result == t.result, "{}", msg);
        }
    }
}
