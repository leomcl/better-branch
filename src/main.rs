mod prompts;
mod theme;

use crate::prompts::Menu;
use crate::theme::colorful::ColorfulTheme;
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

    let current_branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get current branch");

    if !current_branch.status.success() {
        eprintln!("Error getting current branch");
        exit(1);
    }

    let current_branch = String::from_utf8_lossy(&current_branch.stdout).trim().to_string();

    let selection = Menu::with_theme(&ColorfulTheme::default())
        .default(0)
        .items(&branches)
        .current_branch(current_branch)
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
