//! # M29: IPC Bus State
//!
//! Bus state management for the IPC coordination layer. Manages tasks, events,
//! subscriptions, and cascade tracking. This module is the state backend; the actual
//! Unix domain socket server lives in the binary.
//!
//! ## Layer: L7 | Module: M29 | Dependencies: L1, L7 (M30 bus types)
//! ## Wire Protocol: Handshake -> Welcome -> Subscribe/Submit/Event frames
//! ## Design Constraints: C5 (lock ordering), C12 (bounded channel 256)

use std::collections::{HashMap, HashSet, VecDeque};

use crate::m1_foundation::{
    m01_core_types::{now_secs, PaneId, TaskId},
    m02_error_handling::{PvError, PvResult},
};
use super::m30_bus_types::{BusEvent, BusTask, TaskStatus, TaskTarget};

// ──────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────

/// Maximum number of pending tasks in the bus.
const MAX_PENDING_TASKS: usize = 256;

/// Maximum events retained in the event ring buffer.
const MAX_EVENTS: usize = 1000;

/// Maximum cascade events per rate-limit window.
const CASCADE_RATE_LIMIT: u32 = 10;

/// Rate-limit window duration in seconds.
const CASCADE_WINDOW_SECS: f64 = 60.0;

/// Maximum number of subscribers.
const MAX_SUBSCRIBERS: usize = 50;

// ──────────────────────────────────────────────────────────────
// Subscriber
// ──────────────────────────────────────────────────────────────

/// A connected bus subscriber with glob-pattern event filtering.
#[derive(Debug, Clone)]
pub struct BusSubscriber {
    /// The subscriber's sphere ID.
    pub pane_id: PaneId,
    /// Glob patterns this subscriber is interested in.
    pub patterns: Vec<String>,
    /// Session ID assigned at handshake.
    pub session_id: String,
    /// Unix timestamp of last activity.
    pub last_active: f64,
}

impl BusSubscriber {
    /// Create a new subscriber.
    #[must_use]
    pub fn new(pane_id: PaneId, session_id: String) -> Self {
        Self {
            pane_id,
            patterns: Vec::new(),
            session_id,
            last_active: now_secs(),
        }
    }

    /// Whether this subscriber matches an event type.
    #[must_use]
    pub fn matches_event(&self, event_type: &str) -> bool {
        self.patterns.iter().any(|pattern| {
            if pattern == "*" {
                return true;
            }
            if let Some(prefix) = pattern.strip_suffix('*') {
                return event_type.starts_with(prefix);
            }
            pattern == event_type
        })
    }

    /// Touch the last-active timestamp.
    pub fn touch(&mut self) {
        self.last_active = now_secs();
    }
}

// ──────────────────────────────────────────────────────────────
// Cascade rate limiter
// ──────────────────────────────────────────────────────────────

/// Rate limiter for cascade handoff events.
#[derive(Debug, Clone)]
pub struct CascadeRateLimiter {
    /// Count of cascades in the current window.
    count: u32,
    /// Start of the current window (Unix timestamp).
    window_start: f64,
}

impl CascadeRateLimiter {
    /// Create a new rate limiter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            count: 0,
            window_start: now_secs(),
        }
    }

    /// Check whether a new cascade is allowed, and if so, increment the counter.
    ///
    /// # Errors
    /// Returns `PvError::CascadeRateLimit` if the rate limit is exceeded.
    pub fn check_and_increment(&mut self) -> PvResult<()> {
        let now = now_secs();
        if now - self.window_start > CASCADE_WINDOW_SECS {
            self.count = 0;
            self.window_start = now;
        }
        if self.count >= CASCADE_RATE_LIMIT {
            return Err(PvError::CascadeRateLimit {
                per_minute: CASCADE_RATE_LIMIT,
            });
        }
        self.count += 1;
        Ok(())
    }

    /// Current count in this window.
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.count
    }

    /// Reset the rate limiter.
    pub fn reset(&mut self) {
        self.count = 0;
        self.window_start = now_secs();
    }
}

impl Default for CascadeRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────────────────────
// BusState
// ──────────────────────────────────────────────────────────────

/// IPC bus state: tasks, events, subscribers, and cascade tracking.
///
/// Lock ordering: always acquire `AppState` BEFORE `BusState`.
#[derive(Debug)]
pub struct BusState {
    /// Active tasks indexed by ID.
    tasks: HashMap<TaskId, BusTask>,
    /// Event ring buffer (most recent at back).
    events: VecDeque<BusEvent>,
    /// Connected subscribers indexed by session ID.
    subscribers: HashMap<String, BusSubscriber>,
    /// Cascade rate limiter.
    cascade_limiter: CascadeRateLimiter,
    /// Set of completed/failed task IDs for deduplication.
    completed_task_ids: HashSet<String>,
}

impl BusState {
    /// Create a new empty bus state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            events: VecDeque::with_capacity(MAX_EVENTS),
            subscribers: HashMap::new(),
            cascade_limiter: CascadeRateLimiter::new(),
            completed_task_ids: HashSet::new(),
        }
    }

    // ── Task management ──

    /// Submit a new task to the bus.
    ///
    /// # Errors
    /// Returns `PvError::BusProtocol` if too many pending tasks.
    pub fn submit_task(&mut self, task: BusTask) -> PvResult<TaskId> {
        let pending = self.tasks.values().filter(|t| t.is_pending()).count();
        if pending >= MAX_PENDING_TASKS {
            return Err(PvError::BusProtocol(format!(
                "too many pending tasks ({pending}/{MAX_PENDING_TASKS})"
            )));
        }
        let id = task.id.clone();
        self.tasks.insert(id.clone(), task);
        Ok(id)
    }

    /// Claim a task by ID for a specific sphere.
    ///
    /// # Errors
    /// Returns `PvError::BusTaskNotFound` if the task does not exist.
    /// Returns `PvError::BusProtocol` if the task is not pending.
    pub fn claim_task(&mut self, task_id: &TaskId, claimer: PaneId) -> PvResult<()> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| PvError::BusTaskNotFound(task_id.as_str().to_owned()))?;
        if !task.claim(claimer) {
            return Err(PvError::BusProtocol(format!(
                "task {} is not pending (status: {})",
                task_id, task.status
            )));
        }
        Ok(())
    }

    /// Mark a task as completed.
    ///
    /// # Errors
    /// Returns `PvError::BusTaskNotFound` if the task does not exist.
    /// Returns `PvError::BusProtocol` if the task is not claimed.
    pub fn complete_task(&mut self, task_id: &TaskId) -> PvResult<()> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| PvError::BusTaskNotFound(task_id.as_str().to_owned()))?;
        if !task.complete() {
            return Err(PvError::BusProtocol(format!(
                "task {} is not claimed (status: {})",
                task_id, task.status
            )));
        }
        self.completed_task_ids.insert(task_id.as_str().to_owned());
        Ok(())
    }

    /// Mark a task as failed.
    ///
    /// # Errors
    /// Returns `PvError::BusTaskNotFound` if the task does not exist.
    /// Returns `PvError::BusProtocol` if the task is not claimed.
    pub fn fail_task(&mut self, task_id: &TaskId) -> PvResult<()> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| PvError::BusTaskNotFound(task_id.as_str().to_owned()))?;
        if !task.fail() {
            return Err(PvError::BusProtocol(format!(
                "task {} is not claimed (status: {})",
                task_id, task.status
            )));
        }
        self.completed_task_ids.insert(task_id.as_str().to_owned());
        Ok(())
    }

    /// Get all pending tasks.
    #[must_use]
    pub fn pending_tasks(&self) -> Vec<&BusTask> {
        self.tasks.values().filter(|t| t.is_pending()).collect()
    }

    /// Get all tasks matching a specific status.
    #[must_use]
    pub fn tasks_by_status(&self, status: TaskStatus) -> Vec<&BusTask> {
        self.tasks
            .values()
            .filter(|t| t.status == status)
            .collect()
    }

    /// Get a task by ID.
    #[must_use]
    pub fn get_task(&self, task_id: &TaskId) -> Option<&BusTask> {
        self.tasks.get(task_id)
    }

    /// Get pending tasks targeted at a specific sphere.
    #[must_use]
    pub fn tasks_for_sphere(&self, pane_id: &PaneId) -> Vec<&BusTask> {
        self.tasks
            .values()
            .filter(|t| {
                t.is_pending()
                    && match &t.target {
                        TaskTarget::Specific { pane_id: tid } => tid == pane_id,
                        TaskTarget::AnyIdle | TaskTarget::FieldDriven | TaskTarget::Willing => true,
                    }
            })
            .collect()
    }

    /// Total task count (all statuses).
    #[must_use]
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Prune completed/failed tasks older than `max_age_secs`.
    pub fn prune_completed_tasks(&mut self, max_age_secs: f64) {
        let now = now_secs();
        self.tasks.retain(|_, task| {
            if task.is_terminal() {
                if let Some(completed_at) = task.completed_at {
                    return now - completed_at < max_age_secs;
                }
                return false;
            }
            true
        });
    }

    // ── Event management ──

    /// Publish an event to the bus.
    pub fn publish_event(&mut self, event: BusEvent) {
        self.events.push_back(event);
        while self.events.len() > MAX_EVENTS {
            self.events.pop_front();
        }
    }

    /// Get recent events (most recent `n`).
    #[must_use]
    pub fn recent_events(&self, n: usize) -> Vec<&BusEvent> {
        self.events.iter().rev().take(n).collect()
    }

    /// Get events matching a pattern.
    #[must_use]
    pub fn events_matching(&self, pattern: &str) -> Vec<&BusEvent> {
        self.events
            .iter()
            .filter(|e| e.matches_pattern(pattern))
            .collect()
    }

    /// Total event count in the buffer.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    // ── Subscriber management ──

    /// Register a new subscriber.
    ///
    /// # Errors
    /// Returns `PvError::BusProtocol` if the maximum number of subscribers is reached.
    pub fn add_subscriber(&mut self, subscriber: BusSubscriber) -> PvResult<()> {
        if self.subscribers.len() >= MAX_SUBSCRIBERS {
            return Err(PvError::BusProtocol(format!(
                "max subscribers reached ({MAX_SUBSCRIBERS})"
            )));
        }
        self.subscribers
            .insert(subscriber.session_id.clone(), subscriber);
        Ok(())
    }

    /// Remove a subscriber by session ID.
    pub fn remove_subscriber(&mut self, session_id: &str) -> Option<BusSubscriber> {
        self.subscribers.remove(session_id)
    }

    /// Get subscriber IDs that match an event type.
    #[must_use]
    pub fn matching_subscribers(&self, event_type: &str) -> Vec<String> {
        self.subscribers
            .values()
            .filter(|s| s.matches_event(event_type))
            .map(|s| s.session_id.clone())
            .collect()
    }

    /// Total subscriber count.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    /// Update subscription patterns for a subscriber.
    ///
    /// # Errors
    /// Returns `PvError::BusProtocol` if the subscriber is not found.
    pub fn update_subscriptions(
        &mut self,
        session_id: &str,
        patterns: Vec<String>,
    ) -> PvResult<usize> {
        let subscriber = self.subscribers.get_mut(session_id).ok_or_else(|| {
            PvError::BusProtocol(format!("subscriber not found: {session_id}"))
        })?;
        subscriber.patterns = patterns;
        subscriber.touch();
        Ok(subscriber.patterns.len())
    }

    // ── Cascade rate limiting ──

    /// Check cascade rate limit.
    ///
    /// # Errors
    /// Returns `PvError::CascadeRateLimit` if the limit is exceeded.
    pub fn check_cascade_rate(&mut self) -> PvResult<()> {
        self.cascade_limiter.check_and_increment()
    }

    /// Current cascade count in the window.
    #[must_use]
    pub const fn cascade_count(&self) -> u32 {
        self.cascade_limiter.count()
    }

    /// Reset cascade rate limiter.
    pub fn reset_cascade_rate(&mut self) {
        self.cascade_limiter.reset();
    }
}

impl Default for BusState {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn pid(s: &str) -> PaneId {
        PaneId::new(s)
    }

    fn make_task(desc: &str) -> BusTask {
        BusTask::new(desc.into(), TaskTarget::AnyIdle, pid("submitter"))
    }

    fn make_subscriber(id: &str) -> BusSubscriber {
        BusSubscriber::new(pid(id), format!("session-{id}"))
    }

    // ── BusSubscriber ──

    #[test]
    fn subscriber_creation() {
        let sub = make_subscriber("alpha");
        assert_eq!(sub.pane_id.as_str(), "alpha");
        assert!(sub.patterns.is_empty());
        assert!(sub.last_active > 0.0);
    }

    #[test]
    fn subscriber_matches_wildcard() {
        let mut sub = make_subscriber("a");
        sub.patterns = vec!["*".into()];
        assert!(sub.matches_event("anything"));
    }

    #[test]
    fn subscriber_matches_prefix() {
        let mut sub = make_subscriber("a");
        sub.patterns = vec!["field.*".into()];
        assert!(sub.matches_event("field.tick"));
        assert!(!sub.matches_event("sphere.registered"));
    }

    #[test]
    fn subscriber_matches_exact() {
        let mut sub = make_subscriber("a");
        sub.patterns = vec!["field.tick".into()];
        assert!(sub.matches_event("field.tick"));
        assert!(!sub.matches_event("field.decision"));
    }

    #[test]
    fn subscriber_no_patterns_no_match() {
        let sub = make_subscriber("a");
        assert!(!sub.matches_event("anything"));
    }

    #[test]
    fn subscriber_multiple_patterns() {
        let mut sub = make_subscriber("a");
        sub.patterns = vec!["field.*".into(), "sphere.*".into()];
        assert!(sub.matches_event("field.tick"));
        assert!(sub.matches_event("sphere.registered"));
        assert!(!sub.matches_event("bus.task"));
    }

    #[test]
    fn subscriber_touch_updates_timestamp() {
        let mut sub = make_subscriber("a");
        let before = sub.last_active;
        std::thread::sleep(std::time::Duration::from_millis(5));
        sub.touch();
        assert!(sub.last_active >= before);
    }

    // ── CascadeRateLimiter ──

    #[test]
    fn rate_limiter_default() {
        let limiter = CascadeRateLimiter::default();
        assert_eq!(limiter.count(), 0);
    }

    #[test]
    fn rate_limiter_allows_within_limit() {
        let mut limiter = CascadeRateLimiter::new();
        for _ in 0..CASCADE_RATE_LIMIT {
            assert!(limiter.check_and_increment().is_ok());
        }
    }

    #[test]
    fn rate_limiter_blocks_at_limit() {
        let mut limiter = CascadeRateLimiter::new();
        for _ in 0..CASCADE_RATE_LIMIT {
            limiter.check_and_increment().ok();
        }
        assert!(limiter.check_and_increment().is_err());
    }

    #[test]
    fn rate_limiter_count_increments() {
        let mut limiter = CascadeRateLimiter::new();
        assert_eq!(limiter.count(), 0);
        limiter.check_and_increment().ok();
        assert_eq!(limiter.count(), 1);
    }

    #[test]
    fn rate_limiter_reset() {
        let mut limiter = CascadeRateLimiter::new();
        limiter.check_and_increment().ok();
        limiter.reset();
        assert_eq!(limiter.count(), 0);
    }

    // ── BusState: construction ──

    #[test]
    fn bus_state_new_is_empty() {
        let state = BusState::new();
        assert_eq!(state.task_count(), 0);
        assert_eq!(state.event_count(), 0);
        assert_eq!(state.subscriber_count(), 0);
    }

    #[test]
    fn bus_state_default_is_empty() {
        let state = BusState::default();
        assert_eq!(state.task_count(), 0);
    }

    // ── BusState: tasks ──

    #[test]
    fn submit_task_success() {
        let mut state = BusState::new();
        let task = make_task("test task");
        let id = state.submit_task(task).unwrap();
        assert_eq!(state.task_count(), 1);
        assert!(state.get_task(&id).is_some());
    }

    #[test]
    fn submit_task_returns_id() {
        let mut state = BusState::new();
        let task = make_task("work");
        let id = state.submit_task(task).unwrap();
        let fetched = state.get_task(&id).unwrap();
        assert_eq!(fetched.description, "work");
    }

    #[test]
    fn pending_tasks_returns_only_pending() {
        let mut state = BusState::new();
        let id1 = state.submit_task(make_task("pending")).unwrap();
        let id2 = state.submit_task(make_task("claimed")).unwrap();
        state.claim_task(&id2, pid("claimer")).unwrap();
        let pending = state.pending_tasks();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id.as_str(), id1.as_str());
    }

    #[test]
    fn claim_task_success() {
        let mut state = BusState::new();
        let id = state.submit_task(make_task("work")).unwrap();
        assert!(state.claim_task(&id, pid("claimer")).is_ok());
        let task = state.get_task(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Claimed);
    }

    #[test]
    fn claim_task_not_found() {
        let mut state = BusState::new();
        let fake_id = TaskId::new();
        assert!(state.claim_task(&fake_id, pid("x")).is_err());
    }

    #[test]
    fn claim_task_already_claimed() {
        let mut state = BusState::new();
        let id = state.submit_task(make_task("work")).unwrap();
        state.claim_task(&id, pid("a")).unwrap();
        assert!(state.claim_task(&id, pid("b")).is_err());
    }

    #[test]
    fn complete_task_success() {
        let mut state = BusState::new();
        let id = state.submit_task(make_task("work")).unwrap();
        state.claim_task(&id, pid("claimer")).unwrap();
        assert!(state.complete_task(&id).is_ok());
        assert_eq!(state.get_task(&id).unwrap().status, TaskStatus::Completed);
    }

    #[test]
    fn complete_task_not_claimed() {
        let mut state = BusState::new();
        let id = state.submit_task(make_task("work")).unwrap();
        assert!(state.complete_task(&id).is_err());
    }

    #[test]
    fn fail_task_success() {
        let mut state = BusState::new();
        let id = state.submit_task(make_task("work")).unwrap();
        state.claim_task(&id, pid("claimer")).unwrap();
        assert!(state.fail_task(&id).is_ok());
        assert_eq!(state.get_task(&id).unwrap().status, TaskStatus::Failed);
    }

    #[test]
    fn fail_task_not_claimed() {
        let mut state = BusState::new();
        let id = state.submit_task(make_task("work")).unwrap();
        assert!(state.fail_task(&id).is_err());
    }

    #[test]
    fn tasks_by_status() {
        let mut state = BusState::new();
        state.submit_task(make_task("a")).unwrap();
        let id = state.submit_task(make_task("b")).unwrap();
        state.claim_task(&id, pid("x")).unwrap();
        assert_eq!(state.tasks_by_status(TaskStatus::Pending).len(), 1);
        assert_eq!(state.tasks_by_status(TaskStatus::Claimed).len(), 1);
        assert_eq!(state.tasks_by_status(TaskStatus::Completed).len(), 0);
    }

    #[test]
    fn tasks_for_sphere_any_idle() {
        let mut state = BusState::new();
        state.submit_task(make_task("work")).unwrap();
        let tasks = state.tasks_for_sphere(&pid("any-sphere"));
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn tasks_for_sphere_specific() {
        let mut state = BusState::new();
        let task = BusTask::new(
            "targeted".into(),
            TaskTarget::Specific { pane_id: pid("target") },
            pid("sub"),
        );
        state.submit_task(task).unwrap();
        assert_eq!(state.tasks_for_sphere(&pid("target")).len(), 1);
        assert_eq!(state.tasks_for_sphere(&pid("other")).len(), 0);
    }

    #[test]
    fn prune_completed_tasks() {
        let mut state = BusState::new();
        let id = state.submit_task(make_task("work")).unwrap();
        state.claim_task(&id, pid("x")).unwrap();
        state.complete_task(&id).unwrap();
        // Prune with 0 max age should remove it
        state.prune_completed_tasks(0.0);
        assert_eq!(state.task_count(), 0);
    }

    #[test]
    fn prune_keeps_recent_completed() {
        let mut state = BusState::new();
        let id = state.submit_task(make_task("work")).unwrap();
        state.claim_task(&id, pid("x")).unwrap();
        state.complete_task(&id).unwrap();
        // Prune with large max age should keep it
        state.prune_completed_tasks(3600.0);
        assert_eq!(state.task_count(), 1);
    }

    // ── BusState: events ──

    #[test]
    fn publish_event() {
        let mut state = BusState::new();
        state.publish_event(BusEvent::text("field.tick", "data", 1));
        assert_eq!(state.event_count(), 1);
    }

    #[test]
    fn recent_events_returns_latest() {
        let mut state = BusState::new();
        for i in 0..5 {
            state.publish_event(BusEvent::text("test", &format!("ev{i}"), i));
        }
        let recent = state.recent_events(3);
        assert_eq!(recent.len(), 3);
        // Most recent first
        assert_eq!(recent[0].tick, 4);
    }

    #[test]
    fn recent_events_caps_at_available() {
        let mut state = BusState::new();
        state.publish_event(BusEvent::text("test", "data", 0));
        let recent = state.recent_events(100);
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn events_matching() {
        let mut state = BusState::new();
        state.publish_event(BusEvent::text("field.tick", "a", 0));
        state.publish_event(BusEvent::text("sphere.registered", "b", 0));
        state.publish_event(BusEvent::text("field.decision", "c", 0));
        let matched = state.events_matching("field.*");
        assert_eq!(matched.len(), 2);
    }

    #[test]
    fn events_bounded() {
        let mut state = BusState::new();
        for i in 0..1500 {
            state.publish_event(BusEvent::text("test", "d", i));
        }
        assert!(state.event_count() <= MAX_EVENTS);
    }

    // ── BusState: subscribers ──

    #[test]
    fn add_subscriber() {
        let mut state = BusState::new();
        let sub = make_subscriber("alpha");
        assert!(state.add_subscriber(sub).is_ok());
        assert_eq!(state.subscriber_count(), 1);
    }

    #[test]
    fn remove_subscriber() {
        let mut state = BusState::new();
        let sub = make_subscriber("alpha");
        state.add_subscriber(sub).unwrap();
        let removed = state.remove_subscriber("session-alpha");
        assert!(removed.is_some());
        assert_eq!(state.subscriber_count(), 0);
    }

    #[test]
    fn remove_nonexistent_subscriber() {
        let mut state = BusState::new();
        assert!(state.remove_subscriber("nonexistent").is_none());
    }

    #[test]
    fn matching_subscribers() {
        let mut state = BusState::new();
        let mut sub1 = make_subscriber("a");
        sub1.patterns = vec!["field.*".into()];
        let mut sub2 = make_subscriber("b");
        sub2.patterns = vec!["sphere.*".into()];
        state.add_subscriber(sub1).unwrap();
        state.add_subscriber(sub2).unwrap();
        let matches = state.matching_subscribers("field.tick");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn update_subscriptions() {
        let mut state = BusState::new();
        let sub = make_subscriber("a");
        state.add_subscriber(sub).unwrap();
        let count = state
            .update_subscriptions("session-a", vec!["field.*".into(), "sphere.*".into()])
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn update_subscriptions_not_found() {
        let mut state = BusState::new();
        assert!(state
            .update_subscriptions("nonexistent", vec!["*".into()])
            .is_err());
    }

    // ── BusState: cascade ──

    #[test]
    fn cascade_rate_within_limit() {
        let mut state = BusState::new();
        for _ in 0..CASCADE_RATE_LIMIT {
            assert!(state.check_cascade_rate().is_ok());
        }
    }

    #[test]
    fn cascade_rate_exceeded() {
        let mut state = BusState::new();
        for _ in 0..CASCADE_RATE_LIMIT {
            state.check_cascade_rate().ok();
        }
        assert!(state.check_cascade_rate().is_err());
    }

    #[test]
    fn cascade_count() {
        let mut state = BusState::new();
        assert_eq!(state.cascade_count(), 0);
        state.check_cascade_rate().ok();
        assert_eq!(state.cascade_count(), 1);
    }

    #[test]
    fn cascade_rate_reset() {
        let mut state = BusState::new();
        state.check_cascade_rate().ok();
        state.reset_cascade_rate();
        assert_eq!(state.cascade_count(), 0);
    }

    // ── Full lifecycle ──

    #[test]
    fn full_task_lifecycle() {
        let mut state = BusState::new();
        let id = state.submit_task(make_task("lifecycle test")).unwrap();
        assert_eq!(state.pending_tasks().len(), 1);

        state.claim_task(&id, pid("worker")).unwrap();
        assert!(state.pending_tasks().is_empty());
        assert_eq!(state.tasks_by_status(TaskStatus::Claimed).len(), 1);

        state.complete_task(&id).unwrap();
        assert_eq!(state.tasks_by_status(TaskStatus::Completed).len(), 1);
    }

    #[test]
    fn full_event_subscribe_flow() {
        let mut state = BusState::new();
        let mut sub = make_subscriber("listener");
        sub.patterns = vec!["field.*".into()];
        state.add_subscriber(sub).unwrap();

        state.publish_event(BusEvent::text("field.tick", "r=0.95", 1));
        let matching = state.matching_subscribers("field.tick");
        assert_eq!(matching.len(), 1);
    }

    #[test]
    fn task_not_found_error() {
        let mut state = BusState::new();
        let fake = TaskId::from_existing("nonexistent");
        assert!(state.claim_task(&fake, pid("x")).is_err());
        assert!(state.complete_task(&fake).is_err());
        assert!(state.fail_task(&fake).is_err());
    }
}
