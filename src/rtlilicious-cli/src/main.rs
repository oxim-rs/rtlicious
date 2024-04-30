use clap::{Parser, Subcommand};
#[cfg(feature = "trace")]
use nom_tracable::{cumulative_histogram, histogram};
use std::{path::PathBuf, process};

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
            let file = std::fs::read_to_string(opts.input.clone()).unwrap();
            let ret = rtlilicious::parse(&file);
            if let Err(e) = ret {
                //let safe_rem: Vec<String> = e.lines().take(5).map(|l| l.to_string()).collect();
                //log::error!("Failed to parse RTLIL file, the element we were unable to parse starts like this: \n {}", safe_rem.join("\n"));
                log::error!(
                    "The parser could not advance furter than the element begining here, we couldn't parse it or a child element: {}:{} :",
                    opts.input.file_name().unwrap().to_str().unwrap(),
                    e.location_line()
                );
                // get line content:
                dbg!(e.location_line());
                let loc = e.location_offset();
                dbg!(loc);
                let line = file
                    .chars()
                    .skip(loc)
                    .skip_while(|c| *c != '\n')
                    .skip(1)
                    .take_while(|c| *c != '\n')
                    .collect::<String>();
                log::error!("  {}", line);

                process::exit(1);
            }
            let design = ret.unwrap();
            if opts.print {
                println!("{:#?}", design);
            }
            log::info!("Parsed RTLIL file successfully");
            log::info!("stats:");
            let modules = design.modules().len();
            let top_module_id = design
                .modules()
                .iter()
                .find(|(_, m)| m.attributes().contains_key("top"));
            log::info!("  modules: {}", modules);
            if let Some((id, _)) = top_module_id {
                log::info!("  top: {}", id);
            }
            let mut wires = 0;
            let mut cells = 0;
            for module in design.modules() {
                wires += module.1.wires().len();
                cells += module.1.cells().len();
            }
            log::info!("  wires: {}", wires);
            log::info!("  cells: {}", cells);

            // Show histogram
            #[cfg(feature = "trace")]
            {
                histogram();
                cumulative_histogram();
            }
        }
    }
}
