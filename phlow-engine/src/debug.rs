use once_cell::sync::OnceCell;
use phlow_sdk::prelude::Value;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

static DEBUG_CONTROLLER: OnceCell<Arc<DebugController>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct DebugContext {
    pub payload: Option<Value>,
    pub main: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct DebugSnapshot {
    pub context: DebugContext,
    pub step: Value,
    pub pipeline: usize,
    pub compiled: Value,
}

#[derive(Debug)]
struct DebugState {
    current: Option<DebugSnapshot>,
    history: Vec<DebugSnapshot>,
    executing: bool,
    script: Option<Value>,
    release_current: bool,
    release_pipeline: Option<usize>,
}

impl DebugState {
    fn new() -> Self {
        Self {
            current: None,
            history: Vec::new(),
            executing: false,
            script: None,
            release_current: false,
            release_pipeline: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugReleaseResult {
    Released,
    Awaiting,
    NoStep,
}

#[derive(Debug)]
pub struct DebugController {
    state: Mutex<DebugState>,
    notify: Notify,
}

impl DebugController {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(DebugState::new()),
            notify: Notify::new(),
        }
    }

    pub async fn before_step(&self, snapshot: DebugSnapshot) {
        loop {
            let mut state = self.state.lock().await;
            if let Some(release_pipeline) = state.release_pipeline {
                if release_pipeline == snapshot.pipeline {
                    state.executing = true;
                    state.history.push(snapshot);
                    return;
                }
                state.release_pipeline = None;
            }

            if state.current.is_none() {
                state.current = Some(snapshot);
                state.executing = false;
                break;
            }

            drop(state);
            self.notify.notified().await;
        }

        loop {
            let mut state = self.state.lock().await;
            let current_pipeline = state.current.as_ref().map(|current| current.pipeline);
            let should_release = state.release_current
                || state.release_pipeline.is_some_and(|pipe| Some(pipe) == current_pipeline);

            if should_release {
                state.release_current = false;
                if let Some(current) = state.current.take() {
                    state.history.push(current);
                }
                state.executing = true;
                self.notify.notify_waiters();
                return;
            }

            drop(state);
            self.notify.notified().await;
        }
    }

    pub async fn current_snapshot(&self) -> Option<DebugSnapshot> {
        let state = self.state.lock().await;
        state.current.clone()
    }

    pub async fn show_snapshot(&self) -> Option<DebugSnapshot> {
        let state = self.state.lock().await;
        if let Some(current) = &state.current {
            return Some(current.clone());
        }
        if state.executing {
            return state.history.last().cloned();
        }
        None
    }

    pub async fn set_script(&self, script: Value) {
        let mut state = self.state.lock().await;
        state.script = Some(script);
    }

    pub async fn show_script(&self) -> Option<Value> {
        let state = self.state.lock().await;
        state.script.clone()
    }

    pub async fn history(&self) -> Vec<DebugSnapshot> {
        let state = self.state.lock().await;
        state.history.clone()
    }

    pub async fn release_next(&self) -> DebugReleaseResult {
        let mut state = self.state.lock().await;
        if state.current.is_none() {
            return if state.executing {
                DebugReleaseResult::Awaiting
            } else {
                DebugReleaseResult::NoStep
            };
        }
        state.release_current = true;
        state.executing = true;
        self.notify.notify_waiters();
        DebugReleaseResult::Released
    }

    pub async fn release_pipeline(&self) -> DebugReleaseResult {
        let mut state = self.state.lock().await;
        let Some(current) = state.current.as_ref() else {
            return if state.executing {
                DebugReleaseResult::Awaiting
            } else {
                DebugReleaseResult::NoStep
            };
        };
        state.release_pipeline = Some(current.pipeline);
        state.release_current = true;
        state.executing = true;
        self.notify.notify_waiters();
        DebugReleaseResult::Released
    }

    pub async fn pause_release(&self) -> bool {
        let mut state = self.state.lock().await;
        let was_active = state.release_pipeline.is_some();
        state.release_pipeline = None;
        state.release_current = false;
        was_active
    }

    pub async fn finish_step(&self) {
        let mut state = self.state.lock().await;
        state.executing = false;
    }
}

pub fn set_debug_controller(
    controller: Arc<DebugController>,
) -> Result<(), Arc<DebugController>> {
    DEBUG_CONTROLLER.set(controller)
}

pub fn debug_controller() -> Option<&'static Arc<DebugController>> {
    DEBUG_CONTROLLER.get()
}
