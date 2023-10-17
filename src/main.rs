mod vm_translator;

use std::env;
use std::fs::write;
use std::path::{Path, PathBuf};

fn write_lines(outfile: &PathBuf, asm_output: &[String]) {
    write(outfile, asm_output.join("\n")).expect(&format!(
        "Failed to write hack assembly output to {}",
        outfile.to_str().unwrap()
    ));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: vm_translator_rs <infile or directory>");
    }
    let infile_or_directory = Path::new(&args[1]);
    let outfile = if infile_or_directory.is_dir() {
        infile_or_directory
            .join(infile_or_directory.file_name().unwrap())
            .with_extension("asm")
    } else {
        infile_or_directory.with_extension("asm")
    };
    println!(
        "Translating {} and writing hack assembly output to {} ...",
        infile_or_directory.to_str().unwrap(),
        outfile.to_str().unwrap()
    );
    let asm_output = if infile_or_directory.is_dir() {
        vm_translator::translate_directory(infile_or_directory)
    } else {
        vm_translator::translate_file(infile_or_directory)
    };
    write_lines(&outfile, &asm_output);
    println!(
        "Translation successful; output written to {}",
        outfile.to_str().unwrap()
    );
}
