mod errors;
use std::{
    fmt, fs, io,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, PartialEq)]
pub struct FileMetadata {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub created_at: Option<SystemTime>,
    pub permission: Option<String>,
}

use errors::FSError;

pub fn get_files_in_directory(path: &str, hidden: bool) -> Result<Vec<FileMetadata>, FSError> {
    let path_buff = Path::new(path).to_path_buf();

    if !path_buff.exists() {
        return Err(FSError::PathNotFound(path_buff));
    }

    if !path_buff.is_dir() {
        return Err(FSError::PathIsNotADirectory(path_buff));
    }

    let mut output = vec![];

    if hidden {
        output.push(FileMetadata {
            name: String::from("."),
            is_dir: true,
            is_symlink: false,
            size: 0,
            created_at: None,
            permission: None,
        });
        output.push(FileMetadata {
            name: String::from(".."),
            is_dir: true,
            is_symlink: false,
            size: 0,
            created_at: None,
            permission: None,
        });
    }

    let dir_elements = fs::read_dir(path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => FSError::PathNotFound(path_buff.clone()),
        io::ErrorKind::PermissionDenied => FSError::PermissionError(path_buff.clone()),
        _ => FSError::UnknownError(e.to_string()),
    })?;

    for entry in dir_elements {
        match entry {
            Ok(elem) => {
                let file_name = elem.file_name().to_string_lossy().to_string();
                if file_name.starts_with('.') && !hidden {
                    continue;
                }
                let metadata = elem.metadata().map_err(|_| {
                    FSError::UnknownError("Failed to retrieve metadata".to_string())
                })?;

                output.push(FileMetadata {
                    name: file_name,
                    is_dir: metadata.is_dir(),
                    is_symlink: metadata.is_symlink(),
                    size: metadata.size(),
                    created_at: metadata.created().ok(),
                    permission: Some(format!("{:o}", metadata.permissions().mode() & 0o777)),
                });
            }
            Err(e) => eprintln!("Error when reading entry: {}", e),
        }
    }
    output.sort_by_key(|e| e.name.clone());
    Ok(output)
}

pub fn get_permission_string_from_string_number(number: String) -> String {
    let mut output_string = String::new();
    for d in number.chars() {
        if !d.is_digit(8) {
            return number;
        }
        let parsed_d = d.to_digit(8).unwrap();
        if parsed_d & 0o4 == 0o4 {
            output_string += "r"
        } else {
            output_string += "-"
        }
        if parsed_d & 0o2 == 0o2 {
            output_string += "w"
        } else {
            output_string += "-"
        }

        if parsed_d & 0o1 == 0o1 {
            output_string += "x"
        } else {
            output_string += "-"
        }
    }
    output_string
}
mod tests {
    use std::fs::File;
    use std::os::unix::fs::MetadataExt;
    use tempfile::tempdir;

    use crate::errors::FSError;
    use crate::{get_files_in_directory, get_permission_string_from_string_number, FileMetadata};

    #[test]
    fn test_all_good() {
        let dir = tempdir().unwrap();

        let hidden_file_path = dir.path().join(".my-temporary-note.txt");
        let hidden_file = File::create(hidden_file_path);

        let file_path = dir.path().join("my-temporary-note.txt");
        let file = File::create(file_path);
        let file_metadata = file.as_ref().unwrap().metadata().unwrap();

        assert_eq!(
            get_files_in_directory(dir.path().to_str().unwrap(), false).unwrap(),
            vec![FileMetadata {
                name: String::from("my-temporary-note.txt"),
                is_dir: false,
                is_symlink: false,
                size: file_metadata.size(),
                created_at: file_metadata.created().ok(),
                permission: Some("644".to_string()),
            }]
        );

        drop(file);
        drop(hidden_file);
        dir.close().unwrap();
    }

    #[test]
    fn test_all_good_hidden() {
        let dir = tempdir().unwrap();

        let file_path = dir.path().join("my-temporary-note.txt");
        let file = File::create(file_path);
        let file_metadata = file.as_ref().unwrap().metadata().unwrap();

        let hidden_file_path = dir.path().join(".my-temporary-note.txt");
        let hidden_file = File::create(hidden_file_path);
        let hidden_file_metadata = hidden_file.as_ref().unwrap().metadata().unwrap();

        let mut expected = (vec![
            FileMetadata {
                name: String::from("."),
                is_dir: true,
                is_symlink: false,
                size: 0,
                created_at: None,
                permission: None,
            },
            FileMetadata {
                name: String::from(".."),
                is_dir: true,
                is_symlink: false,
                size: 0,
                created_at: None,
                permission: None,
            },
            FileMetadata {
                name: String::from("my-temporary-note.txt"),
                is_dir: false,
                is_symlink: false,
                size: file_metadata.size(),
                created_at: file_metadata.created().ok(),
                permission: Some("644".to_string()),
            },
            FileMetadata {
                name: String::from(".my-temporary-note.txt"),
                is_dir: false,
                is_symlink: false,
                size: hidden_file_metadata.size(),
                created_at: hidden_file_metadata.created().ok(),
                permission: Some("644".to_string()),
            },
        ]);

        expected.sort_by_key(|e| e.name.clone());

        assert_eq!(
            get_files_in_directory(dir.path().to_str().unwrap(), true).unwrap(),
            expected
        );

        drop(file);
        drop(hidden_file);
        dir.close().unwrap();
    }

    #[test]
    fn test_path_not_found() {
        let result = get_files_in_directory("/missingpath", false);
        assert!(matches!(result, Err(FSError::PathNotFound(_))));
    }

    #[test]
    fn test_path_is_not_a_directory() {
        let dir = tempdir().unwrap();

        let file_path = dir.path().join("not_a_directory.txt");
        File::create(&file_path).unwrap();

        let result = get_files_in_directory(file_path.to_str().unwrap(), false);
        assert!(matches!(result, Err(FSError::PathIsNotADirectory(_))));

        dir.close().unwrap();
    }
    #[test]

    fn test_valid_permissions() {
        assert_eq!(
            get_permission_string_from_string_number("644".to_string()),
            "rw-r--r--"
        );
        assert_eq!(
            get_permission_string_from_string_number("755".to_string()),
            "rwxr-xr-x"
        );
        assert_eq!(
            get_permission_string_from_string_number("700".to_string()),
            "rwx------"
        );
        assert_eq!(
            get_permission_string_from_string_number("000".to_string()),
            "---------"
        );
    }

    #[test]
    fn test_invalid_characters() {
        assert_eq!(
            get_permission_string_from_string_number("64x".to_string()),
            "64x"
        );
        assert_eq!(
            get_permission_string_from_string_number("75a".to_string()),
            "75a"
        );
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(get_permission_string_from_string_number("".to_string()), "");
    }

    #[test]
    fn test_non_octal_digits() {
        assert_eq!(
            get_permission_string_from_string_number("89".to_string()),
            "89"
        );
    }

    #[test]
    fn test_single_digit() {
        assert_eq!(
            get_permission_string_from_string_number("7".to_string()),
            "rwx"
        );
        assert_eq!(
            get_permission_string_from_string_number("5".to_string()),
            "r-x"
        );
        assert_eq!(
            get_permission_string_from_string_number("0".to_string()),
            "---"
        );
    }
}
