use clap::{Arg, ArgAction, Command};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use workflow_types::{WorkflowAction, WorkflowConfig, WorkflowStatus, WorkflowStep};

// Error type for NextShell operations
#[derive(Debug)]
enum NextShellError {
    IoError(io::Error),
    SerializationError(serde_json::Error),
    ValidationError(String),
    ExecutionError(String),
    WorkflowNotFound(String),
}

impl From<io::Error> for NextShellError {
    fn from(error: io::Error) -> Self {
        NextShellError::IoError(error)
    }
}

impl From<serde_json::Error> for NextShellError {
    fn from(error: serde_json::Error) -> Self {
        NextShellError::SerializationError(error)
    }
}

impl std::fmt::Display for NextShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NextShellError::IoError(e) => write!(f, "I/O error: {}", e),
            NextShellError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            NextShellError::ValidationError(e) => write!(f, "Validation error: {}", e),
            NextShellError::ExecutionError(e) => write!(f, "Execution error: {}", e),
            NextShellError::WorkflowNotFound(e) => write!(f, "Workflow not found: {}", e),
        }
    }
}

// Represents an indexed workflow with metadata
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct IndexedWorkflow {
    id: String,
    name: String,
    path: PathBuf,
    tags: Vec<String>,
    last_executed: Option<u64>,
    status: Option<WorkflowStatus>,
}

// Represents the workflow index
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct WorkflowIndex {
    workflows: HashMap<String, IndexedWorkflow>,
}

impl WorkflowIndex {
    // Load the workflow index from disk or create a new one if it doesn't exist
    fn load() -> Result<Self, NextShellError> {
        let index_path = get_index_path();
        
        if !index_path.exists() {
            return Ok(WorkflowIndex::default());
        }

        let mut file = File::open(index_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let index = serde_json::from_str(&contents)?;
        Ok(index)
    }

    // Save the workflow index to disk
    fn save(&self) -> Result<(), NextShellError> {
        let index_path = get_index_path();
        
        // Ensure the directory exists
        if let Some(parent) = index_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let contents = serde_json::to_string_pretty(self)?;
        let mut file = File::create(index_path)?;
        file.write_all(contents.as_bytes())?;
        
        Ok(())
    }

    // Add a workflow to the index
    fn add_workflow(&mut self, path: &Path) -> Result<(), NextShellError> {
        let content = fs::read_to_string(path)?;
        let config: WorkflowConfig = serde_json::from_str(&content)?;
        
        // Validate the workflow
        if let Err(e) = config.validate() {
            return Err(NextShellError::ValidationError(e));
        }
        
        // Generate a unique ID based on the filename
        let id = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Use the description as the name or fallback to ID
        let name = if !config.description.is_empty() {
            config.description.clone()
        } else {
            id.clone()
        };
        
        // Add to index
        self.workflows.insert(id.clone(), IndexedWorkflow {
            id,
            name,
            path: path.to_path_buf(),
            tags: config.tags,
            last_executed: None,
            status: None,
        });
        
        self.save()?;
        Ok(())
    }
    
    // Update workflow status
    fn update_status(&mut self, id: &str, status: WorkflowStatus) -> Result<(), NextShellError> {
        if let Some(workflow) = self.workflows.get_mut(id) {
            workflow.status = Some(status);
            
            if status == WorkflowStatus::Running {
                // Update last executed timestamp
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                workflow.last_executed = Some(timestamp);
            }
            
            self.save()?;
            Ok(())
        } else {
            Err(NextShellError::WorkflowNotFound(format!("Workflow with ID '{}' not found", id)))
        }
    }
    
    // Get a workflow by ID
    fn get_workflow(&self, id: &str) -> Option<&IndexedWorkflow> {
        self.workflows.get(id)
    }
    
    // List all workflows
    fn list_workflows(&self) -> Vec<&IndexedWorkflow> {
        self.workflows.values().collect()
    }
}

// Get the path to the workflow index file
fn get_index_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("nextshell");
    path.push("workflow_index.json");
    path
}

// Get the default workflows directory
fn get_workflows_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("nextshell");
    path.push("workflows");
    path
}

// Index all workflows in a directory
fn index_workflows(dir: &Path) -> Result<WorkflowIndex, NextShellError> {
    let mut index = WorkflowIndex::load()?;
    
    // Create directory if it doesn't exist
    if !dir.exists() {
        fs::create_dir_all(dir)?;
        return Ok(index);
    }
    
    // Process each JSON file in the directory
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            match index.add_workflow(&path) {
                Ok(_) => println!("Indexed workflow: {:?}", path.file_name().unwrap_or_default()),
                Err(e) => eprintln!("Failed to index workflow {:?}: {}", path, e),
            }
        }
    }
    
    index.save()?;
    Ok(index)
}

// Main workflow execution engine
struct WorkflowEngine;

impl WorkflowEngine {
    // Execute a workflow by ID
    fn execute_workflow(id: &str, verbose: bool) -> Result<(), NextShellError> {
        let mut index = WorkflowIndex::load()?;
        
        // Get the workflow from the index
        let workflow = index.get_workflow(id)
            .ok_or_else(|| NextShellError::WorkflowNotFound(format!("Workflow with ID '{}' not found", id)))?;
        
        // Load the workflow config
        let content = fs::read_to_string(&workflow.path)?;
        let config: WorkflowConfig = serde_json::from_str(&content)?;
        
        // Update status to Running
        index.update_status(id, WorkflowStatus::Running)?;
        
        println!("Executing workflow: {}", workflow.name);
        
        // Track if any steps failed
        let mut success = true;
        
        // Execute each step
        for step in &config.steps {
            println!("Step: {}", step.name);
            if verbose {
                println!("  Description: {}", step.description);
            }
            
            // Execute each action in the step
            for (i, action) in step.actions.iter().enumerate() {
                if verbose {
                    println!("  Action {}: {}", i + 1, action.command);
                    if !action.args.is_empty() {
                        println!("    Args: {:?}", action.args);
                    }
                }
                
                // Execute the action
                let result = Self::execute_action(action, verbose);
                
                match result {
                    Ok(exit_code) => {
                        if exit_code != 0 {
                            eprintln!("  Action failed with exit code: {}", exit_code);
                            
                            // If step is required, mark workflow as failed
                            if step.required {
                                success = false;
                                break;
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("  Failed to execute action: {}", e);
                        
                        // If step is required, mark workflow as failed
                        if step.required {
                            success = false;
                            break;
                        }
                    }
                }
            }
            
            // If a required step failed, stop execution
            if !success {
                break;
            }
        }
        
        // Update final status
        let final_status = if success {
            WorkflowStatus::Completed
        } else {
            WorkflowStatus::Failed
        };
        
        index.update_status(id, final_status)?;
        
        println!("Workflow execution {}", if success { "completed successfully" } else { "failed" });
        Ok(())
    }
    
    // Execute a single action
    fn execute_action(action: &WorkflowAction, verbose: bool) -> Result<i32, NextShellError> {
        let mut cmd = ProcessCommand::new(&action.command);
        
        // Add arguments
        if !action.args.is_empty() {
            cmd.args(&action.args);
        }
        
        // Add environment variables
        for (key, value) in &action.env {
            cmd.env(key, value);
        }
        
        // Configure stdio
        if verbose {
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
        }
        
        // Execute the command
        let output = cmd.output()
            .map_err(|e| NextShellError::ExecutionError(format!("Failed to execute command: {}", e)))?;
        
        Ok(output.status.code().unwrap_or(-1))
    }
    
    // Validate a workflow
    fn validate_workflow(id: &str) -> Result<(), NextShellError> {
        let index = WorkflowIndex::load()?;
        
        // Get the workflow from the index
        let workflow = index.get_workflow(id)
            .ok_or_else(|| NextShellError::WorkflowNotFound(format!("Workflow with ID '{}' not found", id)))?;
        
        // Load the workflow config
        let content = fs::read_to_string(&workflow.path)?;
        let config: WorkflowConfig = serde_json::from_str(&content)?;
        
        // Validate the workflow
        if let Err(e) = config.validate() {
            return Err(NextShellError::ValidationError(e));
        }
        
        println!("Workflow '{}' is valid", workflow.name);
        Ok(())
    }
    
    // Show workflow status
    fn show_workflow_status(id: &str) -> Result<(), NextShellError> {
        let index = WorkflowIndex::load()?;
        
        // Get the workflow from the index
        let workflow = index.get_workflow(id)
            .ok_or_else(|| NextShellError::WorkflowNotFound(format!("Workflow with ID '{}' not found", id)))?;
        
        println!("Workflow: {}", workflow.name);
        println!("ID: {}", workflow.id);
        println!("Path: {:?}", workflow.path);
        
        if !workflow.tags.is_empty() {
            println!("Tags: {}", workflow.tags.join(", "));
        }
        
        if let Some(timestamp) = workflow.last_executed {
            // Convert timestamp to readable date
            let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
                
            println!("Last executed: {}", datetime);
        } else {
            println!("Last executed: Never");
        }
        
        println!("Status: {}", match &workflow.status {
            Some(status) => format!("{:?}", status),
            None => "Unknown".to_string(),
        });
        
        Ok(())
    }
    
    // List all workflows
    fn list_workflows(verbose: bool) -> Result<(), NextShellError> {
        let index = WorkflowIndex::load()?;
        let workflows = index.list_workflows();
        
        if workflows.is_empty() {
            println!("No workflows found. Use 'index' command to index workflows.");
            return Ok(());
        }
        
        println!("Available workflows:");
        println!("-------------------");
        
        for workflow in workflows {
            println!("{}: {}", workflow.id, workflow.name);
            
            if verbose {
                if !workflow.tags.is_empty() {
                    println!("  Tags: {}", workflow.tags.join(", "));
                }
                
                
                if let Some(status) = &workflow.status {
                    println!("  Status: {:?}", status);
                }
                
                if let Some(timestamp) = workflow.last_executed {
                    // Convert timestamp to readable date
                    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                        
                    println!("  Last executed: {}", datetime);
                }
                
                println!("  Path: {:?}", workflow.path);
                println!();
            }
        }
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up CLI with clap
    let matches = Command::new("nextshell")
        .version("0.1.0")
        .author("NextShell Team")
        .about("Workflow automation tool")
        .subcommand(
            Command::new("list")
                .about("List all workflows")
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show detailed information")
                        .action(ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("execute")
                .about("Execute a workflow")
                .arg(
                    Arg::new("id")
                        .help("Workflow ID to execute")
                        .required(true)
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show detailed execution output")
                        .action(ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("status")
                .about("Show workflow status")
                .arg(
                    Arg::new("id")
                        .help("Workflow ID to check")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("validate")
                .about("Validate a workflow")
                .arg(
                    Arg::new("id")
                        .help("Workflow ID to validate")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("index")
                .about("Index workflows in directory")
                .arg(
                    Arg::new("directory")
                        .short('d')
                        .long("directory")
                        .help("Directory containing workflow files")
                        .default_value(get_workflows_dir().to_str().unwrap_or("."))
                )
        )
        .get_matches();

    // Handle commands
    match matches.subcommand() {
        Some(("list", sub_matches)) => {
            let verbose = sub_matches.get_flag("verbose");
            match WorkflowEngine::list_workflows(verbose) {
                Ok(_) => (),
                Err(e) => eprintln!("Error: {}", e),
            }
        },
        Some(("execute", sub_matches)) => {
            let id = sub_matches.get_one::<String>("id").unwrap();
            let verbose = sub_matches.get_flag("verbose");
            match WorkflowEngine::execute_workflow(id, verbose) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error executing workflow: {}", e);
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
                }
            }
        },
        Some(("status", sub_matches)) => {
            let id = sub_matches.get_one::<String>("id").unwrap();
            match WorkflowEngine::show_workflow_status(id) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error getting workflow status: {}", e);
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
                }
            }
        },
        Some(("validate", sub_matches)) => {
            let id = sub_matches.get_one::<String>("id").unwrap();
            match WorkflowEngine::validate_workflow(id) {
                Ok(_) => println!("Workflow is valid"),
                Err(e) => {
                    eprintln!("Error validating workflow: {}", e);
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
                }
            }
        },
        Some(("index", sub_matches)) => {
            let dir_str = sub_matches.get_one::<String>("directory").unwrap();
            let dir = Path::new(dir_str);
            
            println!("Indexing workflows in: {:?}", dir);
            match index_workflows(dir) {
                Ok(index) => {
                    let count = index.list_workflows().len();
                    println!("Successfully indexed {} workflows", count);
                },
                Err(e) => {
                    eprintln!("Error indexing workflows: {}", e);
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
                }
            }
        },
        _ => {
            // No subcommand provided, print help
            println!("No command specified.\n");
            println!("Available commands:");
            println!("  list     - List all workflows");
            println!("  execute  - Execute a workflow");
            println!("  status   - Show workflow status");
            println!("  validate - Validate a workflow");
            println!("  index    - Index workflows in directory\n");
            println!("Use --help with any command for more information.");
        }
    }

    Ok(())
}
