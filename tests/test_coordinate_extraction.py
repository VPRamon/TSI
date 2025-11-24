#!/usr/bin/env python3
"""Test coordinate extraction from JSON"""

import json
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent / 'src'))

from core.preprocessing import SchedulePreprocessor

def test_extraction():
    """Test that coordinates are extracted correctly from JSON."""
    
    schedule_path = Path('data/schedule.json')
    
    # Load and preprocess
    preprocessor = SchedulePreprocessor(schedule_path)
    preprocessor.load_data()
    df = preprocessor.extract_dataframe()
    
    # Load raw JSON for comparison
    with open(schedule_path) as f:
        json_data = json.load(f)
    
    # Check first 5 blocks
    print("Comparing JSON vs DataFrame coordinates:")
    print("=" * 80)
    
    for i in range(5):
        sb_json = json_data['SchedulingBlock'][i]
        sb_id = sb_json['schedulingBlockId']
        
        # Get coordinates from JSON
        target = sb_json.get('target', {})
        coords_json = target.get('position_', {}).get('coord', {}).get('celestial', {})
        ra_json = coords_json.get('raInDeg')
        dec_json = coords_json.get('decInDeg')
        target_id_json = target.get('id_')
        target_name_json = target.get('name')
        
        # Get from DataFrame
        df_row = df[df['schedulingBlockId'] == sb_id].iloc[0]
        ra_df = df_row['raInDeg']
        dec_df = df_row['decInDeg']
        target_id_df = df_row.get('targetId')
        target_name_df = df_row.get('targetName')
        
        print(f"\nBlock {sb_id}:")
        print(f"  JSON:      targetId={target_id_json}, targetName={target_name_json}, RA={ra_json:.2f}, Dec={dec_json:.2f}")
        print(f"  DataFrame: targetId={target_id_df}, targetName={target_name_df}, RA={ra_df:.2f}, Dec={dec_df:.2f}")
        
        if abs(ra_json - ra_df) > 0.01 or abs(dec_json - dec_df) > 0.01:
            print(f"  ❌ MISMATCH!")
        else:
            print(f"  ✓ Match")

if __name__ == '__main__':
    test_extraction()
