//! APO CLI entry point.

use std::process::ExitCode;

use apo::cli::{Cli, Commands, PacksCommands};
use apo::config::AnalyzerMode;
use apo::packs::dump_builtin_packs;
use apo::report::{
    evidence_json_to_string, json_to_string, pack_json_to_string, render_evidence_llm_prompt,
    render_llm_prompt, render_pack_llm_prompt,
};
use apo::{OutputFormat, analyze_and_write, evidence_and_write, pack_and_write};
use clap::Parser;
use tracing_subscriber::EnvFilter;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();

    let cli = Cli::parse();

    if let Commands::Packs { command } = &cli.command {
        return match command {
            PacksCommands::Dump { output, id } => match dump_builtin_packs(output, id.as_deref()) {
                Ok(written) => {
                    eprintln!("apo: dumped {} built-in pack(s)", written.len());
                    for path in &written {
                        eprintln!("wrote {}", path.display());
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            },
        };
    }

    let config = match cli.into_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };

    match config.mode {
        AnalyzerMode::Hygiene => match analyze_and_write(&config) {
            Ok((report, written)) => {
                if config.prompt_stdout {
                    print!("{}", render_llm_prompt(&report));
                } else if !config.prompt_only {
                    match config.format {
                        OutputFormat::Json => {
                            if let Ok(s) = json_to_string(&report) {
                                println!("{s}");
                            }
                        }
                        OutputFormat::Markdown | OutputFormat::Both | OutputFormat::Sarif => {
                            if let Some(score) = report.policy.overall_score {
                                eprintln!(
                                    "apo: repository hygiene score {:.1}/100 ({} findings, {} gaps)",
                                    score,
                                    report.findings.len(),
                                    report.missing_controls.len()
                                );
                            }
                        }
                    }
                }
                for path in &written {
                    eprintln!("wrote {}", path.display());
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        AnalyzerMode::Evidence => match evidence_and_write(&config) {
            Ok((report, written)) => {
                if config.prompt_stdout {
                    print!("{}", render_evidence_llm_prompt(&report));
                } else if !config.prompt_only {
                    match config.format {
                        OutputFormat::Json => {
                            if let Ok(s) = evidence_json_to_string(&report) {
                                println!("{s}");
                            }
                        }
                        OutputFormat::Markdown | OutputFormat::Both | OutputFormat::Sarif => {
                            eprintln!(
                                "apo: knowledge maturity {:.1}/100 · AI maturity {:.1}/100",
                                report.knowledge_maturity, report.ai_maturity
                            );
                        }
                    }
                }
                for path in &written {
                    eprintln!("wrote {}", path.display());
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        AnalyzerMode::Pack => match pack_and_write(&config) {
            Ok((pack, written)) => {
                if config.prompt_stdout {
                    print!("{}", render_pack_llm_prompt(&pack));
                } else if !config.prompt_only {
                    match config.format {
                        OutputFormat::Json => {
                            if let Ok(s) = pack_json_to_string(&pack) {
                                println!("{s}");
                            }
                        }
                        OutputFormat::Markdown | OutputFormat::Both | OutputFormat::Sarif => {
                            let h = pack
                                .hygiene
                                .policy
                                .overall_score
                                .map(|s| format!("{s:.1}"))
                                .unwrap_or_else(|| "n/a".into());
                            eprintln!(
                                "apo: pack hygiene {h}/100 · knowledge {:.1}/100 · AI {:.1}/100 ({})",
                                pack.evidence.knowledge_maturity,
                                pack.evidence.ai_maturity,
                                pack.evidence_schema
                            );
                            if let Some(diff) = &pack.diff {
                                eprintln!("apo: baseline {}", diff.summary);
                            }
                        }
                    }
                }
                for path in &written {
                    eprintln!("wrote {}", path.display());
                }
                if let Some(threshold) = config.fail_on_score {
                    if let Some(score) = pack.hygiene.policy.overall_score {
                        if score < threshold {
                            eprintln!(
                                "error: hygiene score {score:.1} is below --fail-on-score {threshold}"
                            );
                            return ExitCode::FAILURE;
                        }
                    }
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
    }
}
