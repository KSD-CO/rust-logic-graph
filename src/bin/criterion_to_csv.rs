use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Try to find Criterion JSON summaries under target/criterion
    let crit_dir = Path::new("target/criterion");
    let out_path = Path::new("target/criterion_summary.csv");

    if crit_dir.exists() {
        let mut w = File::create(out_path)?;
        writeln!(w, "bench,mean_s,std_err_s,median_s,sample_count")?;

        for entry in fs::read_dir(crit_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() { continue; }
            let name = entry.file_name().into_string().unwrap_or_default();
            let estimates = entry.path().join("estimates.json");
            if estimates.exists() {
                let f = File::open(&estimates)?;
                let reader = BufReader::new(f);
                let v: serde_json::Value = serde_json::from_reader(reader)?;
                // Try to extract relevant fields; be defensive
                if let Some(mean) = v.pointer("/mean/point_estimate") {
                    let mean_s = mean.as_f64().unwrap_or(0.0);
                    let std_err = v.pointer("/std_error/point_estimate").and_then(|s| s.as_f64()).unwrap_or(0.0);
                    let median = v.pointer("/median/point_estimate").and_then(|m| m.as_f64()).unwrap_or(0.0);
                    let n = v.pointer("/sample_size").and_then(|n| n.as_u64()).unwrap_or(0);
                    writeln!(w, "{},{:.6},{:.6},{:.6},{}", name, mean_s, std_err, median, n)?;
                }
            }
        }

        println!("Wrote Criterion summary to {}", out_path.display());
        return Ok(());
    }

    // Fallback: copy manual CSV if it exists
    let manual = Path::new("target/bench_results.csv");
    if manual.exists() {
        fs::copy(manual, out_path)?;
        println!("Copied manual bench results to {}", out_path.display());
        return Ok(());
    }

    anyhow::bail!("No criterion output or manual bench_results.csv found")
}
