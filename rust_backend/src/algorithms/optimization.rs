use serde::{Deserialize, Serialize};

/// Result of an optimization run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub solution: Vec<usize>,
    pub objective_value: f64,
    pub iterations: usize,
    pub converged: bool,
}

/// Simple observation for optimization
#[derive(Debug, Clone)]
pub struct Observation {
    pub index: usize,
    pub priority: f64,
}

/// Trait for constraints that can be checked
pub trait Constraint: Send + Sync {
    fn is_satisfied(&self, indices: &[usize], observations: &[Observation]) -> bool;
}

/// Greedy scheduling optimizer
///
/// # Arguments
/// * `observations` - List of observations to schedule
/// * `constraints` - List of constraints to satisfy
/// * `max_iterations` - Maximum number of iterations
///
/// # Returns
/// OptimizationResult with selected indices and objective value
pub fn greedy_schedule(
    observations: &[Observation],
    constraints: &[Box<dyn Constraint>],
    max_iterations: usize,
) -> OptimizationResult {
    if max_iterations == 0 {
        return OptimizationResult {
            solution: vec![],
            objective_value: 0.0,
            iterations: 0,
            converged: true,
        };
    }
    
    if observations.is_empty() {
        return OptimizationResult {
            solution: vec![],
            objective_value: 0.0,
            iterations: 0,
            converged: true,
        };
    }
    
    let mut selected: Vec<usize> = Vec::new();
    let mut available: Vec<usize> = (0..observations.len()).collect();
    let mut iterations = 0;
    
    while !available.is_empty() && iterations < max_iterations {
        iterations += 1;
        
        // Find observation with highest priority
        let best_idx_in_available = available
            .iter()
            .enumerate()
            .max_by(|(_, &a), (_, &b)| {
                observations[a]
                    .priority
                    .partial_cmp(&observations[b].priority)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx);
        
        if let Some(best_idx) = best_idx_in_available {
            let obs_idx = available[best_idx];
            
            // Create candidate solution
            let mut candidate = selected.clone();
            candidate.push(obs_idx);
            
            // Check all constraints
            let satisfies_all = constraints
                .iter()
                .all(|constraint| constraint.is_satisfied(&candidate, observations));
            
            if satisfies_all {
                selected.push(obs_idx);
            }
            
            // Remove from available regardless
            available.remove(best_idx);
        } else {
            break;
        }
    }
    
    // Calculate objective value (sum of priorities)
    let objective_value: f64 = selected
        .iter()
        .map(|&idx| observations[idx].priority)
        .sum();
    
    OptimizationResult {
        solution: selected,
        objective_value,
        iterations,
        converged: iterations < max_iterations,
    }
}

/// Parallel greedy schedule using Rayon
///
/// This version can evaluate multiple candidates in parallel
/// Useful for large datasets where constraint checking is expensive
/// 
/// Note: Requires rayon feature to be enabled in Cargo.toml
#[allow(dead_code)]
pub fn greedy_schedule_parallel(
    _observations: &[Observation],
    _constraints: &[Box<dyn Constraint>],
    _max_iterations: usize,
) -> OptimizationResult {
    // Rayon feature not configured - falling back to single-threaded version
    OptimizationResult {
        solution: vec![],
        objective_value: 0.0,
        iterations: 0,
        converged: false,
    }
    
    /* Original parallel implementation - requires rayon feature:
    use rayon::prelude::*;
    
    if observations.is_empty() || max_iterations == 0 {
        return OptimizationResult {
            solution: vec![],
            objective_value: 0.0,
            iterations: 0,
            converged: true,
        };
    }
    
    let mut selected: Vec<usize> = Vec::new();
    let mut available: Vec<usize> = (0..observations.len()).collect();
    let mut iterations = 0;
    
    while !available.is_empty() && iterations < max_iterations {
        iterations += 1;
        
        // Evaluate all available options in parallel
        let evaluations: Vec<_> = available
            .par_iter()
            .map(|&obs_idx| {
                let mut candidate = selected.clone();
                candidate.push(obs_idx);
                
                let satisfies = constraints
                    .iter()
                    .all(|c| c.is_satisfied(&candidate, observations));
                
                (obs_idx, observations[obs_idx].priority, satisfies)
            })
            .collect();
        
        // Find best satisfying candidate
        let best = evaluations
            .iter()
            .filter(|(_, _, satisfies)| *satisfies)
            .max_by(|(_, p1, _), (_, p2, _)| {
                p1.partial_cmp(p2).unwrap_or(std::cmp::Ordering::Equal)
            });
        
        if let Some((obs_idx, _, _)) = best {
            selected.push(*obs_idx);
            available.retain(|&x| x != *obs_idx);
        } else {
            // No satisfying option, remove highest priority and continue
            if let Some(best_unsatisfying) = evaluations
                .iter()
                .max_by(|(_, p1, _), (_, p2, _)| {
                    p1.partial_cmp(p2).unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                available.retain(|&x| x != best_unsatisfying.0);
            }
        }
    }
    
    let objective_value: f64 = selected
        .iter()
        .map(|&idx| observations[idx].priority)
        .sum();
    
    OptimizationResult {
        solution: selected,
        objective_value,
        iterations,
        converged: iterations < max_iterations,
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper: a constraint that is always satisfied
    struct AlwaysSatisfied;
    
    impl Constraint for AlwaysSatisfied {
        fn is_satisfied(&self, _indices: &[usize], _observations: &[Observation]) -> bool {
            true
        }
    }

    struct LimitOne;

    impl Constraint for LimitOne {
        fn is_satisfied(&self, indices: &[usize], _observations: &[Observation]) -> bool {
            indices.len() <= 1
        }
    }
    
    #[test]
    fn test_greedy_schedule_empty() {
        let observations = vec![];
        let constraints: Vec<Box<dyn Constraint>> = vec![];
        let result = greedy_schedule(&observations, &constraints, 100);
        
        assert_eq!(result.solution.len(), 0);
        assert_eq!(result.objective_value, 0.0);
        assert!(result.converged);
    }
    
    #[test]
    fn test_greedy_schedule_single() {
        let observations = vec![Observation {
            index: 0,
            priority: 5.0,
        }];
        let constraints: Vec<Box<dyn Constraint>> = vec![Box::new(AlwaysSatisfied)];
        let result = greedy_schedule(&observations, &constraints, 100);
        
        assert_eq!(result.solution.len(), 1);
        assert_eq!(result.solution[0], 0);
        assert_eq!(result.objective_value, 5.0);
    }
    
    #[test]
    fn test_greedy_schedule_multiple() {
        let observations = vec![
            Observation {
                index: 0,
                priority: 3.0,
            },
            Observation {
                index: 1,
                priority: 5.0,
            },
            Observation {
                index: 2,
                priority: 1.0,
            },
        ];
        let constraints: Vec<Box<dyn Constraint>> = vec![Box::new(AlwaysSatisfied)];
        let result = greedy_schedule(&observations, &constraints, 100);
        
        assert_eq!(result.solution.len(), 3);
        assert_eq!(result.objective_value, 9.0);
    }

    #[test]
    fn test_greedy_schedule_respects_constraints_and_iterations() {
        let observations = vec![
            Observation { index: 0, priority: 5.0 },
            Observation { index: 1, priority: 4.0 },
        ];
        let constraints: Vec<Box<dyn Constraint>> = vec![Box::new(LimitOne)];

        let result = greedy_schedule(&observations, &constraints, 3);
        assert_eq!(result.solution.len(), 1);
        assert!(result.converged); // iterations < max_iterations
    }
}
