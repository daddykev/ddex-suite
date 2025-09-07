// core/build.rs
fn main() {
    #[cfg(feature = "typescript")]
    {
        println!("cargo:warning=Generating TypeScript definitions...");
        // ts-rs will generate types during compilation
    }
}