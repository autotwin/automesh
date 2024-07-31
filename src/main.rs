use automesh::Spn;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the input file.
    #[arg(short, long)]
    input: String,

    /// Name of the output file.
    #[arg(short, long)]
    output: String,
}

fn main() {
    let args = Args::parse();
    let input = if args.input.ends_with(".npy") {
        Spn::from_npy(&args.input)
    } else {
        panic!("Invalid input {} specified.", args.input)
    };
    let output = if args.output.ends_with(".exo") {
        input.into_exodus()
    } else {
        panic!("Invalid output {} specified.", args.output)
    };
    output.write(&args.output);
}