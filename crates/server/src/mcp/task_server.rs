use std::{future::Future, path::PathBuf, str::FromStr};

use db::models::{
    project::Project,
    task::{CreateTask, Task, TaskStatus, TaskWithAttemptStatus, UpdateTask},
    task_attempt::TaskAttempt,
};
use executors::{executors::BaseCodingAgent, profile::ExecutorProfileId};
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::tool::{Parameters, ToolRouter},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json;
use uuid::Uuid;

use crate::routes::task_attempts::CreateTaskAttemptBody;

use super::advanced_tools::*;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTaskRequest {
    #[schemars(description = "The ID of the project to create the task in. This is required!")]
    pub project_id: Uuid,
    #[schemars(description = "The title of the task")]
    pub title: String,
    #[schemars(description = "Optional description of the task")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CreateTaskResponse {
    pub task_id: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ProjectSummary {
    #[schemars(description = "The unique identifier of the project")]
    pub id: String,
    #[schemars(description = "The name of the project")]
    pub name: String,
    #[schemars(description = "The path to the git repository")]
    pub git_repo_path: PathBuf,
    #[schemars(description = "Optional setup script for the project")]
    pub setup_script: Option<String>,
    #[schemars(description = "Optional cleanup script for the project")]
    pub cleanup_script: Option<String>,
    #[schemars(description = "Optional development script for the project")]
    pub dev_script: Option<String>,
    #[schemars(description = "When the project was created")]
    pub created_at: String,
    #[schemars(description = "When the project was last updated")]
    pub updated_at: String,
}

impl ProjectSummary {
    fn from_project(project: Project) -> Self {
        Self {
            id: project.id.to_string(),
            name: project.name,
            git_repo_path: project.git_repo_path,
            setup_script: project.setup_script,
            cleanup_script: project.cleanup_script,
            dev_script: project.dev_script,
            created_at: project.created_at.to_rfc3339(),
            updated_at: project.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListProjectsResponse {
    pub projects: Vec<ProjectSummary>,
    pub count: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTasksRequest {
    #[schemars(description = "The ID of the project to list tasks from")]
    pub project_id: Uuid,
    #[schemars(
        description = "Optional status filter: 'todo', 'inprogress', 'inreview', 'done', 'cancelled'"
    )]
    pub status: Option<String>,
    #[schemars(description = "Maximum number of tasks to return (default: 50)")]
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TaskSummary {
    #[schemars(description = "The unique identifier of the task")]
    pub id: String,
    #[schemars(description = "The title of the task")]
    pub title: String,
    #[schemars(description = "Current status of the task")]
    pub status: String,
    #[schemars(description = "When the task was created")]
    pub created_at: String,
    #[schemars(description = "When the task was last updated")]
    pub updated_at: String,
    #[schemars(description = "Whether the task has an in-progress execution attempt")]
    pub has_in_progress_attempt: Option<bool>,
    #[schemars(description = "Whether the task has a merged execution attempt")]
    pub has_merged_attempt: Option<bool>,
    #[schemars(description = "Whether the last execution attempt failed")]
    pub last_attempt_failed: Option<bool>,
}

impl TaskSummary {
    fn from_task_with_status(task: TaskWithAttemptStatus) -> Self {
        Self {
            id: task.id.to_string(),
            title: task.title.to_string(),
            status: task.status.to_string(),
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
            has_in_progress_attempt: Some(task.has_in_progress_attempt),
            has_merged_attempt: Some(task.has_merged_attempt),
            last_attempt_failed: Some(task.last_attempt_failed),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TaskDetails {
    #[schemars(description = "The unique identifier of the task")]
    pub id: String,
    #[schemars(description = "The title of the task")]
    pub title: String,
    #[schemars(description = "Optional description of the task")]
    pub description: Option<String>,
    #[schemars(description = "Current status of the task")]
    pub status: String,
    #[schemars(description = "When the task was created")]
    pub created_at: String,
    #[schemars(description = "When the task was last updated")]
    pub updated_at: String,
    #[schemars(description = "Whether the task has an in-progress execution attempt")]
    pub has_in_progress_attempt: Option<bool>,
    #[schemars(description = "Whether the task has a merged execution attempt")]
    pub has_merged_attempt: Option<bool>,
    #[schemars(description = "Whether the last execution attempt failed")]
    pub last_attempt_failed: Option<bool>,
}

impl TaskDetails {
    fn from_task(task: Task) -> Self {
        Self {
            id: task.id.to_string(),
            title: task.title,
            description: task.description,
            status: task.status.to_string(),
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
            has_in_progress_attempt: None,
            has_merged_attempt: None,
            last_attempt_failed: None,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListTasksResponse {
    pub tasks: Vec<TaskSummary>,
    pub count: usize,
    pub project_id: String,
    pub applied_filters: ListTasksFilters,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListTasksFilters {
    pub status: Option<String>,
    pub limit: i32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateTaskRequest {
    #[schemars(description = "The ID of the task to update")]
    pub task_id: Uuid,
    #[schemars(description = "New title for the task")]
    pub title: Option<String>,
    #[schemars(description = "New description for the task")]
    pub description: Option<String>,
    #[schemars(description = "New status: 'todo', 'inprogress', 'inreview', 'done', 'cancelled'")]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct UpdateTaskResponse {
    pub task: TaskDetails,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteTaskRequest {
    #[schemars(description = "The ID of the task to delete")]
    pub task_id: Uuid,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StartTaskAttemptRequest {
    #[schemars(description = "The ID of the task to start")]
    pub task_id: Uuid,
    #[schemars(
        description = "The coding agent executor to run ('CLAUDE_CODE', 'CODEX', 'GEMINI', 'CURSOR_AGENT', 'OPENCODE')"
    )]
    pub executor: String,
    #[schemars(description = "Optional executor variant, if needed")]
    pub variant: Option<String>,
    #[schemars(description = "The base branch to use for the attempt")]
    pub base_branch: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct StartTaskAttemptResponse {
    pub task_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DeleteTaskResponse {
    pub deleted_task_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTaskRequest {
    #[schemars(description = "The ID of the task to retrieve")]
    pub task_id: Uuid,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GetTaskResponse {
    pub task: TaskDetails,
}

#[derive(Debug, Clone)]
pub struct TaskServer {
    client: reqwest::Client,
    base_url: String,
    tool_router: ToolRouter<TaskServer>,
}

impl TaskServer {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            tool_router: Self::tool_router(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponseEnvelope<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

impl TaskServer {
    fn success<T: Serialize>(data: &T) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(data)
                .unwrap_or_else(|_| "Failed to serialize response".to_string()),
        )]))
    }

    fn err_value(v: serde_json::Value) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::error(vec![Content::text(
            serde_json::to_string_pretty(&v)
                .unwrap_or_else(|_| "Failed to serialize error".to_string()),
        )]))
    }

    fn err<S: Into<String>>(msg: S, details: Option<S>) -> Result<CallToolResult, ErrorData> {
        let mut v = serde_json::json!({"success": false, "error": msg.into()});
        if let Some(d) = details {
            v["details"] = serde_json::json!(d.into());
        };
        Self::err_value(v)
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        rb: reqwest::RequestBuilder,
    ) -> Result<T, CallToolResult> {
        let resp = rb
            .send()
            .await
            .map_err(|e| Self::err("Failed to connect to AF API", Some(&e.to_string())).unwrap())?;

        if !resp.status().is_success() {
            let status = resp.status();
            return Err(
                Self::err(format!("AF API returned error status: {}", status), None).unwrap(),
            );
        }

        let api_response = resp.json::<ApiResponseEnvelope<T>>().await.map_err(|e| {
            Self::err("Failed to parse AF API response", Some(&e.to_string())).unwrap()
        })?;

        if !api_response.success {
            let msg = api_response.message.as_deref().unwrap_or("Unknown error");
            return Err(Self::err("AF API returned error", Some(msg)).unwrap());
        }

        api_response
            .data
            .ok_or_else(|| Self::err("AF API response missing data field", None).unwrap())
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

#[tool_router]
impl TaskServer {
    #[tool(
        description = "Create a new task/ticket in a project. Always pass the `project_id` of the project you want to create the task in - it is required!"
    )]
    async fn create_task(
        &self,
        Parameters(CreateTaskRequest {
            project_id,
            title,
            description,
        }): Parameters<CreateTaskRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/tasks");
        let task: Task = match self
            .send_json(
                self.client
                    .post(&url)
                    .json(&CreateTask::from_title_description(
                        project_id,
                        title,
                        description,
                    )),
            )
            .await
        {
            Ok(t) => t,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&CreateTaskResponse {
            task_id: task.id.to_string(),
        })
    }

    #[tool(description = "List all the available projects")]
    async fn list_projects(&self) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/projects");
        let projects: Vec<Project> = match self.send_json(self.client.get(&url)).await {
            Ok(ps) => ps,
            Err(e) => return Ok(e),
        };

        let project_summaries: Vec<ProjectSummary> = projects
            .into_iter()
            .map(ProjectSummary::from_project)
            .collect();

        let response = ListProjectsResponse {
            count: project_summaries.len(),
            projects: project_summaries,
        };

        TaskServer::success(&response)
    }

    #[tool(
        description = "List all the task/tickets in a project with optional filtering and execution status. `project_id` is required!"
    )]
    async fn list_tasks(
        &self,
        Parameters(ListTasksRequest {
            project_id,
            status,
            limit,
        }): Parameters<ListTasksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let status_filter = if let Some(ref status_str) = status {
            match TaskStatus::from_str(status_str) {
                Ok(s) => Some(s),
                Err(_) => {
                    return Self::err(
                        "Invalid status filter. Valid values: 'todo', 'in-progress', 'in-review', 'done', 'cancelled'".to_string(),
                        Some(status_str.to_string()),
                    );
                }
            }
        } else {
            None
        };

        let url = self.url(&format!("/api/tasks?project_id={}", project_id));
        let all_tasks: Vec<TaskWithAttemptStatus> =
            match self.send_json(self.client.get(&url)).await {
                Ok(t) => t,
                Err(e) => return Ok(e),
            };

        let task_limit = limit.unwrap_or(50).max(0) as usize;
        let filtered = all_tasks.into_iter().filter(|t| {
            if let Some(ref want) = status_filter {
                &t.status == want
            } else {
                true
            }
        });
        let limited: Vec<TaskWithAttemptStatus> = filtered.take(task_limit).collect();

        let task_summaries: Vec<TaskSummary> = limited
            .into_iter()
            .map(TaskSummary::from_task_with_status)
            .collect();

        let response = ListTasksResponse {
            count: task_summaries.len(),
            tasks: task_summaries,
            project_id: project_id.to_string(),
            applied_filters: ListTasksFilters {
                status: status.clone(),
                limit: task_limit as i32,
            },
        };

        TaskServer::success(&response)
    }

    #[tool(description = "Start working on a task by creating and launching a new task attempt.")]
    async fn start_task_attempt(
        &self,
        Parameters(StartTaskAttemptRequest {
            task_id,
            executor,
            variant,
            base_branch,
        }): Parameters<StartTaskAttemptRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let base_branch = base_branch.trim().to_string();
        if base_branch.is_empty() {
            return Self::err("Base branch must not be empty.".to_string(), None::<String>);
        }

        let executor_trimmed = executor.trim();
        if executor_trimmed.is_empty() {
            return Self::err("Executor must not be empty.".to_string(), None::<String>);
        }

        let normalized_executor = executor_trimmed.replace('-', "_").to_ascii_uppercase();
        let base_executor = match BaseCodingAgent::from_str(&normalized_executor) {
            Ok(exec) => exec,
            Err(_) => {
                return Self::err(
                    format!("Unknown executor '{executor_trimmed}'."),
                    None::<String>,
                );
            }
        };

        let variant = variant.and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        let executor_profile_id = ExecutorProfileId {
            executor: base_executor,
            variant,
        };

        let payload = CreateTaskAttemptBody {
            task_id,
            executor_profile_id,
            base_branch,
        };

        let url = self.url("/api/task-attempts");
        let attempt: TaskAttempt = match self.send_json(self.client.post(&url).json(&payload)).await
        {
            Ok(attempt) => attempt,
            Err(e) => return Ok(e),
        };

        let response = StartTaskAttemptResponse {
            task_id: attempt.task_id.to_string(),
            attempt_id: attempt.id.to_string(),
        };

        TaskServer::success(&response)
    }

    #[tool(
        description = "Update an existing task/ticket's title, description, or status. `project_id` and `task_id` are required! `title`, `description`, and `status` are optional."
    )]
    async fn update_task(
        &self,
        Parameters(UpdateTaskRequest {
            task_id,
            title,
            description,
            status,
        }): Parameters<UpdateTaskRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let status = if let Some(ref status_str) = status {
            match TaskStatus::from_str(status_str) {
                Ok(s) => Some(s),
                Err(_) => {
                    return Self::err(
                        "Invalid status filter. Valid values: 'todo', 'in-progress', 'in-review', 'done', 'cancelled'".to_string(),
                        Some(status_str.to_string()),
                    );
                }
            }
        } else {
            None
        };

        let payload = UpdateTask {
            title,
            description,
            status,
            parent_task_attempt: None,
            image_ids: None,
        };
        let url = self.url(&format!("/api/tasks/{}", task_id));
        let updated_task: Task = match self.send_json(self.client.put(&url).json(&payload)).await {
            Ok(t) => t,
            Err(e) => return Ok(e),
        };

        let details = TaskDetails::from_task(updated_task);
        let repsonse = UpdateTaskResponse { task: details };
        TaskServer::success(&repsonse)
    }

    #[tool(
        description = "Delete a task/ticket from a project. `project_id` and `task_id` are required!"
    )]
    async fn delete_task(
        &self,
        Parameters(DeleteTaskRequest { task_id }): Parameters<DeleteTaskRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/tasks/{}", task_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.delete(&url))
            .await
        {
            return Ok(e);
        }

        let repsonse = DeleteTaskResponse {
            deleted_task_id: Some(task_id.to_string()),
        };

        TaskServer::success(&repsonse)
    }

    #[tool(
        description = "Get detailed information (like task description) about a specific task/ticket. You can use `list_tasks` to find the `task_ids` of all tasks in a project. `project_id` and `task_id` are required!"
    )]
    async fn get_task(
        &self,
        Parameters(GetTaskRequest { task_id }): Parameters<GetTaskRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/tasks/{}", task_id));
        let task: Task = match self.send_json(self.client.get(&url)).await {
            Ok(t) => t,
            Err(e) => return Ok(e),
        };

        let details = TaskDetails::from_task(task);
        let response = GetTaskResponse { task: details };

        TaskServer::success(&response)
    }

    // ============================================================================
    // ADVANCED MCP TOOLS - Full Forge Control
    // ============================================================================

    #[tool(description = "Create a new project")]
    async fn create_project(
        &self,
        Parameters(CreateProjectRequest {
            name,
            git_repo_path,
            setup_script,
            cleanup_script,
            dev_script,
        }): Parameters<CreateProjectRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "name": name,
            "git_repo_path": git_repo_path,
            "setup_script": setup_script,
            "cleanup_script": cleanup_script,
            "dev_script": dev_script,
        });

        let url = self.url("/api/projects");
        let project: Project = match self.send_json(self.client.post(&url).json(&payload)).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&serde_json::json!({
            "project_id": project.id.to_string(),
        }))

    }

    #[tool(description = "Get project details by ID")]
    async fn get_project(
        &self,
        Parameters(GetProjectRequest { project_id }): Parameters<GetProjectRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/projects/{}", project_id));
        let project: Project = match self.send_json(self.client.get(&url)).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&ProjectSummary::from_project(project))
    }

    #[tool(description = "Update an existing project")]
    async fn update_project(
        &self,
        Parameters(UpdateProjectRequest {
            project_id,
            name,
            git_repo_path,
            setup_script,
            cleanup_script,
            dev_script,
        }): Parameters<UpdateProjectRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "name": name,
            "git_repo_path": git_repo_path,
            "setup_script": setup_script,
            "cleanup_script": cleanup_script,
            "dev_script": dev_script,
        });

        let url = self.url(&format!("/api/projects/{}", project_id));
        let project: Project = match self.send_json(self.client.put(&url).json(&payload)).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&ProjectSummary::from_project(project))
    }

    #[tool(description = "Delete a project")]
    async fn delete_project(
        &self,
        Parameters(DeleteProjectRequest { project_id }): Parameters<DeleteProjectRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/projects/{}", project_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.delete(&url))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&DeleteProjectResponse {
            deleted_project_id: project_id.to_string(),
        })
    }

    #[tool(description = "Create a task and immediately start execution")]
    async fn create_task_and_start(
        &self,
        Parameters(crate::mcp::advanced_tools::CreateTaskAndStartRequest {
            project_id,
            title,
            description,
            executor,
            variant,
            base_branch,
            parent_task_attempt,
        }): Parameters<crate::mcp::advanced_tools::CreateTaskAndStartRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let base_branch = base_branch.trim().to_string();
        if base_branch.is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Base branch must not be empty.".to_string(),
            )]));
        }

        let executor_trimmed = executor.trim();
        if executor_trimmed.is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Executor must not be empty.".to_string(),
            )]));
        }

        let normalized_executor = executor_trimmed.replace('-', "_").to_ascii_uppercase();
        let base_executor = match BaseCodingAgent::from_str(&normalized_executor) {
            Ok(exec) => exec,
            Err(_) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown executor '{executor_trimmed}'."
                ))]));
            }
        };

        let variant = variant.and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        let executor_profile_id = ExecutorProfileId {
            executor: base_executor,
            variant,
        };

        let payload = serde_json::json!({
            "task": {
                "project_id": project_id,
                "title": title,
                "description": description,
                "parent_task_attempt": parent_task_attempt,
                "image_ids": null
            },
            "executor_profile_id": executor_profile_id,
            "base_branch": base_branch,
        });

        let url = self.url("/api/tasks/create-and-start");
        let result: TaskWithAttemptStatus =
            match self.send_json(self.client.post(&url).json(&payload)).await {
                Ok(r) => r,
                Err(e) => return Ok(e),
            };

        TaskServer::success(&result)
    }

    #[tool(description = "List task attempts, optionally filtered by project_id")]
    async fn list_task_attempts(
        &self,
        Parameters(ListTaskAttemptsRequest { project_id }): Parameters<ListTaskAttemptsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut url = self.url("/api/task-attempts");
        if let Some(pid) = project_id {
            url.push_str(&format!("?project_id={}", pid));
        }

        let attempts: Vec<TaskAttempt> = match self.send_json(self.client.get(&url)).await {
            Ok(attempts) => attempts,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&ListTaskAttemptsResponse {
            count: attempts.len(),
            task_attempts: attempts
                .into_iter()
                .map(TaskAttemptSummary::from_task_attempt)
                .collect(),
            applied_filters: TaskAttemptFilters { project_id },
        })
    }

    #[tool(description = "Get task attempt details by ID")]
    async fn get_task_attempt(
        &self,
        Parameters(GetTaskAttemptRequest { attempt_id }): Parameters<GetTaskAttemptRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}", attempt_id));
        let attempt: TaskAttempt = match self.send_json(self.client.get(&url)).await {
            Ok(a) => a,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&GetTaskAttemptResponse {
            task_attempt: TaskAttemptSummary::from_task_attempt(attempt),
        })
    }

    #[tool(description = "Send a follow-up message to a running task attempt")]
    async fn follow_up(
        &self,
        Parameters(FollowUpRequest {
            attempt_id,
            message,
        }): Parameters<FollowUpRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "prompt": message,
        });

        let url = self.url(&format!("/api/task-attempts/{}/follow-up", attempt_id));
        let _result: serde_json::Value =
            match self.send_json(self.client.post(&url).json(&payload)).await {
                Ok(r) => r,
                Err(e) => return Ok(e),
            };

        TaskServer::success(&serde_json::json!({
            "success": true,
            "attempt_id": attempt_id.to_string(),
        }))
    }

    #[tool(description = "Rebase a task attempt's branch onto its target branch")]
    async fn rebase_task_attempt(
        &self,
        Parameters(RebaseTaskAttemptRequest { attempt_id }): Parameters<RebaseTaskAttemptRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}/rebase", attempt_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url).json(&serde_json::json!({})))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "attempt_id": attempt_id.to_string()
        }))
    }

    #[tool(description = "Merge a task attempt into its target branch")]
    async fn merge_task_attempt(
        &self,
        Parameters(MergeTaskAttemptRequest { attempt_id }): Parameters<MergeTaskAttemptRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}/merge", attempt_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&MergeTaskAttemptResponse { success: true })
    }

    #[tool(description = "Push task attempt branch to remote")]
    async fn push_task_attempt(
        &self,
        Parameters(PushTaskAttemptRequest { attempt_id }): Parameters<PushTaskAttemptRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}/push", attempt_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Task attempt branch pushed successfully"
        }))
    }

    #[tool(description = "Stop a running task attempt execution")]
    async fn stop_task_attempt(
        &self,
        Parameters(stop_req): Parameters<super::advanced_tools::StopTaskAttemptRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}/stop", stop_req.attempt_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&StopTaskAttemptResponse {
            stopped: true,
            attempt_id: stop_req.attempt_id.to_string(),
        })
    }

    #[tool(description = "Create a GitHub pull request for a task attempt")]
    async fn create_pr(
        &self,
        Parameters(CreatePRRequest {
            attempt_id,
            title,
            body,
        }): Parameters<CreatePRRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "title": title.unwrap_or_else(|| "Pull Request".to_string()),
            "body": body,
        });

        let url = self.url(&format!("/api/task-attempts/{}/pr", attempt_id));

        // The endpoint returns ApiResponse<String> where String is the PR URL
        let pr_url: String = match self.send_json(self.client.post(&url).json(&payload)).await {
            Ok(url) => url,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&serde_json::json!({
            "pr_url": pr_url
        }))
    }

    #[tool(description = "Attach an existing PR to a task attempt by PR number")]
    async fn attach_pr(
        &self,
        Parameters(crate::mcp::advanced_tools::AttachPRRequest {
            attempt_id,
            pr_number,
        }): Parameters<crate::mcp::advanced_tools::AttachPRRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "pr_number": pr_number,
        });

        let url = self.url(&format!("/api/task-attempts/{}/pr/attach", attempt_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url).json(&payload))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "attempt_id": attempt_id.to_string(),
            "pr_number": pr_number,
        }))
    }

    #[tool(description = "Get branch status for a task attempt")]
    async fn get_branch_status(
        &self,
        Parameters(GetBranchStatusRequest { attempt_id }): Parameters<GetBranchStatusRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}/branch-status", attempt_id));
        let status: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(s) => s,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&status)
    }

    #[tool(description = "Replace the execution process for a task attempt")]
    async fn replace_process(
        &self,
        Parameters(ReplaceProcessRequest { attempt_id }): Parameters<ReplaceProcessRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!(
            "/api/task-attempts/{}/replace-process",
            attempt_id
        ));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Process replaced successfully"
        }))
    }

    #[tool(description = "Get commit information for a task attempt")]
    async fn get_commit_info(
        &self,
        Parameters(GetCommitInfoRequest { attempt_id }): Parameters<GetCommitInfoRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}/commit-info", attempt_id));
        let commit_info: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(info) => info,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&commit_info)
    }

    #[tool(description = "Compare commits for a task attempt")]
    async fn compare_commit(
        &self,
        Parameters(CompareCommitRequest { attempt_id }): Parameters<CompareCommitRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}/commit-compare", attempt_id));
        let comparison: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(cmp) => cmp,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&comparison)
    }

    #[tool(description = "Start development server for a task attempt")]
    async fn start_dev_server(
        &self,
        Parameters(StartDevServerRequest { attempt_id }): Parameters<StartDevServerRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!(
            "/api/task-attempts/{}/start-dev-server",
            attempt_id
        ));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Development server started"
        }))
    }

    #[tool(description = "Open editor for a task attempt")]
    async fn open_editor(
        &self,
        Parameters(OpenEditorRequest { attempt_id, editor }): Parameters<OpenEditorRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "editor": editor,
        });

        let url = self.url(&format!("/api/task-attempts/{}/open-editor", attempt_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url).json(&payload))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Editor opened"
        }))
    }

    #[tool(description = "Delete a file in a task attempt")]
    async fn delete_file(
        &self,
        Parameters(DeleteFileRequest {
            attempt_id,
            file_path,
        }): Parameters<DeleteFileRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "file_path": file_path,
        });

        let url = self.url(&format!("/api/task-attempts/{}/delete-file", attempt_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url).json(&payload))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "file_path": file_path,
        }))
    }

    #[tool(description = "Get child task attempts for a task attempt")]
    async fn get_children(
        &self,
        Parameters(GetChildrenRequest { attempt_id }): Parameters<GetChildrenRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}/children", attempt_id));
        let children: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(c) => c,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&children)
    }

    #[tool(description = "Abort merge conflicts for a task attempt")]
    async fn abort_conflicts(
        &self,
        Parameters(AbortConflictsRequest { attempt_id }): Parameters<AbortConflictsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!(
            "/api/task-attempts/{}/conflicts/abort",
            attempt_id
        ));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Conflicts aborted"
        }))
    }

    #[tool(description = "Change target branch for a task attempt")]
    async fn change_target_branch(
        &self,
        Parameters(ChangeTargetBranchRequest {
            attempt_id,
            target_branch,
        }): Parameters<ChangeTargetBranchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "target_branch": target_branch,
        });

        let url = self.url(&format!(
            "/api/task-attempts/{}/change-target-branch",
            attempt_id
        ));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url).json(&payload))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "target_branch": target_branch,
        }))
    }

    #[tool(description = "Get draft for a task attempt")]
    async fn get_draft(
        &self,
        Parameters(GetDraftRequest { attempt_id }): Parameters<GetDraftRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}/draft", attempt_id));
        let draft: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(d) => d,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&draft)
    }

    #[tool(description = "Save draft for a task attempt")]
    async fn save_draft(
        &self,
        Parameters(SaveDraftRequest {
            attempt_id,
            content,
        }): Parameters<SaveDraftRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "content": content,
        });

        let url = self.url(&format!("/api/task-attempts/{}/draft", attempt_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.put(&url).json(&payload))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Draft saved successfully"
        }))
    }

    #[tool(description = "Delete draft for a task attempt")]
    async fn delete_draft(
        &self,
        Parameters(DeleteDraftRequest { attempt_id }): Parameters<DeleteDraftRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/task-attempts/{}/draft", attempt_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.delete(&url))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Draft deleted successfully"
        }))
    }

    #[tool(description = "Set draft queue for a task attempt")]
    async fn set_draft_queue(
        &self,
        Parameters(SetDraftQueueRequest { attempt_id, queue }): Parameters<SetDraftQueueRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "queue": queue,
        });

        let url = self.url(&format!("/api/task-attempts/{}/draft/queue", attempt_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url).json(&payload))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Draft queue set successfully"
        }))
    }

    #[tool(description = "List execution processes, optionally filtered by project_id")]
    async fn list_processes(
        &self,
        Parameters(ListProcessesRequest { project_id }): Parameters<ListProcessesRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut url = self.url("/api/execution-processes");
        if let Some(pid) = project_id {
            url.push_str(&format!("?project_id={}", pid));
        }

        let processes: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&processes)
    }

    #[tool(description = "Get execution process details by ID")]
    async fn get_process(
        &self,
        Parameters(GetProcessRequest { process_id }): Parameters<GetProcessRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/execution-processes/{}", process_id));
        let process: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&process)
    }

    #[tool(description = "Stop a running execution process")]
    async fn stop_process(
        &self,
        Parameters(StopProcessRequest { process_id }): Parameters<StopProcessRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/execution-processes/{}/stop", process_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.post(&url))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Process stopped successfully"
        }))
    }

    #[tool(description = "Upload an image and return its ID")]
    async fn upload_image(
        &self,
        Parameters(UploadImageRequest { data, mime_type }): Parameters<UploadImageRequest>,
    ) -> Result<CallToolResult, ErrorData> {
// Decode base64 data
        use base64::Engine;
        let image_bytes = match base64::engine::general_purpose::STANDARD.decode(&data) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to decode base64 data: {}",
                    e
                ))]));
            }
        };

        // Create multipart form with the image
        let part = match reqwest::multipart::Part::bytes(image_bytes)
            .file_name("image")
            .mime_str(&mime_type)
        {
            Ok(p) => p,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Invalid MIME type '{}': {}",
                    mime_type, e
                ))]));
            }
        };

        let form = reqwest::multipart::Form::new().part("image", part);

        let url = self.url("/api/images/upload");
        let response = match self.client.post(&url).multipart(form).send().await {
            Ok(r) => r,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to upload image: {}",
                    e
                ))]));
            }
        };

        let status = response.status();
        let body = match response.text().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to read response: {}",
                    e
                ))]));
            }
        };

        if !status.is_success() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Upload failed with status {}: {}",
                status, body
            ))]));
        }

        #[derive(serde::Deserialize)]
        struct ApiResponse {
            data: ImageData,
        }

        #[derive(serde::Deserialize)]
        struct ImageData {
            id: Uuid,
        }

        let api_response: ApiResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to parse response: {}",
                    e
                ))]));
            }
        };

        TaskServer::success(&UploadImageResponse {
            image_id: api_response.data.id,
        })
    }

    #[tool(description = "Get image data by ID")]
    async fn get_image(
        &self,
        Parameters(GetImageRequest { image_id }): Parameters<GetImageRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/images/{}/file", image_id));
        let image_data: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(data) => data,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&image_data)
    }

    #[tool(description = "Get application configuration")]
    async fn get_config(&self) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/config");
        let config: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(c) => c,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&config)
    }

    #[tool(description = "Update application configuration")]
    async fn update_config(
        &self,
        Parameters(UpdateConfigRequest { config }): Parameters<UpdateConfigRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "config": config,
        });

        let url = self.url("/api/config");
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.put(&url).json(&payload))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Configuration updated successfully"
        }))
    }

    #[tool(description = "List all drafts for a project")]
    async fn list_drafts(
        &self,
        Parameters(ListDraftsRequest { project_id }): Parameters<ListDraftsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/drafts?project_id={}", project_id));
        let drafts: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(d) => d,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&drafts)
    }

    #[tool(description = "Create a new draft")]
    async fn create_draft(
        &self,
        Parameters(crate::mcp::advanced_tools::CreateDraftRequest {
            project_id,
            title,
            content,
        }): Parameters<crate::mcp::advanced_tools::CreateDraftRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "project_id": project_id,
            "title": title,
            "content": content,
        });

        let url = self.url("/api/drafts");
        let result: serde_json::Value =
            match self.send_json(self.client.post(&url).json(&payload)).await {
                Ok(r) => r,
                Err(e) => return Ok(e),
            };

        TaskServer::success(&result)
    }

    #[tool(description = "Get draft by ID")]
    async fn get_draft_by_id(
        &self,
        Parameters(crate::mcp::advanced_tools::GetDraftByIdRequest { draft_id }): Parameters<
            crate::mcp::advanced_tools::GetDraftByIdRequest,
        >,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/drafts/{}", draft_id));
        let result: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(r) => r,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&result)
    }

    #[tool(description = "Update draft by ID")]
    async fn update_draft(
        &self,
        Parameters(UpdateDraftRequest {
            draft_id,
            title,
            content,
        }): Parameters<UpdateDraftRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let payload = serde_json::json!({
            "title": title,
            "content": content,
        });

        let url = self.url(&format!("/api/drafts/{}", draft_id));
        let result: serde_json::Value =
            match self.send_json(self.client.put(&url).json(&payload)).await {
                Ok(r) => r,
                Err(e) => return Ok(e),
            };

        TaskServer::success(&result)
    }

    #[tool(description = "List all containers")]
    async fn list_containers(&self) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/containers");
        let containers: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(c) => c,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&containers)
    }

    #[tool(description = "Get container details")]
    async fn get_container(
        &self,
        Parameters(request): Parameters<GetContainerRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/containers/{}", request.container_id));
        let result: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(r) => r,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&result)
    }

    #[tool(description = "Get filesystem tree for a project")]
    async fn get_filesystem_tree(
        &self,
        Parameters(crate::mcp::advanced_tools::GetFilesystemTreeRequest {
            project_id,
            path,
        }): Parameters<crate::mcp::advanced_tools::GetFilesystemTreeRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut url = self.url(&format!("/api/filesystem/tree?project_id={}", project_id));
        if let Some(p) = path {
            url.push_str(&format!("&path={}", urlencoding::encode(&p)));
        }

        let tree: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(t) => t,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&tree)
    }

    #[tool(description = "Get file contents from a project")]
    async fn get_file(
        &self,
        Parameters(GetFileRequest {
            project_id,
            file_path,
        }): Parameters<GetFileRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut url = self.url("/api/filesystem/file");
        url.push_str(&format!("?project_id={}", project_id));
        url.push_str(&format!("&file_path={}", urlencoding::encode(&file_path)));

        let file_content: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(content) => content,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&file_content)
    }

    #[tool(description = "Get Forge configuration")]
    async fn get_forge_config(&self) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/forge/config");
        let config: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(c) => c,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&config)
    }

    #[tool(description = "Update Forge configuration")]
    async fn update_forge_config(
        &self,
        Parameters(crate::mcp::advanced_tools::UpdateForgeConfigRequest { config }): Parameters<
            crate::mcp::advanced_tools::UpdateForgeConfigRequest,
        >,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/forge/config");
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.put(&url).json(&config))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Forge configuration updated successfully"
        }))
    }

    #[tool(description = "Get project settings by ID")]
    async fn get_project_settings(
        &self,
        Parameters(GetProjectSettingsRequest { project_id }): Parameters<GetProjectSettingsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/forge/projects/{}/settings", project_id));
        let settings: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(s) => s,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&settings)
    }

    #[tool(description = "Update project settings")]
    async fn update_project_settings(
        &self,
        Parameters(UpdateProjectSettingsRequest {
            project_id,
            settings,
        }): Parameters<UpdateProjectSettingsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url(&format!("/api/forge/projects/{}/settings", project_id));
        if let Err(e) = self
            .send_json::<serde_json::Value>(self.client.put(&url).json(&settings))
            .await
        {
            return Ok(e);
        }

        TaskServer::success(&serde_json::json!({
            "success": true,
            "message": "Project settings updated successfully"
        }))
    }

    #[tool(description = "Get Omni status")]
    async fn get_omni_status(&self) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/forge/omni/status");
        let result: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(r) => r,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&result)
    }

    #[tool(description = "List all Omni instances")]
    async fn list_omni_instances(&self) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/forge/omni/instances");
        let instances: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(i) => i,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&instances)
    }

    #[tool(description = "Validate Omni configuration")]
    async fn validate_omni_config(
        &self,
        Parameters(request): Parameters<ValidateOmniConfigRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/forge/omni/validate");
        let result: serde_json::Value = match self
            .send_json(self.client.post(&url).json(&request))
            .await
        {
            Ok(r) => r,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&result)
    }

    #[tool(description = "List all Omni notifications")]
    async fn list_omni_notifications(&self) -> Result<CallToolResult, ErrorData> {
        let url = self.url("/api/forge/omni/notifications");
        let notifications: serde_json::Value = match self.send_json(self.client.get(&url)).await {
            Ok(n) => n,
            Err(e) => return Ok(e),
        };

        TaskServer::success(&notifications)
    }
}

#[tool_handler]
impl ServerHandler for TaskServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "automagik-forge".to_string(),
                version: "1.0.0".to_string(),
            },
            instructions: Some("A task and project management server. If you need to create or update tickets or tasks then use these tools. Most of them absolutely require that you pass the `project_id` of the project that you are currently working on. This should be provided to you. Call `list_tasks` to fetch the `task_ids` of all the tasks in a project`. TOOLS: 'list_projects', 'list_tasks', 'create_task', 'start_task_attempt', 'get_task', 'update_task', 'delete_task'. Make sure to pass `project_id` or `task_id` where required. You can use list tools to get the available ids.".to_string()),
        }
    }
}
