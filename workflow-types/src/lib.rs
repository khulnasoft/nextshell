use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WorkflowStatus represents the current state of a workflow execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is ready to be executed
    Ready,
    /// Workflow is currently executing
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed to complete
    Failed,
    /// Workflow execution was paused
    Paused,
    /// Workflow was canceled before completion
    Canceled,
}

impl Default for WorkflowStatus {
    fn default() -> Self {
        Self::Ready
    }
}

/// WorkflowAction represents a single command to be executed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowAction {
    /// The command to execute
    pub command: String,
    /// Optional arguments for the command
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

impl WorkflowAction {
    /// Create a new WorkflowAction with a command
    pub fn new(command: impl Into<String>) -> Self {
        WorkflowAction {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
        }
    }

    /// Add arguments to the action
    pub fn with_args(mut self, args: Vec<impl Into<String>>) -> Self {
        self.args = args.into_iter().map(|arg| arg.into()).collect();
        self
    }

    /// Add environment variables to the action
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Validate that the action has a valid command
    pub fn validate(&self) -> Result<(), String> {
        if self.command.is_empty() {
            return Err("Action command cannot be empty".to_string());
        }
        Ok(())
    }
}

/// WorkflowStep represents a single step in a workflow
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Unique identifier for the step
    pub id: String,
    /// Human-readable name for the step
    pub name: String,
    /// Detailed description of what the step does
    #[serde(default)]
    pub description: String,
    /// Actions to be executed in this step
    pub actions: Vec<WorkflowAction>,
    /// Whether this step is required to complete the workflow
    #[serde(default = "default_true")]
    pub required: bool,
    /// Step dependencies - IDs of steps that must be completed before this one
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl WorkflowStep {
    /// Create a new WorkflowStep with required fields
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        WorkflowStep {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            actions: Vec::new(),
            required: true,
            depends_on: Vec::new(),
        }
    }

    /// Add a description to the step
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add actions to the step
    pub fn with_actions(mut self, actions: Vec<WorkflowAction>) -> Self {
        self.actions = actions;
        self
    }

    /// Set whether the step is required
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Add dependencies to the step
    pub fn with_dependencies(mut self, dependencies: Vec<impl Into<String>>) -> Self {
        self.depends_on = dependencies.into_iter().map(|dep| dep.into()).collect();
        self
    }

    /// Validate the step configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Step ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Step name cannot be empty".to_string());
        }
        if self.actions.is_empty() {
            return Err(format!("Step '{}' must have at least one action", self.id));
        }
        
        // Validate all actions
        for (i, action) in self.actions.iter().enumerate() {
            if let Err(e) = action.validate() {
                return Err(format!("Invalid action {} in step '{}': {}", i + 1, self.id, e));
            }
        }
        
        Ok(())
    }
}

/// WorkflowConfig represents the entire workflow configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Version of the workflow
    pub version: String,
    /// Author of the workflow
    pub author: String,
    /// Description of what the workflow does
    pub description: String,
    /// Steps that make up the workflow
    #[serde(default)]
    pub steps: Vec<WorkflowStep>,
    /// Whether this workflow is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Any tags associated with this workflow for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl WorkflowConfig {
    /// Create a new WorkflowConfig with required fields
    pub fn new(
        version: impl Into<String>,
        author: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        WorkflowConfig {
            version: version.into(),
            author: author.into(),
            description: description.into(),
            steps: Vec::new(),
            enabled: true,
            tags: Vec::new(),
        }
    }

    /// Add steps to the workflow
    pub fn with_steps(mut self, steps: Vec<WorkflowStep>) -> Self {
        self.steps = steps;
        self
    }

    /// Add tags to the workflow
    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|tag| tag.into()).collect();
        self
    }

    /// Set whether the workflow is enabled
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Validate the entire workflow configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.version.is_empty() {
            return Err("Workflow version cannot be empty".to_string());
        }
        if self.author.is_empty() {
            return Err("Workflow author cannot be empty".to_string());
        }
        if self.description.is_empty() {
            return Err("Workflow description cannot be empty".to_string());
        }
        if self.steps.is_empty() {
            return Err("Workflow must have at least one step".to_string());
        }

        // Validate all steps
        for step in &self.steps {
            if let Err(e) = step.validate() {
                return Err(format!("Invalid step: {}", e));
            }
        }

        // Validate step dependencies
        let step_ids: Vec<&String> = self.steps.iter().map(|step| &step.id).collect();
        for step in &self.steps {
            for dep_id in &step.depends_on {
                if !step_ids.contains(&&dep_id) {
                    return Err(format!(
                        "Step '{}' depends on non-existent step '{}'",
                        step.id, dep_id
                    ));
                }
            }
        }

        // Check for circular dependencies
        self.check_circular_dependencies()?;

        Ok(())
    }

    /// Check for circular dependencies in the workflow steps
    fn check_circular_dependencies(&self) -> Result<(), String> {
        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();

        for step in &self.steps {
            if self.is_cyclic(&step.id, &mut visited, &mut rec_stack)? {
                return Err(format!("Circular dependency detected involving step '{}'", step.id));
            }
        }

        Ok(())
    }

    fn is_cyclic(
        &self,
        step_id: &str,
        visited: &mut HashMap<String, bool>,
        rec_stack: &mut HashMap<String, bool>,
    ) -> Result<bool, String> {
        // If not visited, mark as visited
        if !visited.contains_key(step_id) {
            visited.insert(step_id.to_string(), true);
            rec_stack.insert(step_id.to_string(), true);

            // Find the step
            let step = self
                .steps
                .iter()
                .find(|s| s.id == step_id)
                .ok_or_else(|| format!("Step '{}' not found", step_id))?;

            // Check all dependencies
            for dep_id in &step.depends_on {
                if !visited.contains_key(dep_id) && self.is_cyclic(dep_id, visited, rec_stack)? {
                    return Ok(true);
                } else if rec_stack.get(dep_id).unwrap_or(&false) == &true {
                    return Ok(true);
                }
            }
        }

        // Remove from recursion stack
        rec_stack.insert(step_id.to_string(), false);
        Ok(false)
    }
}

/// Get the current version of the workflow-types crate
pub fn get_version() -> &'static str {
    "0.1.0"
}
