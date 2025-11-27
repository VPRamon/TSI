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
# SCRIPT: SUBIR UN SCHEDULE MINIMALISTA
# ================================================================
def upload_minimal_schedule():
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

        # 2. Insert simple target
        cursor.execute("""
            INSERT INTO dbo.targets (name, ra_deg, dec_deg)
            OUTPUT inserted.target_id
            VALUES ('Test Target', 150.0, -20.0)
        """)
        row = cursor.fetchone()
        if row is None:
            raise Exception("Failed to insert target.")
        target_id = row[0]
        print(f"target_id = {target_id}")

        # 3. Create basic period (e.g. MJD 61000 → 61000.01)
        cursor.execute("""
            INSERT INTO dbo.periods (start_time_mjd, stop_time_mjd)
            OUTPUT inserted.period_id
            VALUES (61000.0, 61000.01)
        """)
        row = cursor.fetchone()
        if row is None:
            raise Exception("Failed to insert period.")
        period_id = row[0]
        print(f"period_id = {period_id}")

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
