mod extract;

use crate::extract::extract_package;
use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!("{}", &args);

    if args.len() == 1 {
        println!("Invalid Arguments: Too little supplied: ");
        println!("./unitypkgextractor <file_path>");
        exit(1)
    } else if args.len() == 2 {
        let input_file = &args[1];
        extract_package(input_file).expect("Failed extracting package");
        println!("Extracting Unity Package")
    } else {
        println!("Invalid Arguments: Too many supplied: ");
        println!("./unitypkgextractor <file_path> [output_path]");
        exit(1)
    }
    println!("Done!")
}
