use crate::error;
use crate::git;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path;

/// Location of the snippets
#[derive(Serialize, Deserialize, Debug)]
pub struct SnippetLocation {
    pub local: String,
    pub ext: String,
    pub git: Option<bool>,
}

impl SnippetLocation {
    pub fn default(home: &String) -> SnippetLocation {
        return SnippetLocation {
            local: String::from(home.to_owned() + "/.snippets"),
            ext: "md".to_string(),
            git: None,
        };
    }

    /// Create the folder of the SnippetLocation if it does not exist
    pub fn create_if_not_exists(&self) -> Result<(), error::Error> {
        let path = path::Path::new(&self.local);
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(())
    }
}

/// Project folder structure
#[derive(Serialize, Deserialize)]
pub struct Project {
    pub locations: Vec<SnippetLocation>,
}

pub enum ProjectOperation {
    Exist(Project),
    NotExist(Project),
}

impl Project {
    /// Write a project
    pub fn write(&self, folder: &path::Path) -> Result<(), error::Error> {
        let to_write = toml::to_string(self).expect("Cannot serialize project");
        let full_path = path::Path::new(&folder).join(".x.toml");
        let mut f = File::create(&full_path).expect("Cannot create project file");
        f.write_all(to_write.as_bytes())
            .expect("Cannot write to project file");
        Ok(())
    }
    /// Get the default project location
    pub fn default_project() -> Result<ProjectOperation, error::Error> {
        let home = String::from(
            dirs::home_dir()
                .expect("Cannot find the home dir")
                .to_str()
                .unwrap(),
        );

        let format = format!("{}/.x.toml", &home);
        let path = path::Path::new(&format);

        // If exists than deserialize toml
        let mut project_operation = if path.exists() {
            // Read the file
            let mut f = File::open(path).expect("Found project file but can't read it.");
            let mut buffer = String::new();
            f.read_to_string(&mut buffer)?;

            // Deserialize the toml
            let project: Project =
                toml::from_str(&buffer).expect("Cannot deserialize project file");
            ProjectOperation::Exist(project)
        } else {
            ProjectOperation::NotExist(Project {
                locations: vec![SnippetLocation::default(&home)],
            })
        };

        // Determine git status
        if let ProjectOperation::Exist(ref mut project) = project_operation {
            // Determine the git status
            git::determine_git_status(project);
        }

        Ok(project_operation)
    }
}
