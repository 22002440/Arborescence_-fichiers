/// A module representing a file tree structure and related functionalities.
mod file_tree;

/// A module providing functionality to print the file tree.
mod print_tree;

/// A module defining the Size struct used to represent the size of files or directories.
mod size;

use clap::{Parser, Subcommand};
use file_tree::FileTree;
use std::path::{Path, PathBuf};

/// Command-line interface structure defined using the `clap` crate.

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable lexicographic sorting of file paths.
    #[arg(long = "lexicographic-sort")]
    lexicographic_sort: bool,

    /// Filter the file tree based on a provided string.
    #[arg(long = "filter")]
    filter: Option<String>,
}

/// Enum representing different commands that can be executed via the command-line interface.

#[derive(Subcommand, Debug)]
enum Commands {

    /// Show the disk usage tree for the given path 
    
    Usage {

        /// (default '.')
        
        path: Option<PathBuf>,
    },

    /// Find and display duplicate files within the given path.

    Duplicate{
        path: Option<PathBuf>,
    }
}

/// The main function of the program.

fn main() -> std::io::Result<()> {
    // Parse the command-line arguments using the defined CLI structure.

    let cli = Cli::parse();

    // Match on the provided subcommand and execute the corresponding functionality.

    match &cli.command {
        Commands::Usage { path } => {
            // Determine the path to analyze, defaulting to '.' if not provided.
            let path = path.as_deref().unwrap_or(Path::new("."));
            let file_tree = FileTree::new(path)?;

            // Create a file tree for the specified path.
            if let Some(filter) = &cli.filter {
                file_tree.show_filtered(filter, cli.lexicographic_sort); //cargo run --bin main  -- option<--lexicographic-sort> --filter jpg usage option<path>
            } else if cli.lexicographic_sort {
                file_tree.show_lexicographic(); //cargo run --bin --main -- --lexicographic-sort usage option<path>
            } else {
                file_tree.show(); //cargo run --bin main -- usage option<path>
            }
        }
        Commands::Duplicate { path } => { //cargo run --bin main -- duplicate

            // Determine the path to analyze, defaulting to '.' if not provided.
            let path = path.as_deref().unwrap_or(Path::new("."));

            // Create a file tree for the specified path.
            let file_tree = FileTree::new(path)?;
        
            // Find and display duplicate files in the file tree.
            let duplicates = file_tree.find_duplicates();
        
            // Display the duplicates.
            for (signature, paths) in duplicates {
                println!("Signature de Doublon : {}", signature);
                for path in paths {
                    println!("  - {}", path.display());
                }
            }
        }
    }
    Ok(())
}
