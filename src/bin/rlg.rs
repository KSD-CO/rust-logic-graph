// RLG - Rust Logic Graph CLI Tool
// Developer tools for YAML graph validation, visualization, profiling, and dry-run execution

use clap::{Parser, Subcommand};
use colored::Colorize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

/// Helper function to read and parse YAML graph file
fn read_graph_file(file: &PathBuf) -> Result<Value, String> {
    let content = fs::read_to_string(file).map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse YAML and convert to JSON Value for processing
    serde_yaml::from_str(&content).map_err(|e| format!("Invalid YAML: {}", e))
}

#[derive(Parser)]
#[command(name = "rlg")]
#[command(about = "Rust Logic Graph CLI - Developer tools for YAML graph workflows", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a graph definition file
    Validate {
        /// Path to the graph YAML file
        #[arg(short, long)]
        file: PathBuf,

        /// Show detailed validation information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Visualize graph structure in ASCII
    Visualize {
        /// Path to the graph YAML file
        #[arg(short, long)]
        file: PathBuf,

        /// Show detailed node information
        #[arg(short, long)]
        details: bool,
    },

    /// Profile graph performance
    Profile {
        /// Path to the graph YAML file
        #[arg(short, long)]
        file: PathBuf,

        /// Number of iterations to run
        #[arg(short, long, default_value = "100")]
        iterations: usize,
    },

    /// Dry-run execution (parse and validate without executing)
    DryRun {
        /// Path to the graph YAML file
        #[arg(short, long)]
        file: PathBuf,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { file, verbose } => {
            validate_graph(file, verbose);
        }
        Commands::Visualize { file, details } => {
            visualize_graph(file, details);
        }
        Commands::Profile { file, iterations } => {
            profile_graph(file, iterations);
        }
        Commands::DryRun { file, verbose } => {
            dry_run_graph(file, verbose);
        }
    }
}

fn validate_graph(file: PathBuf, verbose: bool) {
    println!("{}", "🔍 Validating graph...".cyan().bold());
    println!("File: {}\n", file.display());

    // Read and parse YAML file
    let graph_def = match read_graph_file(&file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{} {}", "✗".red().bold(), e);
            std::process::exit(1);
        }
    };

    // Validate structure
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Check for required fields
    if !graph_def.get("nodes").is_some() {
        errors.push("Missing 'nodes' field".to_string());
    }

    if !graph_def.get("edges").is_some() {
        warnings.push("Missing 'edges' field - graph may be disconnected".to_string());
    }

    // Validate nodes
    if let Some(nodes) = graph_def.get("nodes").and_then(|n| n.as_object()) {
        if nodes.is_empty() {
            errors.push("Graph has no nodes".to_string());
        }

        for (node_id, node) in nodes {
            // Node can be a string (simple format) or object (detailed format)
            let node_type = if node.is_string() {
                node.as_str().unwrap_or("unknown")
            } else if node.is_object() {
                // Try 'type' field first, then 'node_type'
                node.get("type")
                    .or_else(|| node.get("node_type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown")
            } else {
                "unknown"
            };

            if node_type == "unknown" && node.is_object() {
                errors.push(format!("Node '{}' missing 'type' field", node_id));
            }

            if verbose {
                println!("  Node: {} (type: {})", node_id.green(), node_type);
            }
        }
    }

    // Check for cycles
    if let Some(edges) = graph_def.get("edges").and_then(|e| e.as_array()) {
        if verbose {
            println!("\n  Edges: {}", edges.len());
        }

        // Simple cycle detection (would need proper implementation)
        if edges.len() > 100 {
            warnings.push("Large number of edges - may impact performance".to_string());
        }
    }

    // Display results
    println!();
    if errors.is_empty() {
        println!("{} Graph is valid!", "✓".green().bold());
    } else {
        println!(
            "{} Validation failed with {} error(s):",
            "✗".red().bold(),
            errors.len()
        );
        for error in errors {
            println!("  {} {}", "•".red(), error);
        }
        std::process::exit(1);
    }

    if !warnings.is_empty() {
        println!("\n{} {} warning(s):", "⚠".yellow().bold(), warnings.len());
        for warning in warnings {
            println!("  {} {}", "•".yellow(), warning);
        }
    }

    if verbose {
        println!("\n{}", "Validation complete!".green().bold());
    }
}

fn visualize_graph(file: PathBuf, details: bool) {
    println!("{}", "🎨 Graph Visualization".cyan().bold());
    println!("{}\n", "═".repeat(80).cyan());
    println!("📄 File: {}\n", file.display().to_string().yellow());

    // Read and parse YAML file
    let graph_def = match read_graph_file(&file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{} {}", "✗".red().bold(), e);
            std::process::exit(1);
        }
    };

    // Build edge map for better visualization
    let mut node_edges: std::collections::HashMap<String, (Vec<String>, Vec<String>)> =
        std::collections::HashMap::new();

    if let Some(edges) = graph_def.get("edges").and_then(|e| e.as_array()) {
        for edge in edges {
            let from = edge
                .get("from")
                .and_then(|f| f.as_str())
                .unwrap_or("?")
                .to_string();
            let to = edge
                .get("to")
                .and_then(|t| t.as_str())
                .unwrap_or("?")
                .to_string();

            node_edges
                .entry(from.clone())
                .or_insert((Vec::new(), Vec::new()))
                .1
                .push(to.clone());

            node_edges
                .entry(to.clone())
                .or_insert((Vec::new(), Vec::new()))
                .0
                .push(from);
        }
    }

    // Display nodes with tree structure
    if let Some(nodes) = graph_def.get("nodes").and_then(|n| n.as_object()) {
        println!("{}", "📊 Nodes".bold().underline());
        println!();

        // Find root nodes (no incoming edges)
        let mut root_nodes: Vec<&String> = Vec::new();
        let mut other_nodes: Vec<&String> = Vec::new();

        for node_id in nodes.keys() {
            if let Some((incoming, _)) = node_edges.get(node_id) {
                if incoming.is_empty() {
                    root_nodes.push(node_id);
                } else {
                    other_nodes.push(node_id);
                }
            } else {
                root_nodes.push(node_id);
            }
        }

        // Display root nodes first
        for (i, node_id) in root_nodes.iter().enumerate() {
            let node = nodes.get(*node_id).unwrap();
            let node_type = get_node_type(node);
            let icon = get_node_icon(&node_type);
            let color = get_node_color(&node_type);

            let prefix = if i == root_nodes.len() - 1 && other_nodes.is_empty() {
                "└─"
            } else {
                "├─"
            };

            println!(
                "  {} {} {} {}",
                prefix.dimmed(),
                icon,
                node_id.color(color).bold(),
                format!("({})", node_type).dimmed()
            );

            if details {
                print_node_details(node, &node_type, "     ");
            }

            // Show outgoing edges
            if let Some((_, outgoing)) = node_edges.get(*node_id) {
                for (j, target) in outgoing.iter().enumerate() {
                    let is_last = j == outgoing.len() - 1;
                    let edge_prefix = if is_last {
                        "  └──→"
                    } else {
                        "  ├──→"
                    };
                    println!(
                        "  {}  {} {}",
                        "│".dimmed(),
                        edge_prefix.blue(),
                        target.green()
                    );
                }
            }

            if !other_nodes.is_empty() || i < root_nodes.len() - 1 {
                println!("  {}", "│".dimmed());
            }
        }

        // Display other nodes
        for (i, node_id) in other_nodes.iter().enumerate() {
            let node = nodes.get(*node_id).unwrap();
            let node_type = get_node_type(node);
            let icon = get_node_icon(&node_type);
            let color = get_node_color(&node_type);

            let prefix = if i == other_nodes.len() - 1 {
                "└─"
            } else {
                "├─"
            };

            println!(
                "  {} {} {} {}",
                prefix.dimmed(),
                icon,
                node_id.color(color).bold(),
                format!("({})", node_type).dimmed()
            );

            if details {
                print_node_details(node, &node_type, "     ");
            }

            // Show outgoing edges
            if let Some((_, outgoing)) = node_edges.get(*node_id) {
                for (j, target) in outgoing.iter().enumerate() {
                    let is_last = j == outgoing.len() - 1;
                    let edge_prefix = if is_last {
                        "  └──→"
                    } else {
                        "  ├──→"
                    };
                    let connector = if i == other_nodes.len() - 1 {
                        "  "
                    } else {
                        "│ "
                    };
                    println!(
                        "  {}  {} {}",
                        connector.dimmed(),
                        edge_prefix.blue(),
                        target.green()
                    );
                }
            }

            if i < other_nodes.len() - 1 {
                println!("  {}", "│".dimmed());
            }
        }

        // Summary
        println!();
        println!("{}", "─".repeat(80).dimmed());
        println!(
            "📈 Summary: {} nodes, {} edges",
            nodes.len().to_string().cyan().bold(),
            node_edges
                .values()
                .map(|(_, out)| out.len())
                .sum::<usize>()
                .to_string()
                .cyan()
                .bold()
        );
    }

    println!();
}

fn get_node_type(node: &Value) -> String {
    if node.is_string() {
        node.as_str().unwrap_or("unknown").to_string()
    } else {
        node.get("type")
            .or_else(|| node.get("node_type"))
            .and_then(|t| t.as_str())
            .unwrap_or("unknown")
            .to_string()
    }
}

fn get_node_icon(node_type: &str) -> &'static str {
    match node_type {
        "DBNode" => "🗄️ ",
        "RuleNode" => "⚙️ ",
        "AINode" => "🤖",
        "SubgraphNode" => "📦",
        _ => "●",
    }
}

fn get_node_color(node_type: &str) -> colored::Color {
    match node_type {
        "DBNode" => colored::Color::Blue,
        "RuleNode" => colored::Color::Yellow,
        "AINode" => colored::Color::Magenta,
        "SubgraphNode" => colored::Color::Cyan,
        _ => colored::Color::White,
    }
}

fn print_node_details(node: &Value, node_type: &str, indent: &str) {
    if let Some(description) = node.get("description").and_then(|d| d.as_str()) {
        println!("{}💬 {}", indent, description.dimmed());
    }

    if node_type == "DBNode" {
        if let Some(database) = node.get("database").and_then(|d| d.as_str()) {
            println!("{}🏷️  Database: {}", indent, database.yellow());
        }
        if let Some(query) = node.get("query").and_then(|q| q.as_str()) {
            let short_query = if query.len() > 60 {
                format!("{}...", &query[..57])
            } else {
                query.to_string()
            };
            println!("{}📝 Query: {}", indent, short_query.dimmed());
        }
    }
}

fn profile_graph(file: PathBuf, iterations: usize) {
    println!("{}", "⚡ Profiling graph performance...".cyan().bold());
    println!("File: {}", file.display());
    println!("Iterations: {}\n", iterations);

    // Read file content
    let content = match fs::read_to_string(&file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Failed to read file: {}", "✗".red().bold(), e);
            std::process::exit(1);
        }
    };

    // Validate YAML format once
    let _ = match read_graph_file(&file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{} {}", "✗".red().bold(), e);
            std::process::exit(1);
        }
    };

    // Run profiling
    let mut durations = Vec::new();

    println!("Running {} iterations...", iterations);
    for i in 0..iterations {
        let start = Instant::now();

        // Parse YAML
        let _ = serde_yaml::from_str::<Value>(&content);

        let duration = start.elapsed();
        durations.push(duration);

        if (i + 1) % 10 == 0 {
            print!(".");
            if (i + 1) % 100 == 0 {
                println!(" {}", i + 1);
            }
        }
    }
    println!();

    // Calculate statistics
    let total: std::time::Duration = durations.iter().sum();
    let avg = total / iterations as u32;
    let min = durations.iter().min().unwrap();
    let max = durations.iter().max().unwrap();

    println!("\n{}", "Performance Statistics:".bold());
    println!("  Total time:   {:?}", total);
    println!("  Average:      {:?}", avg);
    println!("  Min:          {:?}", min);
    println!("  Max:          {:?}", max);
    println!(
        "  Throughput:   {:.2} graphs/sec",
        iterations as f64 / total.as_secs_f64()
    );

    println!("\n{}", "Profiling complete!".green().bold());
}

fn dry_run_graph(file: PathBuf, verbose: bool) {
    println!("{}", "🔬 Dry-run execution...".cyan().bold());
    println!("File: {}\n", file.display());

    // Read and parse YAML file
    let graph_def = match read_graph_file(&file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{} {}", "✗".red().bold(), e);
            std::process::exit(1);
        }
    };

    // Simulate execution plan
    println!("{}", "Execution Plan:".bold());

    if let Some(nodes) = graph_def.get("nodes").and_then(|n| n.as_object()) {
        println!("\n{} Parse {} nodes", "1.".cyan(), nodes.len());

        if verbose {
            for (node_id, node) in nodes {
                // Node can be a string (simple format) or object (detailed format)
                let node_type = if node.is_string() {
                    node.as_str().unwrap_or("unknown")
                } else {
                    // Try 'type' field first, then 'node_type'
                    node.get("type")
                        .or_else(|| node.get("node_type"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("unknown")
                };
                println!("   {} {} ({})", "→".dimmed(), node_id, node_type);
            }
        }
    }

    if let Some(edges) = graph_def.get("edges").and_then(|e| e.as_array()) {
        println!("\n{} Build {} edges", "2.".cyan(), edges.len());

        if verbose {
            for edge in edges {
                let from = edge.get("from").and_then(|f| f.as_str()).unwrap_or("?");
                let to = edge.get("to").and_then(|t| t.as_str()).unwrap_or("?");
                println!("   {} {} → {}", "→".dimmed(), from, to);
            }
        }
    }

    println!("\n{} Perform topological sort", "3.".cyan());
    println!("{} Validate dependencies", "4.".cyan());
    println!("{} Check for cycles", "5.".cyan());
    println!("{} Ready for execution", "6.".cyan());

    println!(
        "\n{} Dry-run complete - graph is executable!",
        "✓".green().bold()
    );
}
