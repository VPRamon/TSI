#!/usr/bin/env python3
"""
Database connection string builder for TSI application.

This script reads credentials from db_credentials.py and constructs
the appropriate DATABASE_URL for the Rust backend.
"""

import os
import sys

# Check if we have PostgreSQL or SQL Server credentials
try:
    from db_credentials import server, database, username, password
    
    # Detect database type from server hostname
    if "database.windows.net" in server:
        if "postgres" in server.lower():
            # Azure PostgreSQL
            db_type = "postgresql"
            port = 5432
            # Azure PostgreSQL format: <username>@<servername>
            if "@" not in username:
                # Append server name if needed
                server_short = server.split('.')[0]
                full_username = f"{username}@{server_short}"
            else:
                full_username = username
            
            connection_string = f"{db_type}://{full_username}:{password}@{server}:{port}/{database}?sslmode=require"
        else:
            # Azure SQL Server - need different connection string
            print("⚠️  Detected Azure SQL Server credentials.")
            print("❌ This implementation requires PostgreSQL.")
            print("")
            print("You have two options:")
            print("1. Create an Azure PostgreSQL database")
            print("2. Update the Rust backend to use SQL Server (tiberius driver)")
            print("")
            print("Current credentials are for:")
            print(f"  Server: {server}")
            print(f"  Database: {database}")
            print(f"  Type: Azure SQL Server")
            sys.exit(1)
    else:
        # Generic PostgreSQL or other
        db_type = "postgresql"
        port = 5432
        connection_string = f"{db_type}://{username}:{password}@{server}:{port}/{database}"
    
    # Export for shell
    print(f"export DATABASE_URL='{connection_string}'")
    print("")
    print("# To use in your current shell session, run:")
    print(f"# source <(python3 {sys.argv[0]})")
    print("")
    print("# Or add to your shell profile:")
    print(f"# echo \"export DATABASE_URL='{connection_string}'\" >> ~/.bashrc")

except ImportError as e:
    print(f"❌ Error: Could not import credentials: {e}")
    print("")
    print("Make sure db_credentials.py exists with the following variables:")
    print("  - server")
    print("  - database")
    print("  - username")
    print("  - password")
    sys.exit(1)
except Exception as e:
    print(f"❌ Error building connection string: {e}")
    sys.exit(1)
