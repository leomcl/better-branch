mod prompts;
mod theme;

use crate::prompts::Menu;
use crate::theme::colorful::ColorfulTheme;
use console::{Key, Term};
use std::process::{exit, Command};

const PROTECTED_BRANCHES: &[&str] = &["main", "master", "develop", "trunk"];

fn is_protected_branch(branch: &str) -> bool {
    PROTECTED_BRANCHES.contains(&branch)
}

fn get_branches() -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()
        .map_err(|e| format!("Failed to execute git branch: {}", e))?;

    if !output.status.success() {
        return Err("Error getting branches".to_string());
    }

    let branches_str = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = branches_str
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(branches)
}

fn get_current_branch() -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| format!("Failed to get current branch: {}", e))?;

    if !output.status.success() {
        return Err("Error getting current branch".to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn delete_branch(branch: &str, force: bool) -> Result<(), String> {
    let mut cmd = Command::new("git");
    cmd.arg("branch");
    if force {
        cmd.arg("-D");
    } else {
        cmd.arg("-d");
    }
    cmd.arg(branch);

    let output = cmd.output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

fn prompt_force_delete_unmerged(term: &Term, unmerged: &[String]) -> bool {
    let count = unmerged.len();
    let msg = format!(
        "{} branch{} unmerged changes. Force delete? (y/N)",
        count,
        if count == 1 { " has" } else { "s have" }
    );

    let _ = term.write_line(&msg);
    let _ = term.flush();

    loop {
        if let Ok(key) = term.read_key() {
            match key {
                Key::Char('y' | 'Y') => {
                    let _ = term.clear_last_lines(1);
                    return true;
                }
                Key::Enter | Key::Char('n' | 'N') | Key::Escape => {
                    let _ = term.clear_last_lines(1);
                    return false;
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let branches = match get_branches() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    if branches.is_empty() {
        println!("No branches found.");
        return;
    }

    let current_branch = match get_current_branch() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    let result = Menu::with_theme(&ColorfulTheme::default())
        .default(0)
        .items(&branches)
        .current_branch(current_branch.clone())
        .vim_mode(true)
        .interact_for_delete();

    let delete_result = match result {
        Ok(Some(r)) => r,
        Ok(None) => return,
        Err(e) => {
            eprintln!("Menu error: {}", e);
            exit(1);
        }
    };

    let term = Term::stderr();

    if delete_result.force {
        // Force delete all selected branches
        for branch in &delete_result.branches {
            if branch == &current_branch {
                eprintln!("Cannot delete current branch: {}", branch);
                continue;
            }
            if is_protected_branch(branch) {
                eprintln!("Skipping protected branch: {}", branch);
                continue;
            }
            match delete_branch(branch, true) {
                Ok(()) => println!("Force deleted: {}", branch),
                Err(e) => eprintln!("Failed to force delete {}: {}", branch, e),
            }
        }
    } else {
        // Safe delete - track unmerged failures
        let mut unmerged = Vec::new();
        let mut deleted = Vec::new();

        for branch in &delete_result.branches {
            if branch == &current_branch {
                eprintln!("Cannot delete current branch: {}", branch);
                continue;
            }
            if is_protected_branch(branch) {
                eprintln!("Skipping protected branch: {}", branch);
                continue;
            }
            match delete_branch(branch, false) {
                Ok(()) => deleted.push(branch.clone()),
                Err(e) => {
                    if e.contains("not fully merged") || e.contains("unmerged") {
                        unmerged.push(branch.clone());
                    } else {
                        eprintln!("Failed to delete {}: {}", branch, e);
                    }
                }
            }
        }

        for branch in &deleted {
            println!("Deleted: {}", branch);
        }

        if !unmerged.is_empty() {
            if prompt_force_delete_unmerged(&term, &unmerged) {
                for branch in &unmerged {
                    if is_protected_branch(branch) {
                        eprintln!("Skipping protected branch: {}", branch);
                        continue;
                    }
                    match delete_branch(branch, true) {
                        Ok(()) => println!("Force deleted: {}", branch),
                        Err(e) => eprintln!("Failed to force delete {}: {}", branch, e),
                    }
                }
            } else {
                println!("Skipped {} unmerged branch(es).", unmerged.len());
            }
        }
    }
}
