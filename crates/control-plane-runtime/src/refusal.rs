//! Refused model input. The host declines what the model asked for, says why, and the run
//! continues through Commission's `Refused` outcome. Confinement (symlinks, `.git`, root
//! changes), stale goals or authority, storage, cancellation, provider and process-launch
//! failures are not refusals and stay fatal.
use crate::read_request::{PATH_HELP, ReadPathSyntax};
use serde::Serialize;
use std::{fmt, sync::Mutex};

/// The consecutive refusal that ends an attempt; an admitted action resets the count.
pub const CONSECUTIVE_REFUSAL_LIMIT: usize = 5;

#[derive(Debug, Clone)]
pub struct InputRefusal {
    pub code: &'static str,
    pub reason: String,
    pub instruction: &'static str,
}

impl InputRefusal {
    pub fn new(code: &'static str, reason: impl Into<String>, instruction: &'static str) -> Self {
        Self {
            code,
            reason: reason.into(),
            instruction,
        }
    }

    /// The bounded observation the model receives.
    pub fn observation(&self) -> String {
        #[derive(Serialize)]
        struct Observation<'a> {
            code: &'a str,
            reason: String,
            instruction: &'a str,
            effect: &'a str,
        }
        serde_json::to_string(&Observation {
            code: self.code,
            reason: crate::context::excerpt(&self.reason, 1024),
            instruction: self.instruction,
            effect: "refused; this action changed nothing and is not evidence",
        })
        .expect("refusal observation is serializable")
    }
}

impl fmt::Display for InputRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.reason)
    }
}
impl std::error::Error for InputRefusal {}

/// The refusal a host error stands for, or `None` when the error must stay fatal.
pub fn refusal_of(error: &anyhow::Error) -> Option<InputRefusal> {
    if let Some(refusal) = error.downcast_ref::<InputRefusal>() {
        return Some(refusal.clone());
    }
    if let Some(syntax) = error.downcast_ref::<ReadPathSyntax>() {
        return Some(InputRefusal::new(
            "read_path_syntax",
            syntax.to_string(),
            PATH_HELP,
        ));
    }
    if let Some(limit) = error.downcast_ref::<crate::process::ProcessLimit>() {
        return Some(InputRefusal::new(
            limit.code(),
            limit.to_string(),
            "The command ran and was stopped at this limit; its output is not evidence. Narrow the command or choose another.",
        ));
    }
    None
}

/// Counts refusals since the last admitted action in one attempt.
#[derive(Default)]
pub struct RefusalBudget(Mutex<usize>);

impl RefusalBudget {
    pub fn admitted(&self) {
        if let Ok(mut count) = self.0.lock() {
            *count = 0;
        }
    }

    /// Count one refusal; the `CONSECUTIVE_REFUSAL_LIMIT`th ends the attempt.
    pub fn refused(&self, refusal: &InputRefusal) -> anyhow::Result<usize> {
        let mut count = self
            .0
            .lock()
            .map_err(|_| anyhow::anyhow!("refusal budget poisoned"))?;
        *count += 1;
        anyhow::ensure!(
            *count < CONSECUTIVE_REFUSAL_LIMIT,
            "refusal budget exhausted after {} consecutive refused actions; last refusal {refusal}",
            *count
        );
        Ok(*count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifth_consecutive_refusal_ends_the_attempt_and_admission_resets() {
        let budget = RefusalBudget::default();
        let refusal = InputRefusal::new("write_outside_scope", "tests/x.rs", "stay in scope");
        for expected in 1..CONSECUTIVE_REFUSAL_LIMIT {
            assert_eq!(budget.refused(&refusal).unwrap(), expected);
        }
        budget.admitted();
        for _ in 1..CONSECUTIVE_REFUSAL_LIMIT {
            budget.refused(&refusal).unwrap();
        }
        let exhausted = budget.refused(&refusal).unwrap_err().to_string();
        assert!(exhausted.contains("5 consecutive"), "{exhausted}");
        assert!(exhausted.contains("write_outside_scope"), "{exhausted}");
    }

    #[test]
    fn only_typed_input_errors_are_refusals() {
        let syntax = anyhow::Error::new(crate::read_request::parse("../private").unwrap_err());
        assert_eq!(refusal_of(&syntax).unwrap().code, "read_path_syntax");
        let fatal = anyhow::anyhow!("symlink paths are not tool inputs");
        assert!(refusal_of(&fatal).is_none());
        let wrapped = anyhow::Error::new(InputRefusal::new("too_many_reads", "33", "read fewer"))
            .context("implementation step");
        assert_eq!(refusal_of(&wrapped).unwrap().code, "too_many_reads");
    }
}
