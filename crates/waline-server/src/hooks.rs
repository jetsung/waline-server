use waline_common::models::Comment;
use std::future::Future;
use std::pin::Pin;

/// Hook type alias for async functions
type HookFn = Box<dyn Fn(Comment) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>;

/// Plugin hook system for lifecycle events
pub struct HookSystem {
    pre_save_hooks: Vec<HookFn>,
    post_save_hooks: Vec<HookFn>,
    pre_update_hooks: Vec<HookFn>,
    post_update_hooks: Vec<HookFn>,
    pre_delete_hooks: Vec<HookFn>,
    post_delete_hooks: Vec<HookFn>,
}

impl HookSystem {
    pub fn new() -> Self {
        Self {
            pre_save_hooks: Vec::new(),
            post_save_hooks: Vec::new(),
            pre_update_hooks: Vec::new(),
            post_update_hooks: Vec::new(),
            pre_delete_hooks: Vec::new(),
            post_delete_hooks: Vec::new(),
        }
    }

    /// Register a preSave hook
    pub fn on_pre_save(&mut self, hook: HookFn) {
        self.pre_save_hooks.push(hook);
    }

    /// Register a postSave hook
    pub fn on_post_save(&mut self, hook: HookFn) {
        self.post_save_hooks.push(hook);
    }

    /// Execute preSave hooks
    pub async fn run_pre_save(&self, comment: &Comment) -> Result<(), String> {
        for hook in &self.pre_save_hooks {
            hook(comment.clone()).await?;
        }
        Ok(())
    }

    /// Execute postSave hooks
    pub async fn run_post_save(&self, comment: &Comment) -> Result<(), String> {
        for hook in &self.post_save_hooks {
            let _ = hook(comment.clone()).await; // Best-effort for post hooks
        }
        Ok(())
    }

    /// Execute preDelete hooks
    pub async fn run_pre_delete(&self, comment: &Comment) -> Result<(), String> {
        for hook in &self.pre_delete_hooks {
            hook(comment.clone()).await?;
        }
        Ok(())
    }

    /// Execute postDelete hooks
    pub async fn run_post_delete(&self, comment: &Comment) -> Result<(), String> {
        for hook in &self.post_delete_hooks {
            let _ = hook(comment.clone()).await;
        }
        Ok(())
    }
}
