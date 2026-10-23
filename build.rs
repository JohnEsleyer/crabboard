fn main() {
    cc::Build::new()
        .cpp(true)
        .file("sketch.cpp")
        .include("include")
        .compile("sketch");

    println!("cargo:rerun-if-changed=sketch.cpp");
    println!("cargo:rerun-if-changed=include/Arduino.h");
}
