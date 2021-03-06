use crate::error::Error;
use crate::error::Error::InternalError;
use crate::git;
use crate::project;
use crate::snippet;

use std::process::Command;

use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path;

#[derive(Debug)]
pub enum OpCode<'a> {
    // For the new snippet command
    NewSnippet(&'a project::SnippetLocation),
    // For the add snippet command
    AddSnippet(String, &'a project::SnippetLocation),
    // For listing snippets
    ListSnippets(bool),
    // For syncing snippets with the server
    PullSnippets,
    // Save snippets to repo
    SaveSnippets,
}

/// Find the snippets associated with the project
pub fn find_snippets(project: &project::Project) -> Result<Vec<fs::DirEntry>, Error> {
    // Crawl through directory that is set as project root
    let mut res: Vec<fs::DirEntry> = Vec::new();

    // Read the entries in the folder
    for snippet_location in project.locations.iter() {
        println!("Finding snippets in {},", &snippet_location.local.as_str());
        let mut entries: Vec<fs::DirEntry> = fs::read_dir(&snippet_location.local)?
            .filter_map(|x| x.ok())
            .collect();

        // For each of the entries
        let mut entries: Vec<_> = entries
            .into_iter()
            .filter_map(|e| {
                let dir_ent = e;

                // Get the path
                let path = dir_ent.path();
                // Get the extension
                let ext_opt = path.extension();
                if let Some(ext) = ext_opt {
                    if let Some(s) = ext.to_str() {
                        // Add to list if files match extension
                        if s == snippet_location.ext {
                            return Some(dir_ent);
                        }
                    }
                }
                return None;
            })
            .collect();
        res.append(&mut entries);
    }
    Ok(res)
}

/// Load snippets from the dir entries
pub fn load_snippets(
    dir_entries: &Vec<fs::DirEntry>,
    keywords: &Vec<String>,
) -> Result<Vec<snippet::Snippet>, Error> {
    let keyword_slice = keywords.as_slice();

    // Get all tags for entries
    let mut tag_with_entries: Vec<(u32, &fs::DirEntry, Vec<String>)> = Vec::new();
    for entry in dir_entries {
        // Read the tags
        let tags = snippet::read_tags(entry.path().to_str().unwrap())?;

        // If tag is in the snippet, or no tags are given
        // Filter which don't contain the keyword
        let tag_count : u32 = tags.iter()
            .fold(0, |x, tag| x + if keyword_slice.contains(tag) { 1 } else { 0 });
        if keyword_slice.is_empty() || tag_count > 0 {
            tag_with_entries.push((tag_count, entry, snippet::read_tags(entry.path().to_str().unwrap())?));
        }
    }

    // Sort by number of matched tags
    tag_with_entries.sort_by(|a, b| b.0.cmp(&a.0) );

    // This maps the files and tags, to a snippet
    let result = tag_with_entries
        .iter()
        .map(|(count, entry, tags)| {
            snippet::Snippet::new(entry.path().to_str().unwrap().to_string(), &tags)
        })
        .collect();

    Ok(result)
}

//// Edit snippets
pub fn edit_snippet(program: &str, full_path: &path::Path) -> Result<(), Error> {
    let final_editor = default_editor(program);
    let _output = Command::new(final_editor)
        .arg(&full_path)
        .spawn()?
        .wait_with_output()?;

    Ok(())
}

/// New snippet
pub fn new_snippet(program: &str, working_dir: &path::Path) -> Result<(), Error> {
    let final_editor = default_editor(program);

    let _output = Command::new(final_editor)
        .current_dir(&working_dir)
        .spawn()?
        .wait_with_output()?;

    Ok(())
}

fn default_editor(program: &str) -> String {
    let final_editor: String;
    if let Ok(editor) = env::var("EDITOR") {
        final_editor = editor.into();
    } else {
        final_editor = program.into()
    };
    final_editor
}

//// Start the different operation modes
pub fn start_operation(
    code: &OpCode,
    project: &project::Project,
    keywords: Vec<String>,
) -> Result<Vec<snippet::Snippet>, Error> {
    // Match on operation
    let result = match code {
        OpCode::AddSnippet(new_file, location) => {
            // Create the full path
            let full_path = path::Path::new(&location.local).join(new_file);
            // Create the file
            if full_path.exists() {
                return Err(InternalError("Snippet already exists".to_string()));
            }
            let mut file = File::create(&full_path)?;

            // Write the keywords to the file
            for keyword in &keywords {
                file.write(keyword.as_bytes())?;
                file.write(b",")?;
            }

            // Open vim on location
            edit_snippet("vim", &full_path)?;

            let snippet =
                snippet::Snippet::new(full_path.into_os_string().into_string().unwrap(), &keywords);
            Ok(vec![snippet])
        }

        // Add a new snippet
        OpCode::NewSnippet(location) => {
            let path = path::Path::new(&location.local);

            new_snippet("vim", path)?;
            Ok(vec![])
        }

        // List snippets
        OpCode::ListSnippets(_) => {
            let files = find_snippets(&project)?;
            let snippets = load_snippets(&files, &keywords)?;

            Ok(snippets)
        }

        // Sync snippets
        OpCode::PullSnippets => {
            println!("Pulling snippet locations...");
            for location in &project.locations {
                // Only sync if it is a git location
                if location.git == Some(true) {
                    git::git_pull(location)?;
                }
            }
            Ok(vec![])
        }

        // Sync snippets
        OpCode::SaveSnippets => {
            println!("Saving snippets...");
            for location in &project.locations {
                // Only sync if it is a git location
                if location.git == Some(true) {
                    git::determine_git_modified_status(location).and_then(|s| {
                        if let git::GitStatus::Modified = s {
                            println!("Enter your commit message: ");
                            let mut msg = String::new();
                            io::stdin().read_line(&mut msg)?;
                            // Add
                            git::git_add(location)?;
                            // Commit
                            git::git_commit(location, msg)?;
                            // Push
                            git::git_push(location)?;
                            Ok(())
                        } else {
                            // Push to make sure for unpushed changes, TODO change this later to use rev-parse
                            git::git_push(location)?;

                            Ok(())
                        }
                    })?;
                };
            }
            Ok(vec![])
        }
    };
    result
}
