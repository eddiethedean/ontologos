//! Profile → engine route resolution (DIP — no engine crate dependencies).

use ontologos_core::{DetectedProfileKind, EngineKind, Ontology, Profile, ResolvedRoute};

use crate::{OwlProfile, Result, classify_hybrid, detect_profile};

/// Map detected OWL profile to core routing kind.
#[must_use]
pub fn detected_profile_kind(profile: OwlProfile) -> DetectedProfileKind {
    match profile {
        OwlProfile::El => DetectedProfileKind::El,
        OwlProfile::Rl => DetectedProfileKind::Rl,
        OwlProfile::Ql => DetectedProfileKind::Ql,
        OwlProfile::Dl => DetectedProfileKind::Dl,
    }
}

/// Resolve the engine route for a reasoner profile and ontology.
pub fn resolve_route(profile: Profile, ontology: &Ontology) -> Result<ResolvedRoute> {
    match profile {
        Profile::El => Ok(ResolvedRoute::explicit(EngineKind::El)),
        Profile::Rdfs => Ok(ResolvedRoute::explicit(EngineKind::Rdfs)),
        Profile::Rl => Ok(ResolvedRoute::explicit(EngineKind::Rl)),
        Profile::Alc | Profile::Dl | Profile::DlPreview => Ok(ResolvedRoute::explicit(EngineKind::Dl)),
        Profile::Swrl => Ok(ResolvedRoute::explicit(EngineKind::Dl)),
        Profile::Auto => resolve_auto_route(ontology),
    }
}

fn resolve_auto_route(ontology: &Ontology) -> Result<ResolvedRoute> {
    let report = detect_profile(ontology)?;
    let detected = report
        .detected
        .ok_or_else(|| crate::Error::Message("no profile detected".into()))?;
    let detected_kind = detected_profile_kind(detected);

    if detected == OwlProfile::Dl {
        let hybrid = classify_hybrid(ontology)?;
        if hybrid.modules.len() > 1 {
            return Ok(ResolvedRoute::auto(EngineKind::Hybrid, detected_kind));
        }
        let kind = hybrid
            .modules
            .first()
            .map(|module| match module.profile {
                OwlProfile::Dl => EngineKind::Dl,
                OwlProfile::Rl => EngineKind::Rl,
                OwlProfile::El | OwlProfile::Ql => EngineKind::El,
            })
            .unwrap_or(EngineKind::Dl);
        return Ok(ResolvedRoute::auto(kind, detected_kind));
    }

    let kind = match detected {
        OwlProfile::Rl => EngineKind::Rl,
        OwlProfile::El | OwlProfile::Ql => EngineKind::El,
        OwlProfile::Dl => EngineKind::Dl,
    };
    Ok(ResolvedRoute::auto(kind, detected_kind))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_core::Ontology;

    #[test]
    fn explicit_el_route() {
        let ontology = Ontology::default();
        let route = resolve_route(Profile::El, &ontology).expect("route");
        assert_eq!(route.kind, EngineKind::El);
        assert!(route.detected.is_none());
    }

    #[test]
    fn explicit_dl_route() {
        let ontology = Ontology::default();
        let route = resolve_route(Profile::Dl, &ontology).expect("route");
        assert_eq!(route.kind, EngineKind::Dl);
    }

    #[test]
    fn auto_el_fixture_routes_to_el() {
        let ontology = Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .class("http://example.org/B")
            .unwrap()
            .subclass_of("http://example.org/A", "http://example.org/B")
            .unwrap()
            .build()
            .unwrap();
        let route = resolve_route(Profile::Auto, &ontology).expect("route");
        assert_eq!(route.kind, EngineKind::El);
        assert!(matches!(
            route.detected,
            Some(DetectedProfileKind::El) | Some(DetectedProfileKind::Ql)
        ));
    }
}
