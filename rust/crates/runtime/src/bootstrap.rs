#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BootstrapPhase {
    CliEntry,
    FastPathVersion,
    StartupProfiler,
    SystemPromptFastPath,
    ChromeMcpFastPath,
    DaemonWorkerFastPath,
    BridgeFastPath,
    DaemonFastPath,
    BackgroundSessionFastPath,
    TemplateFastPath,
    EnvironmentRunnerFastPath,
    MainRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapPlan {
    phases: Vec<BootstrapPhase>,
}

impl BootstrapPlan {
    #[must_use]
    pub fn fraude_default() -> Self {
        Self::from_phases(vec![
            BootstrapPhase::CliEntry,
            BootstrapPhase::FastPathVersion,
            BootstrapPhase::StartupProfiler,
            BootstrapPhase::SystemPromptFastPath,
            BootstrapPhase::ChromeMcpFastPath,
            BootstrapPhase::DaemonWorkerFastPath,
            BootstrapPhase::BridgeFastPath,
            BootstrapPhase::DaemonFastPath,
            BootstrapPhase::BackgroundSessionFastPath,
            BootstrapPhase::TemplateFastPath,
            BootstrapPhase::EnvironmentRunnerFastPath,
            BootstrapPhase::MainRuntime,
        ])
    }

    #[must_use]
    pub fn from_phases(phases: Vec<BootstrapPhase>) -> Self {
        let mut deduped = Vec::new();
        for phase in phases {
            if !deduped.contains(&phase) {
                deduped.push(phase);
            }
        }
        Self { phases: deduped }
    }

    #[must_use]
    pub fn phases(&self) -> &[BootstrapPhase] {
        &self.phases
    }
}

#[cfg(test)]
mod tests {
    use super::{BootstrapPhase, BootstrapPlan};

    #[test]
    fn default_plan_contains_all_phases_in_order() {
        let plan = BootstrapPlan::fraude_default();
        let phases = plan.phases();

        assert_eq!(phases[0], BootstrapPhase::CliEntry);
        assert_eq!(*phases.last().unwrap(), BootstrapPhase::MainRuntime);
        assert_eq!(phases.len(), 12);
    }

    #[test]
    fn from_phases_deduplicates_repeated_entries() {
        let plan = BootstrapPlan::from_phases(vec![
            BootstrapPhase::CliEntry,
            BootstrapPhase::CliEntry,
            BootstrapPhase::MainRuntime,
            BootstrapPhase::MainRuntime,
        ]);
        assert_eq!(
            plan.phases(),
            &[BootstrapPhase::CliEntry, BootstrapPhase::MainRuntime]
        );
    }

    #[test]
    fn from_phases_empty_produces_empty_plan() {
        let plan = BootstrapPlan::from_phases(vec![]);
        assert!(plan.phases().is_empty());
    }

    #[test]
    fn from_phases_preserves_insertion_order() {
        let plan = BootstrapPlan::from_phases(vec![
            BootstrapPhase::MainRuntime,
            BootstrapPhase::CliEntry,
            BootstrapPhase::StartupProfiler,
        ]);
        assert_eq!(
            plan.phases(),
            &[
                BootstrapPhase::MainRuntime,
                BootstrapPhase::CliEntry,
                BootstrapPhase::StartupProfiler,
            ]
        );
    }

    #[test]
    fn default_plan_has_no_duplicates() {
        let plan = BootstrapPlan::fraude_default();
        let phases = plan.phases();
        let mut seen = std::collections::HashSet::new();
        for phase in phases {
            assert!(seen.insert(phase), "duplicate phase: {phase:?}");
        }
    }
}
