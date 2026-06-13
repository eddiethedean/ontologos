//! Clash detection for ALC tableau branches.

use ontologos_core::CeId;

use super::Branch;

/// Check direct label/negation clashes and disjointness constraints.
pub fn detect_clash(branch: &mut Branch<'_>) {
    if branch.clash {
        return;
    }
    for world in &branch.worlds {
        for &ce in &world.labels {
            if world.negated.contains(&ce) {
                branch.clash = true;
                return;
            }
        }
        for &(left, right) in &branch.disjoint {
            if world.labels.contains(&left) && world.labels.contains(&right) {
                branch.clash = true;
                return;
            }
        }
    }
}

/// Assert `ce` into a world, detecting immediate clashes.
pub fn assert_label(branch: &mut Branch<'_>, world: usize, ce: CeId) {
    let w = &mut branch.worlds[world];
    if w.negated.contains(&ce) {
        branch.clash = true;
        return;
    }
    if w.labels.insert(ce) {
        w.queue.push_back(ce);
    }
    detect_clash(branch);
}

/// Assert negation of `ce` into a world.
pub fn assert_negation(branch: &mut Branch<'_>, world: usize, ce: CeId) {
    let w = &mut branch.worlds[world];
    if w.labels.contains(&ce) {
        branch.clash = true;
        return;
    }
    w.negated.insert(ce);
    detect_clash(branch);
}
