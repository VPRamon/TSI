use polars::prelude::*;
use anyhow::Result;

/// Compute mean and std (sample, ddof=1) using Polars.
pub fn analyze_values(values: &[f64]) -> Result<(f64, f64)> {
    if values.is_empty() {
        anyhow::bail!("empty input");
    }

    let s = Series::new("values", values);
    let df = DataFrame::new(vec![s])?;

    // Compute mean
    let mean_df = df.mean();
    let mean = mean_df
        .column("values")?
        .f64()?
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("failed to compute mean"))?;

    // Compute std (sample std with ddof=1)
    let std_df = df.std(1);
    let std_val = std_df
        .column("values")?
        .f64()?
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("failed to compute std"))?;

    Ok((mean, std_val))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_values() {
        let vals = vec![1.0, 2.0, 3.0, 4.0];
        let (mean, std) = analyze_values(&vals).unwrap();
        
        // mean of [1,2,3,4] is 2.5
        assert!((mean - 2.5).abs() < 1e-9, "mean mismatch: {}", mean);
        
        // sample std (ddof=1) of [1,2,3,4] is sqrt(1.666...) ≈ 1.2909944487358056
        assert!((std - 1.2909944487358056).abs() < 1e-9, "std mismatch: {}", std);
    }

    #[test]
    fn test_empty_input() {
        let result = analyze_values(&[]);
        assert!(result.is_err());
    }
}
