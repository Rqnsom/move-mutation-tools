// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{cli, configuration::Configuration};
use move_package::source_package::layout::SourcePackageLayout;
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

/// Sets up the path for the mutant.
///
/// It creates the directory structure for the mutant and returns the path to the mutant.
/// This function recognizes if the file is inside a package and creates the directory structure
/// according to its relative path inside the package. If the file is not inside any package,
/// it creates the directory structure in the output directory.
/// Example:
/// The file to be mutated is located in "/a/b/c/sources/X/Y/file.move" (`original_file`).
/// This function constructs the following output path for file.move:
/// "`output_dir/X/Y/file_index.move`"
/// It finds the package root for the file, which is "/a/b/c", then it append the relative path to the output directory.
///
/// If the file is not inside any package, it creates the directory structure in the output directory like:
/// The file to be mutated is located in "/a/b/c/file.move" (`original_file`).
/// This function constructs the following output path for file.move:
/// "`output_dir/file_index.move`"
///
/// Paths can be absolute or relative - it doesn't matter. The function will handle it by
/// canonicalizing the path. The returned path is always relative to the package directory root.
///
/// # Arguments
///
/// * `output_dir` - The directory where the mutant will be output.
/// * `filename` - The path to the original file.
/// * `index` - The index of the mutant.
///
/// # Returns
///
/// * `PathBuf` - The path to the mutant.
pub(crate) fn setup_mutant_path(
    output_dir: &Path,
    original_file: &Path,
    unique_mutant_id: u64,
) -> anyhow::Result<PathBuf> {
    trace!("Trying to set up mutant path for {original_file:?}");

    let file_path_canonicalized = original_file.canonicalize()?;

    // Try to find package root for the file. If the file is not inside any package, assume that it is a single file.
    let root = SourcePackageLayout::try_find_root(&file_path_canonicalized);
    let root_path = if root.is_err() {
        debug!("No package root for {file_path_canonicalized:?}. Assuming mutating a single file.");
        file_path_canonicalized.clone()
    } else {
        // In case of file is inside the package it must follow the Move structure. So we can assume that
        // there will be a sources directory inside the package root. We can omit it.
        root?.join("sources")
    };

    // Stripping whole prefix before file to get it relative path inside the package.
    let relative_path = file_path_canonicalized.strip_prefix(&root_path)?;

    // Construct the directory structure for that specified file in the output directory. If file was inside the package,
    // parent() will return its relative folder path inside the package. If file was outside any package, parent() will return None.
    let output_struct = output_dir.join(relative_path.parent().unwrap_or(Path::new("")));

    // Create the directory structure for that specified file in the output directory. Ignore errors if the directory already exists.
    if let Err(e) = fs::create_dir_all(&output_struct) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(anyhow::anyhow!(
                "Cannot create directory structure for {original_file:?} in {output_dir:?}"
            ));
        }
    }

    let filename = original_file
        .file_stem()
        .ok_or(anyhow::anyhow!("Cannot get file stem of {original_file:?}"))?;

    // Deal with the file as OsString to avoid problems with non-UTF8 characters.
    let mut mutant_path = filename.to_os_string();
    mutant_path.push(OsString::from(format!("_mutant_{unique_mutant_id:x}.move")));

    let mutant_path = output_struct.join(mutant_path);
    Ok(mutant_path)
}

/// Sets up the output directory for the mutants.
///
/// # Arguments
///
/// * `mutator_configuration` - The configuration for the mutator.
///
/// # Returns
///
/// * `anyhow::Result<PathBuf>` - Returns the path to the output directory if successful, or an error if any error occurs.
pub(crate) fn setup_output_dir(mutator_configuration: &Configuration) -> anyhow::Result<PathBuf> {
    // It's safe to unwrap here as we have default value for output directory.
    let output_dir = mutator_configuration
        .project
        .out_mutant_dir
        .clone()
        .unwrap_or(PathBuf::from(cli::DEFAULT_OUTPUT_DIR));
    trace!("Trying to set up output directory to: {output_dir:?}");

    // Check if output directory exists and if it should be overwritten.
    if output_dir.exists() && mutator_configuration.project.no_overwrite {
        return Err(anyhow::anyhow!(
            "Output directory already exists and --no-overwrite flag was used."
        ));
    }

    let _ = fs::remove_dir_all(&output_dir);
    fs::create_dir(&output_dir)?;

    info!("Output directory set to: {output_dir:?}");

    Ok(output_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli;
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use tempfile::tempdir;

    #[test]
    fn setup_mutant_path_handles_non_utf8_characters() {
        let output_dir = Path::new("mutants_output");
        let filename = Path::new("💖");
        let mutant_id = 0;

        fs::File::create(filename).unwrap();
        let result = setup_mutant_path(output_dir, filename, mutant_id);
        fs::remove_file(filename).unwrap();
        fs::remove_dir_all(output_dir).unwrap();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PathBuf::from(format!("mutants_output/💖_mutant_{mutant_id}.move"))
        );
    }

    #[test]
    fn setup_mutant_path_handles_file_without_extension() {
        let output_dir = Path::new("mutants_output_no_extension");
        let filename = Path::new("file1");
        let mutant_id = 0;

        fs::File::create(filename).unwrap();
        let result = setup_mutant_path(output_dir, filename, mutant_id);
        fs::remove_file(filename).unwrap();
        fs::remove_dir_all(output_dir).unwrap();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PathBuf::from(format!(
                "mutants_output_no_extension/file1_mutant_{mutant_id}.move"
            ))
        );
    }

    #[test]
    fn setup_mutant_path_creates_correct_path() {
        let output_dir = Path::new("mutants_output_correct");
        let filename = Path::new("test_correct");
        let mutant_id = 0;

        fs::File::create(filename).unwrap();
        let result = setup_mutant_path(output_dir, filename, mutant_id);
        fs::remove_file(filename).unwrap();
        fs::remove_dir_all(output_dir).unwrap();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PathBuf::from(format!(
                "mutants_output_correct/test_correct_mutant_{mutant_id}.move"
            ))
        );
    }

    #[test]
    fn setup_mutant_path_handles_empty_output_dir() {
        let output_dir = Path::new("");
        let filename = Path::new("test_empty");
        let mutant_id = 0;

        fs::File::create(filename).unwrap();
        let result = setup_mutant_path(output_dir, filename, mutant_id);
        fs::remove_file(filename).unwrap();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PathBuf::from(format!("test_empty_mutant_{mutant_id}.move"))
        );
    }

    #[test]
    fn setup_mutant_path_handles_empty_filename() {
        let output_dir = Path::new("mutants_output_empty_filename");
        let filename = Path::new("");
        let mutant_id = 0;

        let result = setup_mutant_path(output_dir, filename, mutant_id);
        assert!(result.is_err());
    }

    #[test]
    fn setup_output_dir_creates_directory_if_not_exists() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        let options = cli::CLIOptions {
            out_mutant_dir: Some(output_dir.clone()),
            no_overwrite: false,
            ..Default::default()
        };
        let config = Configuration::new(options, None);
        assert!(setup_output_dir(&config).is_ok());
        assert!(output_dir.exists());
    }

    #[test]
    fn setup_output_dir_overwrites_directory_if_exists_and_no_overwrite_is_false() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();
        let options = cli::CLIOptions {
            out_mutant_dir: Some(output_dir.clone()),
            no_overwrite: false,
            ..Default::default()
        };
        let config = Configuration::new(options, None);
        assert!(setup_output_dir(&config).is_ok());
        assert!(output_dir.exists());
    }

    #[test]
    fn setup_output_dir_errors_if_directory_exists_and_no_overwrite_is_true() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();
        let options = cli::CLIOptions {
            out_mutant_dir: Some(output_dir.clone()),
            no_overwrite: true,
            ..Default::default()
        };
        let config = Configuration::new(options, None);
        assert!(setup_output_dir(&config).is_err());
    }
}
