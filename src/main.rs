use std::env;
use std::path;
use structopt::StructOpt;

struct Cli {
   query: String,
   filename: path::PathBuf,
}

fn main() {
    let query: String = env::args().nth(1).expect("No query given");
    let filename: String = env::args().nth(2).expect("No csv filename given");
    let args = Cli {
        query: query,
        filename: path::PathBuf::from(filename),
    };

    println!("Searching for {}", args.query);
    println!("In file {}", args.filename.into_os_string().into_string().unwrap());
}

