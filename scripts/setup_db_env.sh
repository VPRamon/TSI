#!/bin/bash
# Setup environment variables for Azure SQL Server

# Load credentials from db_credentials.py
SERVER=$(python3 -c "from scripts.db_credentials import server; print(server)")
DATABASE=$(python3 -c "from scripts.db_credentials import database; print(database)")
USERNAME=$(python3 -c "from scripts.db_credentials import username; print(username)")
PASSWORD=$(python3 -c "from scripts.db_credentials import password; print(password)")

export DB_SERVER="$SERVER"
export DB_DATABASE="$DATABASE"
export DB_USERNAME="$USERNAME"
export DB_PASSWORD="$PASSWORD"
export DB_PORT="1433"
export DB_TRUST_CERT="true"

echo "✅ Database environment variables set:"
echo "   DB_SERVER=$DB_SERVER"
echo "   DB_DATABASE=$DB_DATABASE"
echo "   DB_USERNAME=$DB_USERNAME"
echo "   DB_PORT=$DB_PORT"
echo "   DB_TRUST_CERT=$DB_TRUST_CERT"
