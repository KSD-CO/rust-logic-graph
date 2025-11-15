use clap::{Parser, Subcommand};
use colored::*;
use rust_logic_graph::{GraphDef, Graph, Orchestrator};
use std::path::PathBuf;
use std::time::Instant;
use anyhow::{Result, Context};

#[derive(Parser)]
#[command(name = "rlg")]
#[command(about = "Rust Logic Graph CLI Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a graph definition file
    Validate {
        /// Path to the graph definition file (JSON or YAML)
        #[arg(short, long)]
        file: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Execute a graph in dry-run mode (without side effects)
    DryRun {
        /// Path to the graph definition file
        #[arg(short, long)]
        file: PathBuf,

        /// Path to the context/input data file (JSON)
        #[arg(short, long)]
        context: Option<PathBuf>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Profile the performance of a graph execution
    Profile {
        /// Path to the graph definition file
        #[arg(short, long)]
        file: PathBuf,

        /// Path to the context/input data file (JSON)
        #[arg(short, long)]
        context: Option<PathBuf>,

        /// Number of iterations to run
        #[arg(short, long, default_value = "1")]
        iterations: usize,
    },

    /// Visualize a graph in ASCII format
    Visualize {
        /// Path to the graph definition file
        #[arg(short, long)]
        file: PathBuf,

        /// Show node details
        #[arg(short, long)]
        details: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { file, verbose } => {
            validate_graph(&file, verbose).await
        }
        Commands::DryRun { file, context, verbose } => {
            dry_run_graph(&file, context, verbose).await
        }
        Commands::Profile { file, context, iterations } => {
            profile_graph(&file, context, iterations).await
        }
        Commands::Visualize { file, details } => {
            visualize_graph(&file, details).await
        }
    }
}

async fn validate_graph(file: &PathBuf, verbose: bool) -> Result<()> {
    println!("{}", "Validating graph...".cyan().bold());

    let graph_def = load_graph_def(file)?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check 1: Verify all nodes have unique IDs
    if verbose {
        println!("  {} Checking node uniqueness...", "✓".green());
    }
    let node_ids: Vec<_> = graph_def.nodes.keys().collect();
    let unique_count = node_ids.len();
    if unique_count != graph_def.nodes.len() {
        errors.push("Duplicate node IDs found".to_string());
    }

    // Check 2: Verify all edges reference existing nodes
    if verbose {
        println!("  {} Checking edge references...", "✓".green());
    }
    for edge in &graph_def.edges {
        if !graph_def.nodes.contains_key(&edge.from) {
            errors.push(format!("Edge references non-existent source node: {}", edge.from));
        }
        if !graph_def.nodes.contains_key(&edge.to) {
            errors.push(format!("Edge references non-existent target node: {}", edge.to));
        }
    }

    // Check 3: Detect cycles (warning, not error)
    if verbose {
        println!("  {} Checking for cycles...", "✓".green());
    }
    if has_cycles(&graph_def) {
        warnings.push("Graph contains cycles - this may cause infinite loops".to_string());
    }

    // Check 4: Find unreachable nodes
    if verbose {
        println!("  {} Checking node reachability...", "✓".green());
    }
    let unreachable = find_unreachable_nodes(&graph_def);
    if !unreachable.is_empty() {
        warnings.push(format!("Unreachable nodes found: {:?}", unreachable));
    }

    // Check 5: Verify at least one node exists
    if graph_def.nodes.is_empty() {
        errors.push("Graph has no nodes".to_string());
    }

    // Print results
    println!();
    if errors.is_empty() && warnings.is_empty() {
        println!("{} Graph is valid!", "✓".green().bold());
        println!("  Nodes: {}", graph_def.nodes.len());
        println!("  Edges: {}", graph_def.edges.len());
    } else {
        if !errors.is_empty() {
            println!("{} Validation errors:", "✗".red().bold());
            for error in &errors {
                println!("  {} {}", "•".red(), error);
            }
        }

        if !warnings.is_empty() {
            println!("{} Warnings:", "⚠".yellow().bold());
            for warning in &warnings {
                println!("  {} {}", "•".yellow(), warning);
            }
        }

        if !errors.is_empty() {
            anyhow::bail!("Graph validation failed");
        }
    }

    Ok(())
}

async fn dry_run_graph(file: &PathBuf, context_file: Option<PathBuf>, verbose: bool) -> Result<()> {
    println!("{}", "Running graph in dry-run mode...".cyan().bold());

    let graph_def = load_graph_def(file)?;
    let mut graph = Graph::new(graph_def);

    // Load context if provided
    if let Some(ctx_file) = context_file {
        if verbose {
            println!("  Loading context from: {}", ctx_file.display());
        }
        let ctx_data = std::fs::read_to_string(&ctx_file)
            .context("Failed to read context file")?;
        let ctx_json: serde_json::Value = serde_json::from_str(&ctx_data)
            .context("Failed to parse context JSON")?;

        if let Some(obj) = ctx_json.as_object() {
            for (key, value) in obj {
                graph.context.set(key, value.clone())?;
            }
        }
    }

    println!();
    println!("{}", "Execution Plan:".bold());
    println!("  Total nodes: {}", graph.def.nodes.len());
    println!("  Total edges: {}", graph.def.edges.len());

    // Show execution order (topological sort)
    if verbose {
        println!();
        println!("{}", "Node execution order:".bold());
        let order = get_execution_order(&graph.def)?;
        for (i, node_id) in order.iter().enumerate() {
            println!("  {}. {} ({:?})", i + 1, node_id, graph.def.nodes.get(node_id).unwrap());
        }
    }

    println!();
    println!("{} Dry-run completed (no actual execution performed)", "✓".green().bold());

    Ok(())
}

async fn profile_graph(file: &PathBuf, context_file: Option<PathBuf>, iterations: usize) -> Result<()> {
    println!("{}", "Profiling graph execution...".cyan().bold());

    let graph_def = load_graph_def(file)?;

    let mut total_duration = std::time::Duration::ZERO;
    let mut min_duration = std::time::Duration::MAX;
    let mut max_duration = std::time::Duration::ZERO;

    println!("  Running {} iteration(s)...", iterations);

    for i in 0..iterations {
        let mut graph = Graph::new(graph_def.clone());

        // Load context if provided
        if let Some(ref ctx_file) = context_file {
            let ctx_data = std::fs::read_to_string(ctx_file)?;
            let ctx_json: serde_json::Value = serde_json::from_str(&ctx_data)?;

            if let Some(obj) = ctx_json.as_object() {
                for (key, value) in obj {
                    graph.context.set(key, value.clone())?;
                }
            }
        }

        let start = Instant::now();

        // Execute graph
        let _ = Orchestrator::execute_graph(&mut graph).await;

        let duration = start.elapsed();

        total_duration += duration;
        min_duration = min_duration.min(duration);
        max_duration = max_duration.max(duration);

        if iterations <= 10 {
            println!("    Iteration {}: {:?}", i + 1, duration);
        }
    }

    let avg_duration = total_duration / iterations as u32;

    println!();
    println!("{}", "Performance Profile:".bold());
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", total_duration);
    println!("  Average time: {:?}", avg_duration);
    println!("  Min time: {:?}", min_duration);
    println!("  Max time: {:?}", max_duration);

    // Calculate throughput
    let ops_per_sec = if avg_duration.as_secs_f64() > 0.0 {
        1.0 / avg_duration.as_secs_f64()
    } else {
        0.0
    };
    println!("  Throughput: {:.2} ops/sec", ops_per_sec);

    Ok(())
}

async fn visualize_graph(file: &PathBuf, details: bool) -> Result<()> {
    println!("{}", "Graph Visualization".cyan().bold());
    println!();

    let graph_def = load_graph_def(file)?;

    // Print nodes
    println!("{}", "Nodes:".bold());
    for (id, node_type) in &graph_def.nodes {
        if details {
            println!("  {} [{}]", id.green(), format!("{:?}", node_type).yellow());
        } else {
            println!("  {}", id.green());
        }
    }

    println!();
    println!("{}", "Edges:".bold());

    // Build adjacency list for better visualization
    let mut adjacency: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for edge in &graph_def.edges {
        adjacency.entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push(edge.to.clone());
    }

    // Print edges in a tree-like structure
    for (from, targets) in &adjacency {
        for (i, to) in targets.iter().enumerate() {
            let connector = if i == targets.len() - 1 { "└─>" } else { "├─>" };
            println!("  {} {} {}", from.cyan(), connector, to.green());
        }
    }

    // ASCII art visualization
    println!();
    println!("{}", "ASCII Graph:".bold());
    draw_ascii_graph(&graph_def);

    println!();
    println!("{}", "Statistics:".bold());
    println!("  Total nodes: {}", graph_def.nodes.len());
    println!("  Total edges: {}", graph_def.edges.len());

    // Calculate in-degree and out-degree
    let mut in_degree: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut out_degree: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for edge in &graph_def.edges {
        *out_degree.entry(edge.from.clone()).or_insert(0) += 1;
        *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
    }

    let entry_points: Vec<_> = graph_def.nodes.keys()
        .filter(|id| in_degree.get(*id).unwrap_or(&0) == &0)
        .collect();
    let exit_points: Vec<_> = graph_def.nodes.keys()
        .filter(|id| out_degree.get(*id).unwrap_or(&0) == &0)
        .collect();

    println!("  Entry points: {} ({:?})", entry_points.len(), entry_points);
    println!("  Exit points: {} ({:?})", exit_points.len(), exit_points);

    Ok(())
}

// Helper functions

fn load_graph_def(file: &PathBuf) -> Result<GraphDef> {
    let content = std::fs::read_to_string(file)
        .context(format!("Failed to read file: {}", file.display()))?;

    let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("");

    let graph_def: GraphDef = match ext {
        "json" => serde_json::from_str(&content)
            .context("Failed to parse JSON")?,
        "yaml" | "yml" => {
            anyhow::bail!("YAML support not yet implemented. Use JSON format.");
        }
        _ => {
            // Try JSON first
            serde_json::from_str(&content)
                .context("Failed to parse as JSON (unknown file extension)")?
        }
    };

    Ok(graph_def)
}

fn has_cycles(graph_def: &GraphDef) -> bool {
    use std::collections::{HashSet, HashMap};

    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph_def.edges {
        adj.entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push(edge.to.clone());
    }

    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    fn dfs(
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if dfs(neighbor, adj, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    for node in graph_def.nodes.keys() {
        if !visited.contains(node) {
            if dfs(node, &adj, &mut visited, &mut rec_stack) {
                return true;
            }
        }
    }

    false
}

fn find_unreachable_nodes(graph_def: &GraphDef) -> Vec<String> {
    use std::collections::{HashSet, HashMap};

    // Find entry points (nodes with no incoming edges)
    let mut has_incoming: HashSet<String> = HashSet::new();
    for edge in &graph_def.edges {
        has_incoming.insert(edge.to.clone());
    }

    let entry_points: Vec<_> = graph_def.nodes.keys()
        .filter(|id| !has_incoming.contains(*id))
        .cloned()
        .collect();

    if entry_points.is_empty() {
        // No entry points, all nodes are potentially unreachable
        return graph_def.nodes.keys().cloned().collect();
    }

    // BFS from all entry points
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph_def.edges {
        adj.entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push(edge.to.clone());
    }

    let mut reachable = HashSet::new();
    let mut queue: Vec<String> = entry_points;

    while let Some(node) = queue.pop() {
        if reachable.contains(&node) {
            continue;
        }
        reachable.insert(node.clone());

        if let Some(neighbors) = adj.get(&node) {
            for neighbor in neighbors {
                if !reachable.contains(neighbor) {
                    queue.push(neighbor.clone());
                }
            }
        }
    }

    graph_def.nodes.keys()
        .filter(|id| !reachable.contains(*id))
        .cloned()
        .collect()
}

fn get_execution_order(graph_def: &GraphDef) -> Result<Vec<String>> {
    use std::collections::{HashMap, VecDeque};

    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();

    // Initialize in-degree
    for node_id in graph_def.nodes.keys() {
        in_degree.insert(node_id.clone(), 0);
    }

    // Build adjacency list and calculate in-degree
    for edge in &graph_def.edges {
        adj.entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push(edge.to.clone());
        *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
    }

    // Topological sort (Kahn's algorithm)
    let mut queue: VecDeque<String> = in_degree.iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| id.clone())
        .collect();

    let mut order = Vec::new();

    while let Some(node) = queue.pop_front() {
        order.push(node.clone());

        if let Some(neighbors) = adj.get(&node) {
            for neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    if order.len() != graph_def.nodes.len() {
        anyhow::bail!("Graph contains cycles, cannot determine execution order");
    }

    Ok(order)
}

fn draw_ascii_graph(graph_def: &GraphDef) {
    use std::collections::HashMap;

    // Build adjacency list
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    for node_id in graph_def.nodes.keys() {
        in_degree.insert(node_id.clone(), 0);
    }

    for edge in &graph_def.edges {
        adj.entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push(edge.to.clone());
        *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
    }

    // Find root nodes (no incoming edges)
    let roots: Vec<_> = graph_def.nodes.keys()
        .filter(|id| in_degree.get(*id).unwrap_or(&0) == &0)
        .cloned()
        .collect();

    // Draw tree-like structure
    fn draw_node(
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashMap<String, bool>,
        prefix: String,
        is_last: bool,
    ) {
        if visited.get(node).copied().unwrap_or(false) {
            println!("{}[{}] (cyclic)", prefix, node.cyan());
            return;
        }

        visited.insert(node.to_string(), true);

        println!("{}{}", prefix, format!("[{}]", node).cyan());

        if let Some(children) = adj.get(node) {
            for (i, child) in children.iter().enumerate() {
                let is_last_child = i == children.len() - 1;
                let new_prefix = if is_last {
                    format!("{}    ", prefix)
                } else {
                    format!("{}│   ", prefix)
                };

                let child_connector = if is_last_child { "└── " } else { "├── " };
                print!("{}{}", new_prefix, child_connector);
                draw_node(child, adj, visited, format!("{}    ", new_prefix), is_last_child);
            }
        }

        visited.insert(node.to_string(), false);
    }

    let mut visited = HashMap::new();

    for (i, root) in roots.iter().enumerate() {
        let is_last = i == roots.len() - 1;
        draw_node(root, &adj, &mut visited, "  ".to_string(), is_last);
    }

    // Handle disconnected nodes
    let all_visited: std::collections::HashSet<_> = visited.keys().cloned().collect();
    let disconnected: Vec<_> = graph_def.nodes.keys()
        .filter(|id| !all_visited.contains(*id))
        .collect();

    if !disconnected.is_empty() {
        println!();
        println!("  {} Disconnected nodes:", "⚠".yellow());
        for node in disconnected {
            println!("    [{}]", node.yellow());
        }
    }
}
