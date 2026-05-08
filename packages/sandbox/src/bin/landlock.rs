//! Landlock LSM for filesystem restriction (WORKING VERSION)
//! Based on landlock crate example
//! Blocks access to sensitive files like /etc/passwd, ~/.ssh/

use landlock::{
    path_beneath_rules, Access, AccessFs, ABI, RestrictionStatus, Ruleset, RulesetAttr,
    RulesetCreatedAttr, RulesetStatus,
};
use std::error::Error;

/// Apply Landlock rules to restrict filesystem access
/// Strategy: Allow read-write to /tmp and package dir only
/// Block everything else (including /etc/passwd, ~/.ssh/, etc.)
pub fn apply_land_lock(package_path: &str) -> Result<(), Box<dyn Error>> {
    eprintln!("Applying Landlock filesystem restrictions...");

    // Use ABI V1 for compatibility
    let abi = ABI::V1;
    
    // Define paths that are ALLOWED (everything else is denied)
    let allowed_readonly = vec![
        "/usr",           // System binaries and libraries
        "/lib",           // Libraries  
        "/lib64",         // 64-bit libraries
        "/dev/null",      // Null device
        "/dev/urandom",   // Random device
        "/dev/pts",       // PTY devices
        "/dev/zero",      // Zero device
    ];
    
    let allowed_readwrite = vec![
        "/tmp",           // Temp directory
        package_path,     // Package directory
    ];

    // Create ruleset and apply
    let ruleset = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))?;
    
    let ruleset_created = ruleset.create()?;
    
    // Add read-only rules
    let ruleset_with_ro = ruleset_created
        .add_rules(path_beneath_rules(&allowed_readonly, AccessFs::from_read(abi)))?;
    
    // Add read-write rules  
    let ruleset_with_rw = ruleset_with_ro
        .add_rules(path_beneath_rules(&allowed_readwrite, AccessFs::from_all(abi)))?;
    
    // Apply the ruleset
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

/// Check if Landlock is available
pub fn is_land_lock_available() -> bool {
    // Simple check - try to create a ruleset
    Ruleset::default()
        .handle_access(AccessFs::from_read(ABI::V1))
        .is_ok()
}
