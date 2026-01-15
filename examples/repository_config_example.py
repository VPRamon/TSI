#!/usr/bin/env python3
"""
Example demonstrating the repository configuration system.

This example shows how to use the different repository types (Local vs Postgres)
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
    os.environ["REPOSITORY_TYPE"] = "local"

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
        print("  Reading repository settings from file...")

        # In Rust code, this would be:
        # let repo = RepositoryFactory::from_config_file("repository.toml").await?;
        # or
        # let repo = RepositoryFactory::from_default_config().await?;

        with open(config_path) as f:
            content = f.read()
            if 'type = "local"' in content:
                print("  Configuration: Local repository")
            elif 'type = "postgres"' in content:
                print("  Configuration: Postgres repository")
    else:
        print(f"✗ No configuration file found at: {config_path}")
        print("  Create repository.toml to configure the repository type")
    print()


def example_postgres_repository():
    """Example: Using Postgres repository."""
    print("=" * 60)
    print("Example 3: Postgres Repository")
    print("=" * 60)

    has_url = os.getenv("DATABASE_URL") is not None or os.getenv("PG_DATABASE_URL") is not None

    if has_url:
        print("✓ Postgres configuration detected in environment")
        print(f"  DATABASE_URL: {os.getenv('DATABASE_URL') or os.getenv('PG_DATABASE_URL')}")

        # In Rust code, this would be:
        # let config = PostgresConfig::from_env()?;
        # let repo = RepositoryFactory::create(RepositoryType::Postgres, Some(&config)).await?;

        print("  - Persistent storage")
        print("  - Suitable for production workloads")
        print("  - Requires network connectivity")
    else:
        print("✗ Postgres configuration not found")
        print("  To use Postgres repository, set environment variables:")
        print("    - DATABASE_URL=postgres://user:pass@host:5432/dbname")
        print("    - or PG_DATABASE_URL=postgres://user:pass@host:5432/dbname")
    print()


def example_builder_pattern():
    """Example: Using RepositoryBuilder pattern."""
    print("=" * 60)
    print("Example 4: Builder Pattern")
    print("=" * 60)

    print("Rust code example:")
    print(
        """
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
    """
    )
    print()


def show_configuration_summary():
    """Show current configuration summary."""
    print("=" * 60)
    print("Current Configuration Summary")
    print("=" * 60)

    repo_type = os.getenv("REPOSITORY_TYPE", "not set")
    print(f"Environment: REPOSITORY_TYPE={repo_type}")

    config_path = Path(__file__).parent.parent / "repository.toml"
    if config_path.exists():
        print(f"Config file: {config_path} (exists)")
        with open(config_path) as f:
            for line in f:
                if "type =" in line:
                    print(f"  {line.strip()}")
                    break
    else:
        print(f"Config file: {config_path} (not found)")

    has_url = os.getenv("DATABASE_URL") or os.getenv("PG_DATABASE_URL")
    if has_url:
        print("Postgres: Configured (DATABASE_URL set)")
    else:
        print("Postgres: Not configured")

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
    example_postgres_repository()
    example_builder_pattern()

    print("=" * 60)
    print("For more information, see:")
    print("  - docs/REPOSITORY_PATTERN.md")
    print("  - backend/repository.toml (example)")
    print("=" * 60)


if __name__ == "__main__":
    main()
