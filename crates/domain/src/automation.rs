//! Automation — trigger → condition → action rules.
//!
//! Automations allow the system to react to events or state changes
//! without manual intervention.
//!
//! TODO(M3): Define `Automation` struct:
//!   - `id: AutomationId`
//!   - `name: String`
//!   - `enabled: bool`
//!   - `triggers: Vec<Trigger>`
//!   - `conditions: Vec<Condition>`
//!   - `actions: Vec<Action>`
//!
//! TODO(M3): Define `Trigger` enum (`StateChanged`, `TimePattern`, `Event`, …).
//! TODO(M3): Define `Condition` enum (`StateIs`, `TimeWindow`, …).
//! TODO(M3): Define `Action` enum (`CallService`, `Delay`, …).
