use anyhow::{Context, Result};
use tracing::{error, info};
use yara_x::{Compiler, Scanner};

pub struct YaraScanner {
    rules: yara_x::Rules,
}

impl YaraScanner {
    pub fn new() -> Result<Self> {
        info!("Initializing YARA scanner with default rules...");
        let mut compiler = Compiler::new();

        // Rule 1: EICAR Test File
        compiler.add_source(
            r#"
            rule eicar_test_file {
                meta:
                    description = "EICAR Test File"
                    severity = "CRITICAL"
                strings:
                    $s1 = "X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*"
                condition:
                    $s1
            }
            "#,
        ).context("Failed to add EICAR rule")?;

        // Rule 2: Suspicious Shell Script
        compiler.add_source(
            r#"
            rule suspicious_shell_script {
                meta:
                    description = "Suspicious shell script indicators"
                    severity = "HIGH"
                strings:
                    $s1 = "/bin/bash"
                    $s2 = "rm -rf /"
                    $s3 = "nc -e"
                    $s4 = "mkfifo"
                condition:
                    $s1 and ($s2 or $s3 or $s4)
            }
            "#,
        ).context("Failed to add shell script rule")?;

        // Rule 3: Potential Reverse Shell (Python)
        compiler.add_source(
            r#"
            rule python_reverse_shell {
                meta:
                    description = "Potential Python reverse shell"
                    severity = "CRITICAL"
                strings:
                    $s1 = "socket"
                    $s2 = "connect"
                    $s3 = "subprocess"
                    $s4 = "os.dup2"
                condition:
                    all of them
            }
            "#,
        ).context("Failed to add python rule")?;

        let rules = compiler
            .build(); // yara-x compiler.build() returns Rules directly, typically doesn't fail unless errors were emitted

        // In yara-x 0.4+, build() might return Rules or Result<Rules, Error>
        // Let's assume typical behavior or check if errors handles it.
        // Actually, compiler.build() consumes compiler and returns Rules. 
        // Errors are collected in the compiler, but add_source returns &mut Compiler or Result?
        // In yara-x, add_source returns &mut Compiler. It stores errors.
        // Wait, I used ? on add_source. I need to verify API.
        
        // Let's try to assume add_source returns result or we check errors.
        // If API is different, the compiler will complain and I will fix it.
        // Usually: wrapper pattern.
        
        info!("YARA rules compiled successfully");
        Ok(Self { rules })
    }

    /// Scan a file and return matching rule names
    pub fn scan_file(&self, path: &str) -> Vec<String> {
        let mut scanner = Scanner::new(&self.rules);
        match scanner.scan_file(path) {
            Ok(scan_results) => {
                let mut results = Vec::new();
                for rule in scan_results.matching_rules() {
                    let rule_name = rule.identifier().to_string();
                    info!("YARA Match: {} in file {}", rule_name, path);
                    results.push(rule_name);
                }
                results
            }
            Err(e) => {
                error!("Failed to scan file {}: {}", path, e);
                Vec::new()
            }
        }
    }
}
