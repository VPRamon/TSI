#!/usr/bin/env python3
"""
Main entry point for training the observation scheduling explainability model.

Usage:
    python main_train.py
    python main_train.py --config custom_config.yaml
"""

import sys
from pathlib import Path
import argparse

# Add project root to path
project_root = Path(__file__).parent
sys.path.insert(0, str(project_root))

from src.tsi.modeling.train_pipeline import TrainingPipeline


def main():
    """Main execution function."""
    parser = argparse.ArgumentParser(
        description='Train observation scheduling explainability model'
    )
    parser.add_argument(
        '--config',
        type=str,
        help='Path to configuration file',
        default='src/tsi/modeling/config/model_config.yaml'
    )
    parser.add_argument(
        '--output-dir',
        type=str,
        help='Override output directory',
        default=None
    )
    
    args = parser.parse_args()
    
    # Create pipeline
    pipeline = TrainingPipeline(args.config)
    
    # Override output dir if specified
    if args.output_dir:
        pipeline.config.config['paths']['output_dir'] = args.output_dir
    
    # Run training
    print("\n" + "="*80)
    print("STARTING OBSERVATION SCHEDULING EXPLAINABILITY PIPELINE")
    print("="*80 + "\n")
    
    pipeline.run()
    
    print("\n" + "="*80)
    print("PIPELINE COMPLETED SUCCESSFULLY")
    print("="*80 + "\n")
    print(f"Artifacts saved to: {pipeline.config.get_paths()['output_dir']}")
    print("\nTo make predictions:")
    print("  python src/tsi/modeling/inference.py --observation data/new_observation.csv")


if __name__ == '__main__':
    main()
