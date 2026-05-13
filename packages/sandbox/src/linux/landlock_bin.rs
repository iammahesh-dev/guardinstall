use landlock::{
    path_beneath_rules, Access, AccessFs, ABI, RestrictionStatus, Ruleset, RulesetAttr,
    RulesetCreatedAttr, RulesetStatus,
};
use std::error::Error;

pub fn apply_land_lock(package_path: &str) -> Result<(), Box<dyn Error>> {
    eprintln!("Applying Landlock filesystem restrictions...");

    let abi = ABI::V1;

    let allowed_readonly = vec![
        "/usr",
        "/lib",
        "/lib64",
        "/dev/null",
        "/dev/urandom",
        "/dev/pts",
        "/dev/zero",
    ];

    let allowed_readwrite = vec![
        "/tmp",
        package_path,
    ];

    let ruleset = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))?;

    let ruleset_created = ruleset.create()?;

    let ruleset_with_ro = ruleset_created
        .add_rules(path_beneath_rules(&allowed_readonly, AccessFs::from_read(abi)))?;

    let ruleset_with_rw = ruleset_with_ro
        .add_rules(path_beneath_rules(&allowed_readwrite, AccessFs::from_all(abi)))?;

    let status: RestrictionStatus = ruleset_with_rw.restrict_self()?;

    match status.ruleset {
        RulesetStatus::FullyEnforced => {
            eprintln!("Landlock fully enforced - filesystem access restricted");
        }
        RulesetStatus::PartiallyEnforced => {
            eprintln!("Landlock partially enforced");
        }
        RulesetStatus::NotEnforced => {
            eprintln!("Warning: Landlock not enforced (kernel may not support)");
        }
    }

    Ok(())
}

pub fn is_land_lock_available() -> bool {
    Ruleset::default()
        .handle_access(AccessFs::from_read(ABI::V1))
        .is_ok()
}
