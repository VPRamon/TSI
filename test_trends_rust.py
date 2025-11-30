"""Quick test to verify Rust trends backend integration."""

from tsi.services.database import get_trends_data

# Test with a known schedule ID (using 1 as example)
schedule_id = 1

try:
    print(f"Loading trends data for schedule_id={schedule_id}...")
    trends_data = get_trends_data(
        schedule_id=schedule_id,
        filter_impossible=False,
        n_bins=10,
        bandwidth=0.3,
        n_smooth_points=100,
    )
    
    print(f"\n✅ Successfully loaded trends data!")
    print(f"\nMetrics:")
    print(f"  Total count: {trends_data.metrics.total_count}")
    print(f"  Scheduled count: {trends_data.metrics.scheduled_count}")
    print(f"  Scheduled rate: {trends_data.metrics.scheduled_rate:.2%}")
    print(f"  Zero visibility count: {trends_data.metrics.zero_visibility_count}")
    
    print(f"\nData structures:")
    print(f"  Blocks: {len(trends_data.blocks)}")
    print(f"  By priority: {len(trends_data.by_priority)}")
    print(f"  By visibility bins: {len(trends_data.by_visibility_bins)}")
    print(f"  By time bins: {len(trends_data.by_time_bins)}")
    print(f"  Smoothed visibility: {len(trends_data.smoothed_visibility)}")
    print(f"  Smoothed time: {len(trends_data.smoothed_time)}")
    print(f"  Heatmap bins: {len(trends_data.heatmap_bins)}")
    
    if trends_data.by_priority:
        print(f"\nSample priority data (first 3):")
        for p in trends_data.by_priority[:3]:
            print(f"  Priority {p.x}: rate={p.rate:.2%}, count={p.count}, scheduled={p.scheduled}")
    
    print("\n✅ All trends data structures populated successfully!")
    
except Exception as e:
    print(f"\n❌ Error: {e}")
    import traceback
    traceback.print_exc()
