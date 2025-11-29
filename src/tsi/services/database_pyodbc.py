"""
Python-based database operations using pyodbc with Azure AD authentication.
This works where tiberius Rust driver doesn't support AAD password auth.
"""

import pyodbc
from typing import List, Dict, Optional, Tuple
import json
from datetime import datetime

def get_connection():
    """Get a connection to Azure SQL Database with AAD authentication."""
    import sys
    import os
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../../scripts'))
    from db_credentials import server, database, username, password, driver
    
    connection_string = (
        f"DRIVER={driver};"
        f"SERVER={server};"
        f"DATABASE={database};"
        f"UID={username};"
        f"PWD={password};"
        "Encrypt=yes;"
        "TrustServerCertificate=no;"
        "Connection Timeout=30;"
        "Authentication=ActiveDirectoryPassword;"
    )
    
    return pyodbc.connect(connection_string, timeout=30)


def health_check() -> bool:
    """Check if database connection is healthy."""
    try:
        conn = get_connection()
        cursor = conn.cursor()
        cursor.execute("SELECT 1")
        cursor.fetchone()
        conn.close()
        return True
    except Exception as e:
        print(f"Health check failed: {e}")
        return False


def list_schedules() -> List[Dict]:
    """List all schedules in the database."""
    conn = get_connection()
    cursor = conn.cursor()
    
    cursor.execute("""
        SELECT schedule_id, schedule_name, 
               CAST(upload_timestamp AS VARCHAR(50)) AS upload_timestamp, 
               checksum
        FROM dbo.schedules
        ORDER BY upload_timestamp DESC
    """)
    
    schedules = []
    for row in cursor.fetchall():
        schedules.append({
            'schedule_id': row[0],
            'schedule_name': row[1] if row[1] else f"Schedule {row[0]}",
            'upload_timestamp': row[2],
            'checksum': row[3]
        })
    
    conn.close()
    return schedules


def fetch_schedule_by_id(schedule_id: int) -> Optional[List[Dict]]:
    """Fetch schedule data by ID."""
    conn = get_connection()
    cursor = conn.cursor()
    
    cursor.execute("""
        SELECT sb.scheduling_block_id, t.name, t.ra_deg, t.dec_deg,
               sb.requested_duration_sec, sb.priority
        FROM dbo.scheduling_blocks sb
        JOIN dbo.targets t ON sb.target_id = t.target_id
        JOIN dbo.schedule_scheduling_blocks ssb ON sb.scheduling_block_id = ssb.scheduling_block_id
        WHERE ssb.schedule_id = ?
    """, (schedule_id,))
    
    blocks = []
    for row in cursor.fetchall():
        blocks.append({
            'scheduling_block_id': row[0],
            'name': row[1],
            'ra_deg': row[2],
            'dec_deg': row[3],
            'duration_min': row[4],
            'priority': row[5]
        })
    
    conn.close()
    return blocks if blocks else None


def fetch_schedule_by_name(schedule_name: str) -> Optional[List[Dict]]:
    """Fetch schedule data by name."""
    conn = get_connection()
    cursor = conn.cursor()
    
    cursor.execute("""
        SELECT sb.scheduling_block_id, t.name, t.ra_deg, t.dec_deg,
               sb.duration_min, sb.priority
        FROM dbo.scheduling_blocks sb
        JOIN dbo.targets t ON sb.target_id = t.target_id
        JOIN dbo.schedules s ON sb.schedule_id = s.schedule_id
        WHERE s.schedule_name = ?
    """, (schedule_name,))
    
    blocks = []
    for row in cursor.fetchall():
        blocks.append({
            'scheduling_block_id': row[0],
            'name': row[1],
            'ra_deg': row[2],
            'dec_deg': row[3],
            'duration_min': row[4],
            'priority': row[5]
        })
    
    conn.close()
    return blocks if blocks else None


def fetch_dark_periods(schedule_id: Optional[int] = None) -> List[Tuple[float, float]]:
    """Fetch dark periods for a schedule or global dark periods."""
    conn = get_connection()
    cursor = conn.cursor()
    
    if schedule_id:
        cursor.execute("""
            SELECT p.start_time_mjd, p.stop_time_mjd
            FROM dbo.dark_periods dp
            JOIN dbo.periods p ON dp.period_id = p.period_id
            WHERE dp.schedule_id = ? OR dp.schedule_id IS NULL
            ORDER BY p.start_time_mjd
        """, (schedule_id,))
    else:
        cursor.execute("""
            SELECT p.start_time_mjd, p.stop_time_mjd
            FROM dbo.dark_periods dp
            JOIN dbo.periods p ON dp.period_id = p.period_id
            WHERE dp.schedule_id IS NULL
            ORDER BY p.start_time_mjd
        """)
    
    periods = [(row[0], row[1]) for row in cursor.fetchall()]
    conn.close()
    return periods


if __name__ == "__main__":
    # Test the connection
    print("Testing Python database connection...")
    
    if health_check():
        print("✅ Health check passed")
        
        schedules = list_schedules()
        print(f"✅ Found {len(schedules)} schedules")
        
        for sched in schedules[:3]:
            print(f"   - {sched['schedule_name']} (ID: {sched['schedule_id']})")
    else:
        print("❌ Health check failed")
