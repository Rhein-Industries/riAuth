//! Shared, bounded safeguards for connector snapshots and reviewed removals.
//! These checks do not authorize an actor or implement reconciliation. Consumers
//! must check live authority and recompute impact before their first mutation.
use crate::{
    agent::{Agent, Principal},
    crypto::{digest, now},
    error::{Error, Result},
    model::User,
    store::Tx,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A controller policy is scoped to one configured connector. Absence of a
/// policy means manual review; a controller never infers an automatic mode.
#[derive(
    schemars::JsonSchema, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum ReconciliationMode {
    #[default]
    ManualReview,
    GuardedAutomatic,
    Automatic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconciliationDecision {
    AwaitingReview,
    Eligible,
}

impl ReconciliationMode {
    /// Bind an explicit automatic policy into a connector fingerprint while
    /// retaining existing manual plan fingerprints across upgrade.
    pub fn fingerprint(self, base: &str) -> Result<String> {
        if self == Self::ManualReview {
            Ok(base.to_owned())
        } else {
            hash(&(base, self))
        }
    }

    /// Guarded automation stops at any removal. Automatic mode may queue a
    /// below-threshold disable, but the shared P03 review floor always wins.
    pub fn decide(self, impact: &RemovalImpact) -> ReconciliationDecision {
        let any_removal = impact.disabled_users > 0
            || impact.missing_users > 0
            || impact.removed_memberships > 0
            || impact.disabled_clients > 0
            || impact.disabled_sources > 0
            || impact.disabled_passwords > 0;
        let floor_review = impact.review_required
            || impact.missing_users > 0
            || impact.removed_memberships > 0
            || impact.disabled_clients > 0
            || impact.disabled_sources > 0
            || impact.disabled_passwords > 0;
        match self {
            Self::ManualReview => ReconciliationDecision::AwaitingReview,
            Self::GuardedAutomatic if any_removal => ReconciliationDecision::AwaitingReview,
            Self::Automatic | Self::GuardedAutomatic if floor_review => {
                ReconciliationDecision::AwaitingReview
            }
            Self::Automatic | Self::GuardedAutomatic => ReconciliationDecision::Eligible,
        }
    }

    pub fn review_reason(self, impact: &RemovalImpact) -> &'static str {
        if self == Self::ManualReview {
            "manual_mode"
        } else if impact.review_required
            || impact.missing_users > 0
            || impact.removed_memberships > 0
            || impact.disabled_clients > 0
            || impact.disabled_sources > 0
            || impact.disabled_passwords > 0
        {
            "removal_review_required"
        } else {
            "guarded_removal"
        }
    }
}

/// Run a local connector plan through the shared controller decision. The
/// connector's existing apply transaction remains the only mutation boundary
/// and must re-fetch its source and revalidate the exact plan and authority.
pub fn reconcile_plan(
    mode: ReconciliationMode,
    impact: &RemovalImpact,
    plan: serde_json::Value,
    apply: impl FnOnce(&str) -> Result<serde_json::Value>,
) -> Result<serde_json::Value> {
    use serde_json::json;
    if mode.decide(impact) == ReconciliationDecision::AwaitingReview {
        let reason = mode.review_reason(impact);
        return Ok(json!({"decision":"awaiting_review","mode":mode,"reason":reason,"plan":plan}));
    }
    let id = plan["id"]
        .as_str()
        .or_else(|| plan["plan_id"].as_str())
        .ok_or_else(|| Error::internal("Connector plan has no ID"))?;
    let result = apply(id)?;
    Ok(json!({"decision":"applied","mode":mode,"plan":plan,"result":result}))
}

#[derive(schemars::JsonSchema, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemovalImpact {
    pub disabled_users: usize,
    pub missing_users: usize,
    pub removed_memberships: usize,
    /// Desired-state access removals. Omitted from existing connector plans.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub disabled_clients: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub disabled_sources: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub disabled_passwords: usize,
    pub review_required: bool,
}
fn is_zero(value: &usize) -> bool {
    *value == 0
}
impl RemovalImpact {
    /// Preserve the cloud guard's conservative policy: every absence or managed
    /// membership removal needs review; explicit suspension needs review for all
    /// active users, >=5 at >=20%, or >=2 at >=50%. Equality crosses a threshold.
    pub fn assess(&mut self, active_users: usize) {
        self.review_required = self.missing_users > 0
            || self.removed_memberships > 0
            || self.disabled_clients > 0
            || self.disabled_sources > 0
            || self.disabled_passwords > 0
            || large_removal(self.disabled_users, active_users);
    }
}
pub fn large_removal(removed: usize, active: usize) -> bool {
    active > 0
        && (removed == active
            || removed >= 5 && removed.saturating_mul(100) >= active.saturating_mul(20)
            || removed >= 2 && removed.saturating_mul(100) >= active.saturating_mul(50))
}

#[derive(schemars::JsonSchema, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewBinding {
    pub content_digest: String,
    pub authority_digest: String,
}
fn authority(tx: &Tx<'_>, actor: &Principal) -> Result<String> {
    let permissions: BTreeSet<_> = actor
        .permissions
        .iter()
        .map(|p| (&p.action, &p.resource))
        .collect();
    let delegation = if let Some(id) = actor.id.strip_prefix("agent:").filter(|_| actor.agent) {
        let agent = tx
            .get::<Agent>("agents", id)?
            .ok_or_else(Error::forbidden)?;
        let parent = agent
            .parent_user
            .as_ref()
            .map(|id| {
                let user = tx.get::<User>("users", id)?.ok_or_else(Error::forbidden)?;
                Ok::<_, Error>((user.id, user.enabled, user.admin, user.epoch))
            })
            .transpose()?;
        serde_json::json!([agent.parent_user, agent.expires_at, parent])
    } else {
        let user = tx
            .get::<User>("users", &actor.id)?
            .ok_or_else(Error::forbidden)?;
        if actor.delegated {
            serde_json::json!([
                user.id,
                user.enabled,
                user.admin,
                user.epoch,
                actor.grants,
                tx.get::<u64>("human_grant_generations", &actor.id)?
                    .unwrap_or(0),
            ])
        } else {
            serde_json::json!([user.id, user.enabled, user.admin, user.epoch])
        }
    };
    hash(&(&actor.id, actor.agent, permissions, delegation))
}

pub fn hash(content: &impl Serialize) -> Result<String> {
    Ok(digest(
        &serde_json::to_string(content).map_err(Error::internal)?,
    ))
}
pub(crate) fn plan_content(plan: &impl Serialize) -> Result<serde_json::Value> {
    let mut content = serde_json::to_value(plan).map_err(Error::internal)?;
    let fields = content
        .as_object_mut()
        .ok_or_else(|| Error::internal("Invalid connector plan"))?;
    fields.remove("review");
    fields.remove("applied");
    Ok(content)
}
impl ReviewBinding {
    pub fn new(tx: &Tx<'_>, actor: &Principal, content: &impl Serialize) -> Result<Self> {
        Ok(Self {
            content_digest: hash(content)?,
            authority_digest: authority(tx, actor)?,
        })
    }
    /// Empty legacy bindings fail closed; a fresh plan is required after upgrade.
    pub fn validate(&self, tx: &Tx<'_>, actor: &Principal, content: &impl Serialize) -> Result<()> {
        if *self != Self::new(tx, actor, content)? {
            return Err(Error::conflict(
                "Connector plan content or authority changed; create and review a new plan",
            ));
        }
        Ok(())
    }
    pub fn confirm(&self, id: &str, impact: &RemovalImpact, reviewed: Option<&str>) -> Result<()> {
        if (impact.review_required || impact.missing_users > 0 || impact.removed_memberships > 0)
            && reviewed != Some(id)
        {
            return Err(Error::conflict(
                "Connector removals require explicit review; confirm this exact plan ID after inspecting removal_impact and changes",
            ));
        }
        Ok(())
    }
}

/// Common apply boundary for connector plans. Callers still authenticate the
/// actor, validate connector-specific source snapshots and ownership, and
/// compare the resulting changes before committing. This gate binds the exact
/// plan, current authority/configuration/revision and recomputed P03 impact.
pub struct ApplyGate<'a> {
    pub id: &'a str,
    pub revision: u64,
    pub expires_at: u64,
    pub fingerprint_matches: bool,
    pub expected_impact: &'a RemovalImpact,
    pub observed_impact: &'a RemovalImpact,
    pub review: &'a ReviewBinding,
    pub reviewed_plan: Option<&'a str>,
}

impl ApplyGate<'_> {
    pub fn validate(&self, tx: &Tx<'_>, actor: &Principal, plan: &impl Serialize) -> Result<()> {
        crate::reconciliation::validate_apply_lease(tx, actor)?;
        if self.expires_at <= now()
            || self.revision != tx.get::<u64>("meta", "revision")?.unwrap_or(0)
            || !self.fingerprint_matches
        {
            return Err(Error::conflict(
                "Connector plan expired or source configuration or local revision changed; create a new plan",
            ));
        }
        self.review.validate(tx, actor, &plan_content(plan)?)?;
        if self.expected_impact != self.observed_impact {
            return Err(Error::conflict(
                "Connector removal impact changed; create a new plan",
            ));
        }
        self.review
            .confirm(self.id, self.observed_impact, self.reviewed_plan)
    }
}

/// Tracks a complete crawl, independently of the peer's transport/parser. Totals
/// are exact only when the protocol promises that (LDAP's estimate is not exact).
pub(crate) struct Pagination {
    pages: usize,
    items: usize,
    max_pages: usize,
    max_items: usize,
    total: Option<usize>,
    seen: BTreeSet<String>,
}
fn incomplete() -> Error {
    Error::new(
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "connector_incomplete_snapshot",
        "Connector snapshot has incomplete, repeated or inconsistent pagination; retry a complete crawl before planning or applying removals",
    )
}
impl Pagination {
    pub fn new(max_pages: usize, max_items: usize) -> Self {
        Self {
            pages: 0,
            items: 0,
            max_pages,
            max_items,
            total: None,
            seen: BTreeSet::new(),
        }
    }
    pub fn page(
        &mut self,
        cursor: &str,
        count: usize,
        total: Option<usize>,
        more: bool,
    ) -> Result<()> {
        if self.pages >= self.max_pages
            || !self.seen.insert(cursor.to_owned())
            || self.items.saturating_add(count) > self.max_items
            || more && count == 0
            || self.pages > 0 && count == 0
        {
            return Err(incomplete());
        }
        if let Some(total) = total {
            if total > self.max_items || self.total.is_some_and(|old| old != total) {
                return Err(incomplete());
            }
            self.total = Some(total);
        }
        self.pages += 1;
        self.items += count;
        if self.total.is_some_and(|total| {
            self.items > total || more && self.items >= total || !more && self.items != total
        }) {
            return Err(incomplete());
        }
        Ok(())
    }
}

#[cfg(test)]
mod reconciliation_tests {
    use super::*;

    #[test]
    fn automatic_modes_keep_the_removal_review_floor() {
        let mut impact = RemovalImpact::default();
        assert_eq!(
            ReconciliationMode::ManualReview.decide(&impact),
            ReconciliationDecision::AwaitingReview
        );
        assert_eq!(
            ReconciliationMode::GuardedAutomatic.decide(&impact),
            ReconciliationDecision::Eligible
        );
        assert_eq!(
            ReconciliationMode::Automatic.decide(&impact),
            ReconciliationDecision::Eligible
        );

        impact.disabled_users = 1;
        assert_eq!(
            ReconciliationMode::GuardedAutomatic.decide(&impact),
            ReconciliationDecision::AwaitingReview
        );
        assert_eq!(
            ReconciliationMode::Automatic.decide(&impact),
            ReconciliationDecision::Eligible
        );
        impact.review_required = true;
        assert_eq!(
            ReconciliationMode::Automatic.decide(&impact),
            ReconciliationDecision::AwaitingReview
        );

        impact = RemovalImpact {
            missing_users: 1,
            ..Default::default()
        };
        assert_eq!(
            ReconciliationMode::Automatic.decide(&impact),
            ReconciliationDecision::AwaitingReview
        );
        assert!(
            ReviewBinding::default()
                .confirm("plan", &impact, None)
                .is_err()
        );
        assert!(
            ReviewBinding::default()
                .confirm("plan", &impact, Some("other"))
                .is_err()
        );
        assert!(
            ReviewBinding::default()
                .confirm("plan", &impact, Some("plan"))
                .is_ok()
        );

        impact = RemovalImpact {
            removed_memberships: 1,
            ..Default::default()
        };
        assert_eq!(
            ReconciliationMode::Automatic.decide(&impact),
            ReconciliationDecision::AwaitingReview
        );
        assert!(
            ReviewBinding::default()
                .confirm("plan", &impact, None)
                .is_err()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn thresholds_include_equality_and_preserve_small_offboarding() {
        for (removed, active, expected) in [
            (0, 0, false),
            (1, 10, false),
            (1, 1, true),
            (4, 25, false),
            (5, 26, false),
            (5, 25, true),
            (2, 5, false),
            (2, 4, true),
        ] {
            assert_eq!(
                large_removal(removed, active),
                expected,
                "{removed}/{active}"
            );
        }
        let mut impact = RemovalImpact {
            missing_users: 1,
            ..Default::default()
        };
        impact.assess(100);
        assert!(impact.review_required);
        impact.missing_users = 0;
        impact.removed_memberships = 1;
        impact.assess(100);
        assert!(impact.review_required);
    }
    #[test]
    fn complete_empty_and_multi_page_results_only() {
        assert!(
            Pagination::new(20, 2000)
                .page("first", 0, Some(0), false)
                .is_ok()
        );
        for (count, total, more) in [(0, Some(2), true), (1, Some(2), false), (2, Some(1), false)] {
            assert!(
                Pagination::new(20, 2000)
                    .page("first", count, total, more)
                    .is_err()
            );
        }
        let mut pages = Pagination::new(20, 2000);
        pages.page("first", 1, Some(2), true).unwrap();
        assert!(pages.page("first", 1, Some(2), false).is_err());
        let mut pages = Pagination::new(20, 2000);
        pages.page("first", 1, Some(2), true).unwrap();
        assert!(pages.page("next", 1, Some(3), false).is_err());
        let mut pages = Pagination::new(20, 2000);
        pages.page("first", 1, Some(2), true).unwrap();
        pages.page("next", 1, Some(2), false).unwrap();
        let mut pages = Pagination::new(1, 1);
        pages.page("first", 1, None, true).unwrap();
        assert!(pages.page("next", 1, None, false).is_err());
    }
}
