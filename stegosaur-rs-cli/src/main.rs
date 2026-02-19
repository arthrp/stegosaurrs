use clap::{Parser, Subcommand};
use stegosaur_rs;

#[derive(Parser)]
#[command(name = "stegosaur-rs-cli")]
#[command(about = "`Stegousaurrs` CLI - encode or decode text in images")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode text into an image
    Encode {
        /// Path to the input image
        #[arg(short, long)]
        input: String,

        /// Text to encode
        #[arg(short, long)]
        text: String,

        /// Path for the output image
        #[arg(short, long)]
        output: String,
    },

    /// Decode text from an image
    Decode {
        /// Path to the image containing encoded text
        #[arg(short, long)]
        input: String,
    },
}

fn main() {
    let cli = Cli::parse();

    println!("{}", std::env::temp_dir().display());

    match cli.command {
        Commands::Encode { input, text, output } => {
            if let Err(e) = stegosaur_rs::encode_lossless(&input, &text, &output) {
                eprintln!("Error encoding: {}", e);
                std::process::exit(1);
            }
            println!("Encoded text into {}", output);
        }
        Commands::Decode { input } => {
            match stegosaur_rs::decode_lossless(&input) {
                Ok(decoded) => println!("{}", decoded),
                Err(e) => {
                    eprintln!("Error decoding: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
