mod fuzzy_select;

use crate::fuzzy_select::{ColorfulTheme, FuzzySelect};
use std::process::{exit, Command};

fn main() {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()
        .expect("Failed to execute git branch");

    if !output.status.success() {
        eprintln!("Error getting branches");
        exit(1);
    }

    let branches_str = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<&str> = branches_str
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if branches.is_empty() {
        println!("No branches found.");
        return;
    }

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .default(0)
        .items(&branches)
        .vim_mode(true)
        .interact()
        .unwrap();

    let selected_branch = branches[selection];

    println!("Checking out {}...", selected_branch);

    let status = Command::new("git")
        .args(["checkout", selected_branch])
        .status()
        .expect("Failed to execute git checkout");

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}
