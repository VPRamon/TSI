"""Quick test for the correlation insights generation."""

import numpy as np
import pandas as pd

from tsi.services.analytics import generate_correlation_insights

# Create sample correlation matrix
np.random.seed(42)
corr_data = {
    "priority": [1.0, 0.65, -0.15, 0.45],
    "requested_hours": [0.65, 1.0, 0.32, 0.55],
    "elevation_range_deg": [-0.15, 0.32, 1.0, 0.12],
    "total_visibility_hours": [0.45, 0.55, 0.12, 1.0],
}

correlations = pd.DataFrame(
    corr_data,
    index=["priority", "requested_hours", "elevation_range_deg", "total_visibility_hours"],
)

print("Correlation Matrix:")
print(correlations)
print("\n" + "=" * 80 + "\n")

# Generate insights
insights = generate_correlation_insights(correlations)

print("Generated Insights:")
print("\n")
for i, insight in enumerate(insights, 1):
    print(f"{i}. {insight}")
    print()

print("=" * 80)
print(f"Total insights generated: {len(insights)}")
