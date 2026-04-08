use clap::Parser;
use std::{env, fs, io::ErrorKind, os::unix::fs as Fs, path::PathBuf};

use anyhow::{Result, anyhow};

// #[derive(Debug)]
// enum DotheaErrors {
//     NotADirectory,
//     InvalidPath,
// }
//

const IGNORE_LIST: [&str; 2] = [".gitignore", ".git"];

#[derive(Parser)]
struct Args {
    directory: Option<PathBuf>,
    symlink_directory: Option<PathBuf>,
    #[arg(short = 'D', long)]
    delete: bool,
}

fn make_files_symlinks(
    paths: Vec<String>,
    origin_base_dir: &str,
    remove_links: bool,
    home_dir: PathBuf,
) -> Result<()> {
    for path in paths {
        let file_path = path
            .strip_prefix(origin_base_dir)
            .ok_or_else(|| anyhow!("Failed to get filepath {}", path))?;
        let mut link_path = PathBuf::new();
        link_path.push(&home_dir);
        link_path.push(file_path);
        let dir_path = link_path.parent().unwrap();
        fs::create_dir_all(dir_path)?;
        println!("{} becomes {}", path, link_path.to_str().unwrap());
        if remove_links {
            if link_path.is_symlink() {
                fs::remove_file(link_path)?;
            }
        } else {
            match Fs::symlink(&path, link_path) {
                Ok(_) => {}
                Err(ref e) if e.kind() == ErrorKind::AlreadyExists => {
                    println!("{} already exists in home", path);
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    Ok(())
}

fn loop_over_dirc(curr_path: &str, paths: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(curr_path)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| IGNORE_LIST.contains(&name))
        {
            continue;
        }
        if path.is_file() {
            paths.push(path.to_str().unwrap().to_string());
        } else {
            loop_over_dirc(path.to_str().unwrap(), paths)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Args::parse();
    let remove_links = cli.delete;
    let directory_path: PathBuf = cli.directory.unwrap_or(env::current_dir()?);
    let symlink_dir: PathBuf = cli
        .symlink_directory
        .unwrap_or(dirs::home_dir().ok_or_else(|| {
            anyhow!("Cannot determine home directory or directory to symlink to")
        })?);

    if !directory_path.exists() {
        return Err(anyhow!("The given path doesn't exist"));
    }
    if !directory_path.is_dir() {
        return Err(anyhow!("The given path is not a directory"));
    }

    let mut paths: Vec<String> = vec![];
    loop_over_dirc(directory_path.to_str().unwrap(), &mut paths)?;

    make_files_symlinks(
        paths,
        directory_path.to_str().unwrap(),
        remove_links,
        symlink_dir,
    )?;

    Ok(())
}
