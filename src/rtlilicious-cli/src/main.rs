use clap::{Parser, Subcommand};
use nom_tracable::histogram;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command()]
    Parse(ParseOpts),
}

#[derive(Parser)]
struct ParseOpts {
    /// The input file to parse
    #[arg(short, long)]
    input: PathBuf,
    // option to print
    #[arg(short, long)]
    print: bool,
}

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let args = Cli::parse();

    match args.command {
        Commands::Parse(opts) => {
            let file = std::fs::read_to_string(opts.input).unwrap();
            let ret = rtlilicious::parse(&file);
            if let Err(e) = ret {
                let safe_rem: Vec<String> = e.lines().take(5).map(|l| l.to_string()).collect();
                log::error!("Failed to parse RTLIL file, the element we were unable to parse starts like this: \n {}", safe_rem.join("\n"));
            }
            let design = ret.unwrap();
            if opts.print {
                println!("{:#?}", design);
            }
            log::info!("Parsed RTLIL file successfully");
            log::info!("stats:");
            let modules = design.modules().len();
            log::info!("  modules: {}", modules);
            let mut wires = 0;
            let mut cells = 0;
            for module in design.modules() {
                wires += module.1.wires().len();
                cells += module.1.cells().len();
            }
            log::info!("  wires: {}", wires);
            log::info!("  cells: {}", cells);

            // Show histogram
            histogram();
        }
    }
}
