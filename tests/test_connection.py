#!/usr/bin/env python3
"""Quick test to verify database connectivity."""

import socket
import sys

try:
    from scripts.db_credentials import server, database, username, password
except ImportError as e:
    print(f"❌ Failed to import credentials: {e}")
    sys.exit(1)

print("📋 Database Configuration:")
print(f"   Server: {server}")
print(f"   Database: {database}")
print(f"   Username: {username}")
print(f"   Password: {'*' * len(password) if password else '<not set>'}")
print()

# Test 1: DNS resolution
print("🔍 Test 1: DNS Resolution")
try:
    hostname = server.split(':')[0]  # Remove port if present
    ip = socket.gethostbyname(hostname)
    print(f"   ✅ Resolved {hostname} to {ip}")
except socket.gaierror as e:
    print(f"   ❌ DNS lookup failed: {e}")
    print("   → Check that server name is correct")
    sys.exit(1)

# Test 2: TCP connectivity on port 1433
print("\n🔌 Test 2: TCP Connection (port 1433)")
try:
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(10)
    result = sock.connect_ex((hostname, 1433))
    sock.close()
    
    if result == 0:
        print(f"   ✅ Port 1433 is reachable")
    else:
        print(f"   ❌ Cannot connect to port 1433 (error code: {result})")
        print("   → Possible causes:")
        print("      - Azure firewall blocking your IP")
        print("      - Port 1433 blocked by local firewall")
        print("      - Server is down or doesn't exist")
        sys.exit(1)
except socket.timeout:
    print(f"   ❌ Connection timeout after 10 seconds")
    print("   → Azure firewall is likely blocking your IP")
    print("   → Add your IP in Azure Portal → SQL Server → Networking")
    sys.exit(1)
except Exception as e:
    print(f"   ❌ Connection error: {e}")
    sys.exit(1)

print("\n✅ Basic connectivity tests passed!")
print("\nNext steps:")
print("   1. Verify credentials are correct in Azure Portal")
print("   2. Try running: ./run_dashboard.sh")
