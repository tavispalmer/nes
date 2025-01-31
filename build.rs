use std::process::Command;

fn main() {
    // build notsofatso
    Command::new("cmake")
        .arg("-Bbuild")
        .current_dir("notsofatso")
        .output()
        .unwrap();

    Command::new("cmake")
        .arg("--build")
        .arg("build")
        .current_dir("notsofatso")
        .output()
        .unwrap();

    // link notsofatso
    println!("cargo:rustc-link-search=native=notsofatso/build");
    println!("cargo:rustc-link-lib=static=nsf_core");
}
