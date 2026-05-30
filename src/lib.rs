//! # lau-reputation
//!
//! Reputation system for the LAU game. Tracks how players are perceived based
//! on their actions, not their words.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ReputationScore
// ---------------------------------------------------------------------------

/// A multi-dimensional reputation score. Every dimension lives in `[-1.0, 1.0]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReputationScore {
    pub helpfulness: f64,
    pub creativity: f64,
    pub reliability: f64,
    pub generosity: f64,
    pub teaching: f64,
    pub conservation: f64,
}

impl ReputationScore {
    /// Create a neutral score (all zeros).
    pub fn neutral() -> Self {
        Self {
            helpfulness: 0.0,
            creativity: 0.0,
            reliability: 0.0,
            generosity: 0.0,
            teaching: 0.0,
            conservation: 0.0,
        }
    }

    /// Weighted overall score.
    pub fn overall(&self) -> f64 {
        // weights: reliability & helpfulness matter a bit more
        self.helpfulness * 0.20
            + self.creativity * 0.15
            + self.reliability * 0.20
            + self.generosity * 0.15
            + self.teaching * 0.15
            + self.conservation * 0.15
    }

    /// Name of the highest-scoring dimension.
    pub fn top_trait(&self) -> String {
        let dims = self.as_pairs();
        dims.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.to_string())
            .unwrap_or_else(|| "helpfulness".to_string())
    }

    /// Name of the lowest-scoring dimension.
    pub fn needs_work(&self) -> String {
        let dims = self.as_pairs();
        dims.iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.to_string())
            .unwrap_or_else(|| "helpfulness".to_string())
    }

    fn as_pairs(&self) -> [(&str, f64); 6] {
        [
            ("helpfulness", self.helpfulness),
            ("creativity", self.creativity),
            ("reliability", self.reliability),
            ("generosity", self.generosity),
            ("teaching", self.teaching),
            ("conservation", self.conservation),
        ]
    }

    /// Clamp every dimension into `[-1.0, 1.0]`.
    fn clamp(&mut self) {
        self.helpfulness = self.helpfulness.clamp(-1.0, 1.0);
        self.creativity = self.creativity.clamp(-1.0, 1.0);
        self.reliability = self.reliability.clamp(-1.0, 1.0);
        self.generosity = self.generosity.clamp(-1.0, 1.0);
        self.teaching = self.teaching.clamp(-1.0, 1.0);
        self.conservation = self.conservation.clamp(-1.0, 1.0);
    }
}

impl Default for ReputationScore {
    fn default() -> Self {
        Self::neutral()
    }
}

// ---------------------------------------------------------------------------
// ReputationAction
// ---------------------------------------------------------------------------

/// The kind of action that can affect reputation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReputationAction {
    HelpedPlayer(String),
    SharedBlueprint(String),
    CompletedQuest(String),
    BrokeConservation(f64),
    TaughtPeer(String, f64),
    GiftedItem(String),
    AbandonedCollab(String),
    CreatedChallenge(String),
    ReceivedPraise(String),
    ReceivedReport(String),
}

// ---------------------------------------------------------------------------
// ReputationEvent
// ---------------------------------------------------------------------------

/// A single recorded reputation event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReputationEvent {
    pub action: ReputationAction,
    pub tick: u64,
    pub witnesses: Vec<String>,
    pub impact: f64,
}

// ---------------------------------------------------------------------------
// ReputationTracker
// ---------------------------------------------------------------------------

/// Central reputation tracker for all players.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationTracker {
    pub scores: HashMap<String, ReputationScore>,
    pub events: HashMap<String, Vec<ReputationEvent>>,
    pub decay_rate: f64,
}

impl ReputationTracker {
    /// Create a new tracker with the given decay rate (0.0 = no decay, 1.0 = instant).
    pub fn new(decay_rate: f64) -> Self {
        Self {
            scores: HashMap::new(),
            events: HashMap::new(),
            decay_rate,
        }
    }

    /// Record an event and update the player's score.
    pub fn record(&mut self, player: &str, event: ReputationEvent) {
        let score = self.scores.entry(player.to_string()).or_default();
        let delta = event.impact;
        match &event.action {
            ReputationAction::HelpedPlayer(_) => {
                score.helpfulness += delta;
            }
            ReputationAction::SharedBlueprint(_) => {
                score.generosity += delta;
                score.creativity += delta * 0.5;
            }
            ReputationAction::CompletedQuest(_) => {
                score.reliability += delta;
            }
            ReputationAction::BrokeConservation(severity) => {
                score.conservation -= severity.abs() * delta;
            }
            ReputationAction::TaughtPeer(_, quality) => {
                score.teaching += quality.abs() * delta;
            }
            ReputationAction::GiftedItem(_) => {
                score.generosity += delta;
            }
            ReputationAction::AbandonedCollab(_) => {
                score.reliability -= delta;
            }
            ReputationAction::CreatedChallenge(_) => {
                score.creativity += delta;
            }
            ReputationAction::ReceivedPraise(_) => {
                score.helpfulness += delta * 0.2;
                score.creativity += delta * 0.2;
                score.reliability += delta * 0.2;
                score.generosity += delta * 0.2;
                score.teaching += delta * 0.2;
                score.conservation += delta * 0.2;
            }
            ReputationAction::ReceivedReport(_) => {
                score.helpfulness -= delta * 0.2;
                score.creativity -= delta * 0.2;
                score.reliability -= delta * 0.2;
                score.generosity -= delta * 0.2;
                score.teaching -= delta * 0.2;
                score.conservation -= delta * 0.2;
            }
        }
        score.clamp();
        self.events
            .entry(player.to_string())
            .or_default()
            .push(event);
    }

    /// Get the current reputation score for a player.
    pub fn get_score(&self, player: &str) -> ReputationScore {
        self.scores
            .get(player)
            .cloned()
            .unwrap_or_default()
    }

    /// Decay all scores toward zero. Reputation must be maintained.
    pub fn decay(&mut self) {
        for score in self.scores.values_mut() {
            score.helpfulness *= 1.0 - self.decay_rate;
            score.creativity *= 1.0 - self.decay_rate;
            score.reliability *= 1.0 - self.decay_rate;
            score.generosity *= 1.0 - self.decay_rate;
            score.teaching *= 1.0 - self.decay_rate;
            score.conservation *= 1.0 - self.decay_rate;
        }
    }

    /// Top `n` players sorted by overall score (descending).
    pub fn top_players(&self, n: usize) -> Vec<(&String, &ReputationScore)> {
        let mut v: Vec<_> = self.scores.iter().collect();
        v.sort_by(|a, b| {
            b.1.overall()
                .partial_cmp(&a.1.overall())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v.truncate(n);
        v
    }

    /// Players whose overall score exceeds the given threshold.
    pub fn trusted_players(&self, threshold: f64) -> Vec<&String> {
        self.scores
            .iter()
            .filter(|(_, s)| s.overall() > threshold)
            .map(|(name, _)| name)
            .collect()
    }

    /// Number of recorded events for a player.
    pub fn action_count(&self, player: &str) -> usize {
        self.events
            .get(player)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Endorsement
// ---------------------------------------------------------------------------

/// A peer endorsement: one player vouches for another's trait.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Endorsement {
    pub from: String,
    pub to: String,
    pub trait_name: String,
    pub tick: u64,
    pub weight: f64,
}

// ---------------------------------------------------------------------------
// EndorsementLog
// ---------------------------------------------------------------------------

/// Log of all peer endorsements.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EndorsementLog {
    pub endorsements: Vec<Endorsement>,
}

impl EndorsementLog {
    /// Create a new empty endorsement log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an endorsement.
    pub fn endorse(&mut self, from: &str, to: &str, trait_name: &str, tick: u64) {
        let weight = 1.0; // default weight
        self.endorsements.push(Endorsement {
            from: from.to_string(),
            to: to.to_string(),
            trait_name: trait_name.to_string(),
            tick,
            weight,
        });
    }

    /// Get all endorsements targeting a player.
    pub fn endorsements_for(&self, player: &str) -> Vec<&Endorsement> {
        self.endorsements
            .iter()
            .filter(|e| e.to == player)
            .collect()
    }

    /// Count of endorsements for a player.
    pub fn endorsement_count(&self, player: &str) -> usize {
        self.endorsements_for(player).len()
    }

    /// The most-endorsed trait for a player, or `None` if no endorsements.
    pub fn most_endorsed_trait(&self, player: &str) -> Option<String> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for e in self.endorsements.iter().filter(|e| e.to == player) {
            *counts.entry(e.trait_name.clone()).or_default() += 1;
        }
        counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(t, _)| t)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tracker() -> ReputationTracker {
        ReputationTracker::new(0.1)
    }

    fn event(action: ReputationAction, impact: f64) -> ReputationEvent {
        ReputationEvent {
            action,
            tick: 100,
            witnesses: vec!["observer".to_string()],
            impact,
        }
    }

    // 1. ReputationScore::neutral is all zeros
    #[test]
    fn neutral_score() {
        let s = ReputationScore::neutral();
        assert_eq!(s.helpfulness, 0.0);
        assert_eq!(s.creativity, 0.0);
        assert_eq!(s.reliability, 0.0);
        assert_eq!(s.generosity, 0.0);
        assert_eq!(s.teaching, 0.0);
        assert_eq!(s.conservation, 0.0);
    }

    // 2. Overall of neutral is zero
    #[test]
    fn overall_neutral() {
        let s = ReputationScore::neutral();
        assert!((s.overall()).abs() < f64::EPSILON);
    }

    // 3. Top trait detection
    #[test]
    fn top_trait() {
        let s = ReputationScore {
            helpfulness: 0.8,
            creativity: 0.3,
            reliability: 0.5,
            generosity: 0.1,
            teaching: 0.2,
            conservation: 0.0,
        };
        assert_eq!(s.top_trait(), "helpfulness");
    }

    // 4. Needs work detection
    #[test]
    fn needs_work() {
        let s = ReputationScore {
            helpfulness: 0.5,
            creativity: 0.3,
            reliability: -0.2,
            generosity: 0.1,
            teaching: 0.2,
            conservation: 0.0,
        };
        assert_eq!(s.needs_work(), "reliability");
    }

    // 5. Score clamps to [-1, 1]
    #[test]
    fn score_clamps() {
        let mut s = ReputationScore {
            helpfulness: 2.0,
            creativity: -3.0,
            reliability: 0.5,
            generosity: 0.1,
            teaching: 0.2,
            conservation: 0.0,
        };
        s.clamp();
        assert_eq!(s.helpfulness, 1.0);
        assert_eq!(s.creativity, -1.0);
    }

    // 6. HelpedPlayer increases helpfulness
    #[test]
    fn helped_player() {
        let mut t = tracker();
        t.record("alice", event(ReputationAction::HelpedPlayer("bob".into()), 0.3));
        let s = t.get_score("alice");
        assert!((s.helpfulness - 0.3).abs() < f64::EPSILON);
    }

    // 7. SharedBlueprint increases generosity and creativity
    #[test]
    fn shared_blueprint() {
        let mut t = tracker();
        t.record(
            "alice",
            event(ReputationAction::SharedBlueprint("bridge".into()), 0.4),
        );
        let s = t.get_score("alice");
        assert!((s.generosity - 0.4).abs() < f64::EPSILON);
        assert!((s.creativity - 0.2).abs() < f64::EPSILON);
    }

    // 8. BrokeConservation decreases conservation
    #[test]
    fn broke_conservation() {
        let mut t = tracker();
        t.record(
            "eve",
            event(ReputationAction::BrokeConservation(0.5), 0.3),
        );
        let s = t.get_score("eve");
        assert!(s.conservation < 0.0);
    }

    // 9. TaughtPeer increases teaching
    #[test]
    fn taught_peer() {
        let mut t = tracker();
        t.record(
            "alice",
            event(ReputationAction::TaughtPeer("bob".into(), 0.8), 0.5),
        );
        let s = t.get_score("alice");
        assert!(s.teaching > 0.0);
    }

    // 10. AbandonedCollab decreases reliability
    #[test]
    fn abandoned_collab() {
        let mut t = tracker();
        t.record(
            "eve",
            event(ReputationAction::AbandonedCollab("project-x".into()), 0.4),
        );
        let s = t.get_score("eve");
        assert!((s.reliability + 0.4).abs() < f64::EPSILON);
    }

    // 11. GiftedItem increases generosity
    #[test]
    fn gifted_item() {
        let mut t = tracker();
        t.record(
            "alice",
            event(ReputationAction::GiftedItem("sword".into()), 0.5),
        );
        let s = t.get_score("alice");
        assert!((s.generosity - 0.5).abs() < f64::EPSILON);
    }

    // 12. ReceivedPraise boosts all dimensions slightly
    #[test]
    fn received_praise() {
        let mut t = tracker();
        t.record(
            "alice",
            event(ReputationAction::ReceivedPraise("great job".into()), 1.0),
        );
        let s = t.get_score("alice");
        assert!(s.helpfulness > 0.0);
        assert!(s.creativity > 0.0);
        assert!(s.reliability > 0.0);
        assert!(s.generosity > 0.0);
        assert!(s.teaching > 0.0);
        assert!(s.conservation > 0.0);
    }

    // 13. ReceivedReport decreases all dimensions slightly
    #[test]
    fn received_report() {
        let mut t = tracker();
        t.record(
            "eve",
            event(ReputationAction::ReceivedReport("griefing".into()), 1.0),
        );
        let s = t.get_score("eve");
        assert!(s.helpfulness < 0.0);
        assert!(s.creativity < 0.0);
        assert!(s.reliability < 0.0);
    }

    // 14. CompletedQuest increases reliability
    #[test]
    fn completed_quest() {
        let mut t = tracker();
        t.record(
            "alice",
            event(ReputationAction::CompletedQuest("dragon".into()), 0.6),
        );
        let s = t.get_score("alice");
        assert!((s.reliability - 0.6).abs() < f64::EPSILON);
    }

    // 15. CreatedChallenge increases creativity
    #[test]
    fn created_challenge() {
        let mut t = tracker();
        t.record(
            "alice",
            event(ReputationAction::CreatedChallenge("maze".into()), 0.7),
        );
        let s = t.get_score("alice");
        assert!((s.creativity - 0.7).abs() < f64::EPSILON);
    }

    // 16. Decay works
    #[test]
    fn decay_reduces_scores() {
        let mut t = tracker();
        t.record("alice", event(ReputationAction::HelpedPlayer("bob".into()), 0.5));
        let before = t.get_score("alice").helpfulness;
        t.decay();
        let after = t.get_score("alice").helpfulness;
        assert!(after < before);
        assert!(after > 0.0);
    }

    // 17. Top players sorted by overall
    #[test]
    fn top_players_ordering() {
        let mut t = tracker();
        t.record("alice", event(ReputationAction::HelpedPlayer("bob".into()), 0.8));
        t.record("carol", event(ReputationAction::HelpedPlayer("dave".into()), 0.2));
        t.record("eve", event(ReputationAction::HelpedPlayer("frank".into()), 0.5));
        let top = t.top_players(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "alice");
        assert_eq!(top[1].0, "eve");
    }

    // 18. Trusted players filtering
    #[test]
    fn trusted_players() {
        let mut t = tracker();
        t.record("alice", event(ReputationAction::HelpedPlayer("bob".into()), 0.8));
        t.record("eve", event(ReputationAction::AbandonedCollab("x".into()), 0.9));
        let trusted = t.trusted_players(0.0);
        assert!(trusted.contains(&&"alice".to_string()));
        assert!(!trusted.contains(&&"eve".to_string()));
    }

    // 19. Action count
    #[test]
    fn action_count() {
        let mut t = tracker();
        assert_eq!(t.action_count("alice"), 0);
        t.record("alice", event(ReputationAction::HelpedPlayer("bob".into()), 0.3));
        t.record("alice", event(ReputationAction::GiftedItem("sword".into()), 0.2));
        assert_eq!(t.action_count("alice"), 2);
    }

    // 20. Get score for unknown player returns neutral
    #[test]
    fn unknown_player_score() {
        let t = tracker();
        let s = t.get_score("nobody");
        assert_eq!(s, ReputationScore::neutral());
    }

    // 21. Endorsement basic operations
    #[test]
    fn endorsement_basic() {
        let mut log = EndorsementLog::new();
        log.endorse("bob", "alice", "helpfulness", 100);
        log.endorse("carol", "alice", "creativity", 101);
        assert_eq!(log.endorsement_count("alice"), 2);
        assert_eq!(log.endorsement_count("bob"), 0);
    }

    // 22. Most endorsed trait
    #[test]
    fn most_endorsed_trait() {
        let mut log = EndorsementLog::new();
        log.endorse("bob", "alice", "helpfulness", 100);
        log.endorse("carol", "alice", "helpfulness", 101);
        log.endorse("dave", "alice", "creativity", 102);
        assert_eq!(log.most_endorsed_trait("alice"), Some("helpfulness".to_string()));
    }

    // 23. Most endorsed trait returns None when no endorsements
    #[test]
    fn most_endorsed_trait_none() {
        let log = EndorsementLog::new();
        assert_eq!(log.most_endorsed_trait("alice"), None);
    }

    // 24. Endorsements for player
    #[test]
    fn endorsements_for_player() {
        let mut log = EndorsementLog::new();
        log.endorse("bob", "alice", "helpfulness", 100);
        log.endorse("carol", "dave", "reliability", 101);
        let a = log.endorsements_for("alice");
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].from, "bob");
    }

    // 25. Multiple decays converge toward zero
    #[test]
    fn multiple_decays() {
        let mut t = ReputationTracker::new(0.5);
        t.record("alice", event(ReputationAction::HelpedPlayer("bob".into()), 1.0));
        for _ in 0..20 {
            t.decay();
        }
        let s = t.get_score("alice");
        assert!(s.helpfulness.abs() < 0.001);
    }

    // 26. Serde round-trip for ReputationScore
    #[test]
    fn serde_score() {
        let s = ReputationScore {
            helpfulness: 0.5,
            creativity: -0.3,
            reliability: 0.1,
            generosity: 0.9,
            teaching: -0.7,
            conservation: 0.0,
        };
        let json = serde_json::to_string(&s).unwrap();
        let s2: ReputationScore = serde_json::from_str(&json).unwrap();
        assert_eq!(s, s2);
    }

    // 27. Serde round-trip for ReputationTracker
    #[test]
    fn serde_tracker() {
        let mut t = tracker();
        t.record("alice", event(ReputationAction::HelpedPlayer("bob".into()), 0.5));
        let json = serde_json::to_string(&t).unwrap();
        let t2: ReputationTracker = serde_json::from_str(&json).unwrap();
        assert_eq!(t2.action_count("alice"), 1);
    }

    // 28. Serde round-trip for EndorsementLog
    #[test]
    fn serde_endorsement_log() {
        let mut log = EndorsementLog::new();
        log.endorse("bob", "alice", "teaching", 42);
        let json = serde_json::to_string(&log).unwrap();
        let log2: EndorsementLog = serde_json::from_str(&json).unwrap();
        assert_eq!(log2.endorsement_count("alice"), 1);
    }
}
