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
        propagate_subsumptions(branch, world, ce);
    }
    detect_clash(branch);
}

fn propagate_subsumptions(branch: &mut Branch<'_>, world: usize, _trigger: CeId) {
    if branch.clash {
        return;
    }
    loop {
        let mut progressed = false;
        for &(sub, sup) in branch.tbox_subsumptions.clone().iter() {
            if branch.worlds[world].labels.contains(&sub)
                && !branch.worlds[world].labels.contains(&sup)
            {
                let w = &mut branch.worlds[world];
                if w.negated.contains(&sup) {
                    branch.clash = true;
                    return;
                }
                w.labels.insert(sup);
                w.queue.push_back(sup);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
        detect_clash(branch);
        if branch.clash {
            return;
        }
    }
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
