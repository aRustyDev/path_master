// find all matching dirs /etc/*paths.d
// for each dir load the dir contents in order
// join each files line contents with seperator(default=":")
// join each file string with seperator(default=":")
// use clap::builder::Str;
// use infer::Type;
use regex::Regex;
use std::{
    env, fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

pub const ROOTD: &str = r"/etc/";
pub const PATHSD: &str = r"(?<env>.*?)paths.d$";

// Represents each /etc/*paths.d/ directory, including
// - Directory Path
// - ENV VAR key
// - PATH String Value
#[derive(Debug)]
pub struct Pathd {
    pub dir: PathBuf,
    pub key: String,
    pub paths: Vec<String>,
}

impl Pathd {
    fn new(path: &PathBuf) -> Self {
        Pathd {
            dir: path.clone(),
            key: envkey(path).unwrap(),
            paths: Vec::new(),
        }
    }
    // Checks if path is empty dir
    // Path MUST exist
    pub fn is_valid(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Q: How does this handle passing errors from the '?' do they get wrapped in an Ok?
        Ok(self.exists()? && self.is_dir()? && !self.is_empty()?)
    }
    // Checks if path is empty dir
    // Path MUST exist
    fn is_empty(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut entries = fs::read_dir(self.dir.clone())?;
        Ok(entries.next().is_none())
        // return Err(io::Error::new(
        //     io::ErrorKind::NotFound,
        //     format!("Directory is empty: {}", path.display()),
        // )
        // .into());
    }
    // Path MUST be a directory
    fn is_dir(&self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.dir.is_dir())
        // return Err(io::Error::new(
        //     io::ErrorKind::NotADirectory,
        //     format!("Path is not a directory: {}", path.display()),
        // )
        // .into())
    }
    fn exists(&self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.dir.exists())
        // return Err(io::Error::new(
        //     io::ErrorKind::NotFound,
        //     format!("Path does not exist: {}", path.display()),
        // )
        // .into())
    }
    fn add_path(&mut self, path: &String) {
        let invalid_chars = ['\0', '?', '<', '>', ':', '|', '*', '"', '\\'];
        // 1. expand env vars
        let expanded_path = expand_env_vars(path);
        let trimmed_line = path.trim();
        // 2. check is valid path (no bad characters)
        if !path
            .chars()
            .any(|c| invalid_chars.contains(&c) || c < '\x20')
        {
            // println!("--line: {:?}", path);
            // Keep only valid lines: non-empty after trimming and not starting with '#'
            if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') {
                self.paths.push(expanded_path); // Push the expanded line (before trimming)
            }
        }
    }
    // Return the PATH:JOINED:STRING
    pub fn path_string(&self) -> Option<String> {
        if !self.paths.is_empty() {
            Some(self.paths.join(":"))
        } else {
            None
        }
    }
    // SHOULD: run checks & append file to Pathd.paths
    pub fn get_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        for file in fs::read_dir(self.dir.clone())? {
            let file = file?;
            let path = file.path();
            if is_text_file(&path)? {
                files.push(path);
            }
        }
        files.sort();
        for path in files {
            for line in get_contents(&path)? {
                self.add_path(&line);
            }
        }
        Ok(())
    }
}

// // Find all Path.d directories
// fn find_pathd() -> Vec<Pathd> {}

// Extracts <*> key from directory name, defaults to PATH
pub fn envkey(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    // Compile the regex
    let re = Regex::new(PATHSD)?;
    // Convert PathBuf -> str
    let pathstr = path.to_str().unwrap();
    // Remove the ROOTD string
    let stripped = pathstr.strip_prefix(ROOTD).unwrap();
    // Trigger Capture Groups
    let caps = re.captures(stripped).unwrap();
    // If Capture string is empty return "PATH" else return the ENV key
    if (&caps["env"]).to_string().is_empty() {
        Ok("PATH".to_string())
    } else {
        Ok((&caps["env"]).to_string())
    }
}

// Uses heuristics to determine if a file is a text file or not
fn is_text_file(path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
    if !path.is_file() {
        return Ok(false);
    } else {
        let contents = fs::read(path)?; // Read entire file into bytes
        Ok(str::from_utf8(&contents).is_ok()) // Check if it's valid UTF-8
    }
}

// Returns the contents of a given file as a vector of each line in the file
fn get_contents(path: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut lines = Vec::new();
    let file = fs::File::open(&path)?;
    let reader = io::BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result?; // Handle potential errors during line reading
        lines.push(line);
    }
    Ok(lines)
}

/// Expands environment variables in a given string.
/// It supports both `$VAR` and `${VAR}` syntax.
/// TODO: Support advanced syntax like "${XDG_CONFIG_HOME:-$HOME/.config}/foo"
fn expand_env_vars(line: &String) -> String {
    // Regex to match $VAR or ${VAR} patterns
    let re = Regex::new(r"\$(?P<name>[a-zA-Z_][a-zA-Z0-9_]*|\{[a-zA-Z_][a-zA-Z0-9_]*\})").unwrap();
    let mut result = String::new();
    let mut last_match_end = 0;

    for mat in re.find_iter(line.as_str()) {
        let matched_text = mat.as_str();
        // Determine the actual variable name, stripping '$' or '${...}'
        let var_name = if matched_text.starts_with("${") && matched_text.ends_with('}') {
            &matched_text[2..matched_text.len() - 1] // Remove "${" and "}"
        } else {
            &matched_text[1..] // Remove "$"
        };

        // Append the part of the string before the current match
        result.push_str(&line[last_match_end..mat.start()]);

        // Try to get the environment variable's value
        if let Ok(value) = env::var(var_name) {
            result.push_str(&value); // Append the expanded value
        } else {
            result.push_str(matched_text); // If not found, keep the original variable string
        }
        last_match_end = mat.end();
    }

    // Append any remaining part of the string after the last match
    result.push_str(&line[last_match_end..]);
    result
}

/// Finds directories whose full paths match the given regular expression pattern.
///
/// # Arguments
/// * `start_path` - The starting directory from which to begin the search.
/// * `regex_pattern` - The regular expression string to match against directory paths.
///
/// # Returns
/// A `Result` containing a `Vec<PathBuf>` of matching directory paths,
/// or an error if the regex pattern is invalid or directory traversal fails.
///
/// # Errors
/// Returns `regex::Error` if the `regex_pattern` is invalid.
/// Returns `io::Error` if there's an issue during directory traversal (e.g., permissions).
///
/// # Examples
/// ```
/// use std::fs;
/// use std::path::PathBuf;
///
/// // Create some dummy directories for testing
/// let _ = fs::create_dir_all("test_search/project_alpha/src");
/// let _ = fs::create_dir_all("test_search/project_beta/target");
/// let _ = fs::create_dir_all("test_search/logs/2023_archive");
/// let _ = fs::create_dir_all("test_search/logs/2024_current");
/// let _ = fs::File::create("test_search/logs/file.txt"); // A file, not a directory
///
/// // Find directories containing "project" in their path
/// let projects = find_matching_directories_regex(
///     &PathBuf::from("test_search"),
///     r".*project.*"
/// )?;
/// assert_eq!(projects.len(), 2); // project_alpha and project_beta
///
/// // Find directories whose name ends with "_archive"
/// let archives = find_matching_directories_regex(
///     &PathBuf::from("test_search"),
///     r".*_archive$"
/// )?;
/// assert_eq!(archives.len(), 1);
/// assert!(archives.contains(&PathBuf::from("test_search/logs/2023_archive")));
///
/// // Clean up dummy directories
/// let _ = fs::remove_dir_all("test_search");
/// ```
pub fn get_paths_dirs_regex(
    start_path: &Path,
    regex_pattern: &str,
) -> Result<Vec<Pathd>, Box<dyn std::error::Error>> {
    // Compile the regular expression
    let re = Regex::new(regex_pattern)?;
    let mut pathds = Vec::new();

    // Iterate only through direct entries in start_path
    for entry_result in fs::read_dir(start_path)? {
        let entry = entry_result?;
        let path = entry.path(); // Full path of the entry

        // Check if the entry is a directory
        if path.is_dir() {
            // Get the directory's name (last component of the path)
            if let Some(dir_name_os_str) = path.file_name() {
                if let Some(dir_name_str) = dir_name_os_str.to_str() {
                    // Match the regex against the directory name
                    if re.is_match(dir_name_str) {
                        let pathd = Pathd::new(&path.to_path_buf());
                        if pathd.is_valid()? {
                            // For some reason this will print the directory name to stdout
                            pathds.push(pathd);
                        }
                    }
                } else {
                    eprintln!("Warning: Skipping non-UTF-8 directory name: {:?}", path);
                }
            }
        }
    }

    Ok(pathds)
}
