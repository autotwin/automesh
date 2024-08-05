use automesh::Spn;
use clap::Parser;

#[derive(Parser)]
#[command(about = "
Automatic mesh generation. \n
\x1b[1;4mNotes:\x1b[0m
  - Input/output file types are inferred.
  - Scaling is applied before translation.",
arg_required_else_help = true, long_about = None, version)]
struct Args {
    /// Name of the NumPy input file.  Example: --input <input_file>.npy   (TODO: work in progress <input_file>.spn)
    #[arg(short, long)]
    input: String,

    /// Name of the Exodus output file. Example: --output <output_file>.exo
    #[arg(short, long)]
    output: String,

    /// Number of voxels in the x-direction.
    #[arg(short = 'x', long, default_value_t = 0)]
    nelx: usize,

    /// Number of voxels in the y-direction.
    #[arg(short = 'y', long, default_value_t = 0)]
    nely: usize,

    /// Number of voxels in the z-direction.
    #[arg(short = 'z', long, default_value_t = 0)]
    nelz: usize,

    /// Scale of coordinates in the x-direction.
    #[arg(long, default_value_t = 1.0)]
    xscale: f64,

    /// Scale of coordinates in the y-direction.
    #[arg(long, default_value_t = 1.0)]
    yscale: f64,

    /// Scale of coordinates in the z-direction.
    #[arg(long, default_value_t = 1.0)]
    zscale: f64,

    /// Translation of coordinates in the x-direction.
    #[arg(long, default_value_t = 0.0)]
    xtranslate: f64,

    /// Translation of coordinates in the y-direction.
    #[arg(long, default_value_t = 0.0)]
    ytranslate: f64,

    /// Translation of coordinates in the z-direction.
    #[arg(long, default_value_t = 0.0)]
    ztranslate: f64,
}

#[cfg(feature = "exodus")]
extern "C" {
    fn ex_close(exoid: i32) -> i32;
}

fn main() {
    #[cfg(feature = "exodus")]
    let a = unsafe {
        let b = ex_close(0);
        b
    };
    #[cfg(feature = "exodus")]
    println!("{:?}", a);

    let args = Args::parse();
    if args.xscale <= 0.0 {
        panic!("Need to specify xscale > 0.")
    } else if args.yscale <= 0.0 {
        panic!("Need to specify yscale > 0.")
    } else if args.zscale <= 0.0 {
        panic!("Need to specify zscale > 0.")
    }
    let input = if args.input.ends_with(".npy") {
        Spn::from_npy(&args.input)
    } else if args.input.ends_with(".spn") {
        if args.nelx < 1 {
            panic!("Need to specify nelx > 0.")
        } else if args.nely < 1 {
            panic!("Need to specify nely > 0.")
        } else if args.nelz < 1 {
            panic!("Need to specify nelz > 0.")
        } else {
            Spn::new(&args.input, [args.nelx, args.nely, args.nelz])
        }
    } else {
        panic!("Invalid input ({}) specified.", args.input)
    };
    let output = if args.output.ends_with(".exo") {
        input.into_exodus(
            &[args.xscale, args.yscale, args.zscale],
            &[args.xtranslate, args.ytranslate, args.ztranslate],
        )
    } else {
        panic!("Invalid output ({}) specified.", args.output)
    };
    output.write(&args.output);
}
