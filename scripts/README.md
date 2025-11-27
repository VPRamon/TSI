# post-query.py Usage Guide

This script demonstrates how to post (insert) and get (retrieve) a minimal schedule from a SQL Server database using Python and pyodbc. Credentials are stored securely in `scripts/db_credentials.py` (see `.gitignore`).

## Prerequisites
- Python 3.x
- `pyodbc` package (`pip install pyodbc`)
- Valid database credentials in `scripts/db_credentials.py`

## Setup
1. Clone the repository.
2. Install dependencies:
   ```bash
   pip install pyodbc
   ```
3. Create and fill in `scripts/db_credentials.py` with your database info:
   ```python
   server = "<your-server>"
   database = "<your-database>"
   username = "<your-username>"
   password = "<your-password>"
   driver = "{ODBC Driver 18 for SQL Server}"
   ```

## Usage
Run the script to insert a minimal schedule and fetch its details:

```bash
python scripts/post-query.py
```

### What it does
- Inserts a new schedule, target, period, and scheduling block into the database.
- Associates them according to the schema.
- Fetches and prints the schedule, its blocks, targets, and periods.

### Example Output
```
schedule_id = 1
target_id = 1
period_id = 1
scheduling_block_id = 1
Schedule minimalista insertado correctamente.

--- SCHEDULE DATA (schedule_id=1) ---
Schedule: (1, datetime.datetime(...), 'dummy-test-checksum')
Scheduling Blocks:
(...)
Targets:
(...)
Periods:
(...)
--- END SCHEDULE DATA ---
```

## Customization
- Edit the SQL statements in `post-query.py` to insert more complex schedules or query additional data.
- Use the provided functions as templates for other database operations.

## Security
- Credentials are stored in `scripts/db_credentials.py` and ignored by git via `.gitignore`.

## Troubleshooting
- Ensure your database is accessible and the schema matches `scripts/schedule-schema-mmsql.sql`.
- Check for errors in the output for missing drivers or connection issues.

## License
See `LICENSE` in the repository.
