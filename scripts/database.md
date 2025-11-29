# Database Schema Documentation

This document provides a comprehensive explanation of the database schema used for managing astronomical observing schedules, targets, constraints, and their associated temporal structures.

---

## 1. Overview

The schema defines a structured system for representing observing schedules, astronomical targets, observing constraints, reusable time periods, scheduling blocks, and visibility information. The design emphasizes reusability, normalization, and clear relationships between scheduling metadata and astronomical requirements.

# Test Mermaid

```mermaid
erDiagram

    SCHEDULES {
        BIGINT      schedule_id PK
        NVARCHAR    schedule_name
        DATETIMEOFFSET upload_timestamp
        NVARCHAR    checksum
    }

    TARGETS {
        BIGINT   target_id PK
        NVARCHAR name
        FLOAT    ra_deg
        FLOAT    dec_deg
        FLOAT    ra_pm_masyr
        FLOAT    dec_pm_masyr
        FLOAT    equinox
    }

    ALTITUDE_CONSTRAINTS {
        BIGINT altitude_constraints_id PK
        FLOAT  min_alt_deg
        FLOAT  max_alt_deg
    }

    AZIMUTH_CONSTRAINTS {
        BIGINT azimuth_constraints_id PK
        FLOAT  min_az_deg
        FLOAT  max_az_deg
    }

    CONSTRAINTS {
        BIGINT constraints_id PK
        FLOAT  start_time_mjd
        FLOAT  stop_time_mjd
        BIGINT altitude_constraints_id FK
        BIGINT azimuth_constraints_id  FK
    }

    VISIBILITY_PERIODS {
        BIGINT target_id FK
        BIGINT constraints_id FK
        FLOAT  start_time_mjd
        FLOAT  stop_time_mjd
    }

    SCHEDULING_BLOCKS {
        BIGINT scheduling_block_id PK
        BIGINT target_id          FK
        BIGINT constraints_id     FK
        NUMERIC priority
        INT     min_observation_sec
        INT     requested_duration_sec
    }

    SCHEDULE_DARK_PERIODS {
        BIGINT schedule_id    FK
        FLOAT  start_time_mjd
        FLOAT  stop_time_mjd
    }

    SCHEDULE_SCHEDULING_BLOCKS {
        BIGINT schedule_id         FK
        BIGINT scheduling_block_id FK
        FLOAT  start_time_mjd
        FLOAT  stop_time_mjd
    }

    %% Relationships

    ALTITUDE_CONSTRAINTS ||--o{ CONSTRAINTS : "altitude_constraints_id"
    AZIMUTH_CONSTRAINTS  ||--o{ CONSTRAINTS : "azimuth_constraints_id"

    TARGETS     ||--o{ VISIBILITY_PERIODS : "visible_for"
    CONSTRAINTS ||--o{ VISIBILITY_PERIODS : "under_constraints"

    TARGETS     ||--o{ SCHEDULING_BLOCKS : "requested_on"
    CONSTRAINTS ||--o{ SCHEDULING_BLOCKS : "constrained_by"

    SCHEDULES ||--o{ SCHEDULE_DARK_PERIODS : "has_dark_periods"

    SCHEDULES         ||--o{ SCHEDULE_SCHEDULING_BLOCKS : "includes_block"
    SCHEDULING_BLOCKS ||--o{ SCHEDULE_SCHEDULING_BLOCKS : "scheduled_in"


```