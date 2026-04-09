use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../ui/src");
    println!("cargo:rerun-if-changed=assets/input.css");

    let status = Command::new("npx")
        .args(&[
            "@tailwindcss/cli",
            "-i",
            "assets/input.css",
            "-o",
            "assets/tailwind.css",
        ])
        .status()
        .expect("Failed to execute Tailwind CSS");

    if !status.success() {
        println!("cargo:warning=Tailwind CSS compilation failed. Ensure you have run `npm install` in the workspace root.");
    }
}
