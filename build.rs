use std::{env, path::Path};

fn main() {
    let pwd_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&*pwd_dir).join("target/seacas/lib");
    println!("cargo:rustc-link-search=native={}", path.to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib=exodus");
}
