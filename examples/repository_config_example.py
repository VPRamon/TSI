#!/usr/bin/env python3
"""
Example demonstrating the new repository configuration system.

This example shows how to use the different repository types (Local vs Azure)
and configuration methods (environment variables, config file, programmatic).
"""

import os
import sys
from pathlib import Path

# Add the parent directory to the path so we can import tsi_rust
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from tsi_rust import tsi_rust_api
except ImportError:
    print("Warning: tsi_rust module not built. Run 'cargo build' first.")
    sys.exit(1)


def example_local_repository():
    """Example: Using local in-memory repository."""
    print("=" * 60)
    print("Example 1: Local Repository (In-Memory)")
    print("=" * 60)
    
    # Set environment to use local repository
    os.environ['REPOSITORY_TYPE'] = 'local'
    
    # In Rust code, this would be:
    # let repo = RepositoryFactory::from_env().await?;
    # or
    # let repo = RepositoryFactory::create_local();
    
    print("✓ Using local in-memory repository")
    print("  - Fast, isolated storage")
    print("  - Perfect for testing and development")
    print("  - Data is not persisted between runs")
    print()


def example_config_file():
    """Example: Using repository.toml configuration file."""
    print("=" * 60)
    print("Example 2: Configuration File")
    print("=" * 60)
    
    # Create example config file
    config_path = Path(__file__).parent.parent / "repository.toml"
    
    if config_path.exists():
        print(f"✓ Found configuration file: {config_path}")
        print(f"  Reading repository settings from file...")
        
        # In Rust code, this would be:
        # let repo = RepositoryFactory::from_config_file("repository.toml").await?;
        # or
        # let repo = RepositoryFactory::from_default_config().await?;
        
        with open(config_path) as f:
            content = f.read()
            if 'type = "local"' in content:
                print("  Configuration: Local repository")
            elif 'type = "azure"' in content:
                print("  Configuration: Azure repository")
    else:
        print(f"✗ No configuration file found at: {config_path}")
        print(f"  Create repository.toml to configure the repository type")
    print()


def example_azure_repository():
    """Example: Using Azure SQL Server repository."""
    print("=" * 60)
    print("Example 3: Azure Repository (Production)")
    print("=" * 60)
    
    # Check if Azure configuration is available
    has_server = os.getenv('DB_SERVER') is not None
    has_database = os.getenv('DB_DATABASE') is not None
    
    if has_server and has_database:
        print("✓ Azure configuration detected in environment")
        print(f"  Server: {os.getenv('DB_SERVER')}")
        print(f"  Database: {os.getenv('DB_DATABASE')}")
        
        # In Rust code, this would be:
        # let config = DbConfig::from_env()?;
        # let repo = RepositoryFactory::create_azure(&config).await?;
        
        print("  - Persistent storage")
        print("  - Suitable for production workloads")
        print("  - Requires network connectivity")
    else:
        print("✗ Azure configuration not found")
        print("  To use Azure repository, set environment variables:")
        print("    - DB_SERVER=<server>.database.windows.net")
        print("    - DB_DATABASE=<database_name>")
        print("    - DB_USERNAME=<username>")
        print("    - DB_PASSWORD=<password>")
    print()


def example_builder_pattern():
    """Example: Using RepositoryBuilder pattern."""
    print("=" * 60)
    print("Example 4: Builder Pattern")
    print("=" * 60)
    
    print("Rust code example:")
    print("""
    // Create with explicit configuration
    let repo = RepositoryBuilder::new()
        .repository_type(RepositoryType::Local)
        .build()
        .await?;
    
    // Create from environment
    let repo = RepositoryBuilder::new()
        .from_env()?
        .build()
        .await?;
    
    // Create from config file
    let repo = RepositoryBuilder::new()
        .from_config_file("repository.toml")?
        .build()
        .await?;
    """)
    print()


def show_configuration_summary():
    """Show current configuration summary."""
    print("=" * 60)
    print("Current Configuration Summary")
    print("=" * 60)
    
    # Check environment variable
    repo_type = os.getenv('REPOSITORY_TYPE', 'not set')
    print(f"Environment: REPOSITORY_TYPE={repo_type}")
    
    # Check config file
    config_path = Path(__file__).parent.parent / "repository.toml"
    if config_path.exists():
        print(f"Config file: {config_path} (exists)")
        with open(config_path) as f:
            for line in f:
                if 'type =' in line:
                    print(f"  {line.strip()}")
                    break
    else:
        print(f"Config file: {config_path} (not found)")
    
    # Azure config
    if os.getenv('DB_SERVER'):
        print(f"Azure: Configured (DB_SERVER set)")
    else:
        print(f"Azure: Not configured")
    
    print()


def main():
    """Run all examples."""
    print("\n" + "=" * 60)
    print("Repository Configuration Examples")
    print("=" * 60)
    print()
    
    show_configuration_summary()
    example_local_repository()
    example_config_file()
    example_azure_repository()
    example_builder_pattern()
    
    print("=" * 60)
    print("For more information, see:")
    print("  - rust_backend/REPOSITORY_CONFIG.md")
    print("  - rust_backend/repository.toml (example)")
    print("  - rust_backend/repository.azure.toml.example")
    print("=" * 60)


if __name__ == "__main__":
    main()
