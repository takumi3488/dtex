mod engine;
use engine::parser::to_latex;

use clap::Parser;
use std::{
    error::Error,
    fs::{read_to_string, write},
    result::Result,
};
use tectonic;

#[derive(Parser, Debug)]
#[clap(name = "texd")]
struct Args {
    /// Input file
    /// Extension must be .d.tex
    input: String,

    /// Flag to output .tex file
    #[arg(short, long)]
    tex: bool,

    /// Flag not to output .pdf file
    #[arg(long)]
    no_pdf: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let input = args.input;
    if !input.contains(".d.tex") {
        return Err("Input file must be a .d.tex file".into());
    }

    let tex = args.tex;

    let dtex = read_to_string(input.clone())?;
    let latex = to_latex(&(dtex + "\n\n"))?;

    if tex {
        write(input.clone().replace(".d.tex", ".tex"), latex.clone())?;
    }

    if args.no_pdf {
        return Ok(());
    }

    let pdf_data = tectonic::latex_to_pdf(latex)?;
    write(input.replace(".d.tex", ".pdf"), pdf_data)?;

    Ok(())
}
