use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct GovernanceChange {
    pub key: String,
    pub value: f64,
    pub rollback_value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Proposed,
    Scheduled,
    Active,
    RolledBack,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct GovernanceProposal {
    pub proposal_id: String,
    pub title: String,
    pub submitted_epoch: u64,
    pub activation_epoch: Option<u64>,
    pub status: ProposalStatus,
    pub change: GovernanceChange,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GovernanceError {
    #[error("proposal not found")]
    ProposalNotFound,
    #[error("invalid delay")]
    InvalidDelay,
    #[error("invalid proposal state")]
    InvalidState,
}

pub struct GovernanceEngine {
    pub current_epoch: u64,
    pub min_activation_delay_epochs: u64,
    next_id: u64,
    proposals: HashMap<String, GovernanceProposal>,
    params: HashMap<String, f64>,
}

impl GovernanceEngine {
    pub fn new(min_activation_delay_epochs: u64) -> Self {
        Self {
            current_epoch: 0,
            min_activation_delay_epochs,
            next_id: 1,
            proposals: HashMap::new(),
            params: HashMap::new(),
        }
    }

    pub fn submit_proposal(&mut self, title: &str, change: GovernanceChange) -> String {
        let id = format!("gov-{:04}", self.next_id);
        self.next_id += 1;

        self.proposals.insert(
            id.clone(),
            GovernanceProposal {
                proposal_id: id.clone(),
                title: title.to_string(),
                submitted_epoch: self.current_epoch,
                activation_epoch: None,
                status: ProposalStatus::Proposed,
                change,
            },
        );

        id
    }

    pub fn schedule_proposal(
        &mut self,
        proposal_id: &str,
        delay_epochs: u64,
    ) -> Result<(), GovernanceError> {
        if delay_epochs < self.min_activation_delay_epochs {
            return Err(GovernanceError::InvalidDelay);
        }

        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
        if proposal.status != ProposalStatus::Proposed {
            return Err(GovernanceError::InvalidState);
        }

        proposal.activation_epoch = Some(self.current_epoch + delay_epochs);
        proposal.status = ProposalStatus::Scheduled;
        Ok(())
    }

    pub fn advance_epochs(&mut self, delta: u64) {
        self.current_epoch = self.current_epoch.saturating_add(delta);
    }

    pub fn activate_due_proposals(&mut self) {
        let now = self.current_epoch;
        let ids = self
            .proposals
            .iter()
            .filter_map(|(id, proposal)| {
                if proposal.status == ProposalStatus::Scheduled
                    && proposal.activation_epoch.unwrap_or(u64::MAX) <= now
                {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for id in ids {
            if let Some(proposal) = self.proposals.get_mut(&id) {
                self.params
                    .insert(proposal.change.key.clone(), proposal.change.value);
                proposal.status = ProposalStatus::Active;
            }
        }
    }

    pub fn rollback_proposal(&mut self, proposal_id: &str) -> Result<(), GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::InvalidState);
        }

        self.params
            .insert(proposal.change.key.clone(), proposal.change.rollback_value);
        proposal.status = ProposalStatus::RolledBack;
        Ok(())
    }

    pub fn get_param(&self, key: &str) -> Option<f64> {
        self.params.get(key).copied()
    }

    pub fn proposal_status(&self, proposal_id: &str) -> Option<ProposalStatus> {
        self.proposals.get(proposal_id).map(|p| p.status)
    }

    pub fn run_rollback_drill(
        &mut self,
        title: &str,
        change: GovernanceChange,
    ) -> Result<bool, GovernanceError> {
        let proposal_id = self.submit_proposal(title, change.clone());
        self.schedule_proposal(&proposal_id, self.min_activation_delay_epochs)?;
        self.advance_epochs(self.min_activation_delay_epochs);
        self.activate_due_proposals();
        self.rollback_proposal(&proposal_id)?;

        let value = self.get_param(&change.key).unwrap_or(f64::NAN);
        Ok((value - change.rollback_value).abs() < 1e-12)
    }
}

impl Default for GovernanceEngine {
    fn default() -> Self {
        Self::new(2)
    }
}

#[cfg(test)]
mod tests {
    use super::{GovernanceChange, GovernanceEngine, GovernanceError, ProposalStatus};

    #[test]
    fn activation_delay_is_enforced() {
        let mut engine = GovernanceEngine::new(3);
        let proposal_id = engine.submit_proposal(
            "raise_quality_floor",
            GovernanceChange {
                key: "quality_floor".to_string(),
                value: 0.8,
                rollback_value: 0.7,
            },
        );

        let err = engine
            .schedule_proposal(&proposal_id, 1)
            .expect_err("delay too small should fail");
        assert_eq!(err, GovernanceError::InvalidDelay);

        engine
            .schedule_proposal(&proposal_id, 3)
            .expect("valid delay should pass");
        engine.advance_epochs(3);
        engine.activate_due_proposals();

        assert_eq!(engine.proposal_status(&proposal_id), Some(ProposalStatus::Active));
        assert_eq!(engine.get_param("quality_floor"), Some(0.8));
    }

    #[test]
    fn rollback_drill_restores_parameter() {
        let mut engine = GovernanceEngine::new(2);
        let ok = engine
            .run_rollback_drill(
                "weight_update",
                GovernanceChange {
                    key: "w1_accuracy".to_string(),
                    value: 0.5,
                    rollback_value: 0.4,
                },
            )
            .expect("drill should succeed");

        assert!(ok);
    }
}
