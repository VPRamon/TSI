import pyodbc
from db_credentials import server, database, username, password, driver

connection_string = (
    f"DRIVER={driver};"
    f"SERVER={server};"
    f"DATABASE={database};"
    f"UID={username};"          # full AAD UPN, e.g. ramon.valles@bootcamp-upgrade.com
    f"PWD={password};"          # its password
    "Encrypt=yes;"
    "TrustServerCertificate=no;"
    "Connection Timeout=30;"
    "Authentication=ActiveDirectoryPassword;"
)

# ================================================================
# HELPER FUNCTIONS: GET OR CREATE PATTERN
# ================================================================

def get_or_create_target(cursor, name, ra_deg, dec_deg, ra_pm_masyr=0, dec_pm_masyr=0, equinox=2000.0):
    """
    Get existing target ID or create a new target.
    Returns the target_id.
    """
    # Try to find existing target by natural key
    cursor.execute("""
        SELECT target_id
        FROM dbo.targets
        WHERE ra_deg = ? AND dec_deg = ? 
          AND ra_pm_masyr = ? AND dec_pm_masyr = ?
          AND equinox = ?
    """, (ra_deg, dec_deg, ra_pm_masyr, dec_pm_masyr, equinox))
    
    row = cursor.fetchone()
    if row is not None:
        target_id = row[0]
        print(f"Found existing target_id = {target_id}")
        return target_id
    
    # Insert new target if not found
    cursor.execute("""
        INSERT INTO dbo.targets (name, ra_deg, dec_deg, ra_pm_masyr, dec_pm_masyr, equinox)
        OUTPUT inserted.target_id
        VALUES (?, ?, ?, ?, ?, ?)
    """, (name, ra_deg, dec_deg, ra_pm_masyr, dec_pm_masyr, equinox))
    
    row = cursor.fetchone()
    if row is None:
        raise Exception("Failed to insert target.")
    target_id = row[0]
    print(f"Created new target_id = {target_id}")
    return target_id


def get_or_create_period(cursor, start_time_mjd, stop_time_mjd):
    """
    Get existing period ID or create a new period.
    Returns the period_id.
    """
    # Try to find existing period by natural key
    cursor.execute("""
        SELECT period_id
        FROM dbo.periods
        WHERE start_time_mjd = ? AND stop_time_mjd = ?
    """, (start_time_mjd, stop_time_mjd))
    
    row = cursor.fetchone()
    if row is not None:
        period_id = row[0]
        print(f"Found existing period_id = {period_id}")
        return period_id
    
    # Insert new period if not found
    cursor.execute("""
        INSERT INTO dbo.periods (start_time_mjd, stop_time_mjd)
        OUTPUT inserted.period_id
        VALUES (?, ?)
    """, (start_time_mjd, stop_time_mjd))
    
    row = cursor.fetchone()
    if row is None:
        raise Exception("Failed to insert period.")
    period_id = row[0]
    print(f"Created new period_id = {period_id}")
    return period_id


def get_or_create_altitude_constraint(cursor, min_alt_deg=0, max_alt_deg=90):
    """
    Get existing altitude constraint ID or create a new one.
    Returns the altitude_constraints_id.
    """
    # Try to find existing constraint by natural key
    cursor.execute("""
        SELECT altitude_constraints_id
        FROM dbo.altitude_constraints
        WHERE min_alt_deg = ? AND max_alt_deg = ?
    """, (min_alt_deg, max_alt_deg))
    
    row = cursor.fetchone()
    if row is not None:
        constraint_id = row[0]
        print(f"Found existing altitude_constraints_id = {constraint_id}")
        return constraint_id
    
    # Insert new constraint if not found
    cursor.execute("""
        INSERT INTO dbo.altitude_constraints (min_alt_deg, max_alt_deg)
        OUTPUT inserted.altitude_constraints_id
        VALUES (?, ?)
    """, (min_alt_deg, max_alt_deg))
    
    row = cursor.fetchone()
    if row is None:
        raise Exception("Failed to insert altitude constraint.")
    constraint_id = row[0]
    print(f"Created new altitude_constraints_id = {constraint_id}")
    return constraint_id


def get_or_create_azimuth_constraint(cursor, min_az_deg=0, max_az_deg=360):
    """
    Get existing azimuth constraint ID or create a new one.
    Returns the azimuth_constraints_id.
    """
    # Try to find existing constraint by natural key
    cursor.execute("""
        SELECT azimuth_constraints_id
        FROM dbo.azimuth_constraints
        WHERE min_az_deg = ? AND max_az_deg = ?
    """, (min_az_deg, max_az_deg))
    
    row = cursor.fetchone()
    if row is not None:
        constraint_id = row[0]
        print(f"Found existing azimuth_constraints_id = {constraint_id}")
        return constraint_id
    
    # Insert new constraint if not found
    cursor.execute("""
        INSERT INTO dbo.azimuth_constraints (min_az_deg, max_az_deg)
        OUTPUT inserted.azimuth_constraints_id
        VALUES (?, ?)
    """, (min_az_deg, max_az_deg))
    
    row = cursor.fetchone()
    if row is None:
        raise Exception("Failed to insert azimuth constraint.")
    constraint_id = row[0]
    print(f"Created new azimuth_constraints_id = {constraint_id}")
    return constraint_id


def get_or_create_constraint(cursor, time_constraints_id=None, 
                             altitude_constraints_id=None, 
                             azimuth_constraints_id=None):
    """
    Get existing composite constraint ID or create a new one.
    Returns the constraints_id.
    At least one of the three constraint IDs must be provided.
    """
    if all(x is None for x in [time_constraints_id, altitude_constraints_id, azimuth_constraints_id]):
        raise ValueError("At least one constraint ID must be provided")
    
    # Try to find existing composite constraint by natural key
    cursor.execute("""
        SELECT constraints_id
        FROM dbo.constraints
        WHERE (time_constraints_id IS NULL AND ? IS NULL OR time_constraints_id = ?)
          AND (altitude_constraints_id IS NULL AND ? IS NULL OR altitude_constraints_id = ?)
          AND (azimuth_constraints_id IS NULL AND ? IS NULL OR azimuth_constraints_id = ?)
    """, (time_constraints_id, time_constraints_id,
          altitude_constraints_id, altitude_constraints_id,
          azimuth_constraints_id, azimuth_constraints_id))
    
    row = cursor.fetchone()
    if row is not None:
        constraint_id = row[0]
        print(f"Found existing constraints_id = {constraint_id}")
        return constraint_id
    
    # Insert new composite constraint if not found
    cursor.execute("""
        INSERT INTO dbo.constraints (time_constraints_id, altitude_constraints_id, azimuth_constraints_id)
        OUTPUT inserted.constraints_id
        VALUES (?, ?, ?)
    """, (time_constraints_id, altitude_constraints_id, azimuth_constraints_id))
    
    row = cursor.fetchone()
    if row is None:
        raise Exception("Failed to insert composite constraint.")
    constraint_id = row[0]
    print(f"Created new constraints_id = {constraint_id}")
    return constraint_id


# ================================================================
# SCRIPT: SUBIR UN SCHEDULE MINIMALISTA
# ================================================================
def upload_minimal_schedule():
    try:
        with pyodbc.connect(connection_string, autocommit=False) as conn:
            cursor = conn.cursor()

            # 1. Insert empty schedule (metadata only)
            cursor.execute("""
                INSERT INTO dbo.schedules (checksum)
                OUTPUT inserted.schedule_id
                VALUES ('dummy-test-checksum')
            """)
            row = cursor.fetchone()
            if row is None:
                raise Exception("Failed to insert schedule.")
            schedule_id = row[0]
            print(f"schedule_id = {schedule_id}")

            # 2. Get or create target (using helper function)
            target_id = get_or_create_target(
                cursor, 
                name='Test Target',
                ra_deg=150.0,
                dec_deg=-20.0
            )

            # 3. Get or create period (using helper function)
            period_id = get_or_create_period(
                cursor,
                start_time_mjd=61000.0,
                stop_time_mjd=61000.01
            )

            # 4. Create minimal scheduling block (no constraints)
            cursor.execute("""
                INSERT INTO dbo.scheduling_blocks (
                    target_id,
                    constraints_id,
                    priority,
                    min_observation_sec,
                    requested_duration_sec
                )
                OUTPUT inserted.scheduling_block_id
                VALUES (?, NULL, 1.0, 30, 60)
            """, (target_id,))
            row = cursor.fetchone()
            if row is None:
                raise Exception("Failed to insert scheduling block.")
            scheduling_block_id = row[0]
            print(f"scheduling_block_id = {scheduling_block_id}")

            # 5. Associate scheduling block to schedule
            cursor.execute("""
                INSERT INTO dbo.schedule_scheduling_blocks (
                    schedule_id,
                    scheduling_block_id,
                    scheduled_period_id
                )
                VALUES (?, ?, ?)
            """, (schedule_id, scheduling_block_id, period_id))

            # 6. Commit transaction
            conn.commit()

            print("Schedule minimalista insertado correctamente.")
            return schedule_id
    
    except pyodbc.IntegrityError as e:
        print(f"Database integrity error: {e}")
        raise
    except Exception as e:
        print(f"Error uploading schedule: {e}")
        raise


def get_schedule(schedule_id):
    with pyodbc.connect(connection_string, autocommit=True) as conn:
        cursor = conn.cursor()

        print(f"\n--- SCHEDULE DATA (schedule_id={schedule_id}) ---")

        # 1) Schedule metadata: only select the columns we know
        cursor.execute("""
            SELECT schedule_id, checksum
            FROM dbo.schedules
            WHERE schedule_id = ?
        """, (schedule_id,))
        schedule = cursor.fetchone()
        print("Schedule:")
        print(schedule)

        # 2) Scheduling blocks for this schedule
        cursor.execute("""
            SELECT sb.scheduling_block_id,
                   sb.target_id,
                   sb.constraints_id,
                   sb.priority,
                   sb.min_observation_sec,
                   sb.requested_duration_sec
            FROM dbo.scheduling_blocks sb
            JOIN dbo.schedule_scheduling_blocks ssb
              ON sb.scheduling_block_id = ssb.scheduling_block_id
            WHERE ssb.schedule_id = ?
        """, (schedule_id,))
        blocks = cursor.fetchall()
        print("Scheduling Blocks:")
        for block in blocks:
            print(block)

        # 3) Targets for these blocks
        cursor.execute("""
            SELECT t.target_id,
                   t.name,
                   t.ra_deg,
                   t.dec_deg
            FROM dbo.targets t
            JOIN dbo.scheduling_blocks sb
              ON t.target_id = sb.target_id
            JOIN dbo.schedule_scheduling_blocks ssb
              ON sb.scheduling_block_id = ssb.scheduling_block_id
            WHERE ssb.schedule_id = ?
        """, (schedule_id,))
        targets = cursor.fetchall()
        print("Targets:")
        for target in targets:
            print(target)

        # 4) Periods scheduled for this schedule
        cursor.execute("""
            SELECT p.period_id,
                   p.start_time_mjd,
                   p.stop_time_mjd
            FROM dbo.periods p
            JOIN dbo.schedule_scheduling_blocks ssb
              ON p.period_id = ssb.scheduled_period_id
            WHERE ssb.schedule_id = ?
        """, (schedule_id,))
        periods = cursor.fetchall()
        print("Periods:")
        for period in periods:
            print(period)

        print("--- END SCHEDULE DATA ---\n")



if __name__ == "__main__":
    schedule_id = upload_minimal_schedule()
    get_schedule(schedule_id)
