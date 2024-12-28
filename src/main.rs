use chrono::{DateTime, Local};
use clap::{arg, command, value_parser, Arg, ArgAction, Command};
use colored::Colorize;
use lsr::{get_files_in_directory, get_permission_string_from_string_number, FileMetadata};
use std::path::PathBuf;

fn main() {
    let matches = Command::new("lsr")
        .version("1.0")
        .about("Minimal ls cmd util rewrite in rust")
        .arg(Arg::new("path").required(false))
        .arg(
            Arg::new("all")
                .short('a')
                .action(ArgAction::SetTrue)
                .help("List the hidden files"),
        )
        .arg(
            Arg::new("long")
                .short('l')
                .action(ArgAction::SetTrue)
                .help("List the metadata properties"),
        )
        .arg(
            Arg::new("color")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Colorize the output"),
        )
        .get_matches();

    let path = matches
        .get_one::<String>("path")
        .map(String::as_str)
        .unwrap_or(".");

    let include_hidden = matches.get_flag("all");
    let long_format = matches.get_flag("long");
    let color = matches.get_flag("color");

    let results = get_files_in_directory(path, include_hidden);

    match results {
        Ok(elements) => {
            let max_name_width = elements.iter().map(|e| e.name.len()).max().unwrap_or(0);
            let max_size_width = elements
                .iter()
                .map(|e| e.size.to_string().len())
                .max()
                .unwrap_or(0);
            for e in elements {
                if !long_format {
                    short_format_print(e, color);
                } else {
                    long_format_print(e, max_size_width, max_name_width, color);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn short_format_print(entry: FileMetadata, colorized: bool) {
    let mut s_to_print = format!("{}", entry.name).white();

    if colorized && entry.is_dir {
        s_to_print = s_to_print.blue();
    }

    if colorized && entry.is_symlink {
        s_to_print = s_to_print.red();
    }

    println!("{}", s_to_print)
}

fn long_format_print(
    entry: FileMetadata,
    max_size_width: usize,
    max_name_width: usize,
    colorized: bool,
) {
    let created_at = entry
        .created_at
        .map(|time| {
            let datetime: DateTime<Local> = time.into();
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or(" ".to_string());

    let s_permission =
        get_permission_string_from_string_number(entry.permission.unwrap_or("".to_string()));
    let mut s_to_print = format!(
        "{:20} {:9} {:>size_width$} {:<name_width$}",
        created_at,
        s_permission,
        entry.size,
        entry.name,
        size_width = max_size_width,
        name_width = max_name_width
    )
    .white();

    if colorized && entry.is_dir {
        s_to_print = s_to_print.blue();
    }

    if colorized && entry.is_symlink {
        s_to_print = s_to_print.red();
    }

    println!("{}", s_to_print);
}
