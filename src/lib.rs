/*!
# Secure Path

This provides method to constraint path inside the specified rootfs.
```rust
extern crate secure_path;
use secure_path::secure_join::*;

let rootfs = "/home/rootfs";
let p = "../../../a/b/c";

assert_eq!("/home/rootfs/a/b/c", secure_join(rootfs, p));
```
*/
pub mod secure_join;
