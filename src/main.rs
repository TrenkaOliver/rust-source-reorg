use std::{fs::{self, OpenOptions}, io::{self, Write}, path::{Path, PathBuf}};
use rust_source_reorg::*;

fn enter_path(printing: &str) -> PathBuf {
    let mut path = String::new();
    println!("{}", printing);
    io::stdin().read_line(&mut path).expect("error reading input");
    Path::new(path.trim()).join("main.rs")
}   

fn main() -> Result<(), io::Error> {
    let path = enter_path("Enter the path of the src");
    let write_to = fs::canonicalize(Path::new("./"))?.join(format!("{}_reorganized", path.parent().unwrap().parent().unwrap().file_name().unwrap().to_str().unwrap())).join("main.rs");
    
    if path == write_to {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Source and destionation is the same"));
    }

    if write_to.is_dir() {
        fs::remove_dir_all(write_to.parent().unwrap())?;
    }
    
    write_file_recursive(&path, &write_to)?;

    Ok(())
}


fn write_file_recursive(read_from: &Path, write_to: &Path) -> Result<(), io::Error> {
    if read_from == write_to {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Source and destionation is the same"));
    }

    if let Some(dir) = write_to.parent() {
        fs::create_dir_all(dir)?;
    }

    let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(write_to)?;
    
    for block in fs::read_to_string(read_from)?.split_inclusive(';') {
        file.write_all(block.as_bytes())?;

        let mut block = block.trim().to_string();
        if block.is_empty() { continue; }
        if remove_comments(&mut block) { continue; }
        remove_scopes(&mut block);
        let special_path = handle_attributes(&mut block);
        cut_off_between_strings("pub", "mod", false, &mut block, None);

        if let Some(module_name) = block.strip_prefix("mod") {
            if !module_name.chars().next().unwrap().is_whitespace() {continue;}
            let module_name = match module_name.strip_suffix(';') {
                Some(s) => s.trim(),
                None => continue,
            };
            println!("|{}|", module_name);
            let read_from_parent = fs::canonicalize(read_from.parent().unwrap())?;
            let write_to_parent = fs::canonicalize(write_to.parent().unwrap())?;

            if let Some(special_path) = special_path {
                let path_to_read = fs::canonicalize(read_from_parent.join(special_path))?;

                if let Ok(path_to_write) = path_to_read.strip_prefix(read_from_parent) {
                    let path_to_write = write_to_parent.join(path_to_write);
                    write_file_recursive(&path_to_read, &path_to_write)?;
                } else {
                    let read_from_grand_parent = read_from.parent().unwrap().parent().unwrap();

                    if let Ok(extra_path) = path_to_read.strip_prefix(&read_from_grand_parent) {
                        let path_to_write = write_to_parent.parent().unwrap().join(extra_path);
                        write_file_recursive(&path_to_read, &path_to_write)?;
                    }
                }
            } else {
                let path_to_read = read_from_parent.join(module_name).join("mod.rs");
                let path_to_write = if read_from.file_name().unwrap() == "main.rs" {
                    write_to_parent.join(format!("{}.rs", module_name))
                } else {
                    write_to_parent.join(write_to.file_stem().unwrap()).join(format!("{}.rs", module_name))
                };
                match write_file_recursive(&path_to_read, &path_to_write) {
                    Ok(()) => (),
                    Err(error) => match error.kind() {
                        io::ErrorKind::NotFound => {
                            let path_to_read = read_from_parent.join(format!("{}.rs", module_name));
                            match write_file_recursive(&path_to_read, &path_to_write) {
                                Ok(()) => (),
                                Err(error) => match error.kind() {
                                    io::ErrorKind::NotFound => {
                                        let path_to_read = read_from_parent.join(read_from.file_stem().unwrap()).join(format!("{}.rs", module_name));
                                        write_file_recursive(&path_to_read, &path_to_write)?;
                                    }

                                    _ => return Err(error)
                                }
                            } 
                        },
                        _ => return Err(error),
                    }
                }
            }
        }

    }

    Ok(())
}