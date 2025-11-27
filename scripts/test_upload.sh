#!/bin/bash

# Test script to validate the upload_schedule binary works correctly
# This creates a minimal test dataset and uploads it

set -e

echo "=== Upload Schedule Test ==="
echo

# Check if password is provided
if [ -z "$DB_PASSWORD" ]; then
    echo "⚠️  Warning: DB_PASSWORD not set. Skipping database connection test."
    echo "   Run with: DB_PASSWORD='password' ./test_upload.sh"
    echo
    TEST_DB=false
else
    TEST_DB=true
fi

# Create test data directory
TEST_DIR="/tmp/schedule_test"
mkdir -p "$TEST_DIR"

echo "1. Creating minimal test JSON files..."

# Create minimal schedule.json
cat > "$TEST_DIR/schedule_test.json" << 'EOF'
{
    "SchedulingBlock": [
        {
            "priority": 5.0,
            "schedulingBlockId": 9999999,
            "target": {
                "id_": 1,
                "name": "Test Target",
                "position_": {
                    "coord": {
                        "celestial": {
                            "decInDeg": -20.0,
                            "decProperMotionInMarcsecYear": 0.0,
                            "equinox": 2000.0,
                            "raInDeg": 150.0,
                            "raProperMotionInMarcsecYear": 0.0
                        }
                    }
                }
            },
            "schedulingBlockConfiguration_": {
                "constraints_": {
                    "azimuthConstraint_": {
                        "maxAzimuthAngleInDeg": 360.0,
                        "minAzimuthAngleInDeg": 0.0
                    },
                    "elevationConstraint_": {
                        "maxElevationAngleInDeg": 90.0,
                        "minElevationAngleInDeg": 30.0
                    },
                    "timeConstraint_": {
                        "minObservationTimeInSec": 60,
                        "requestedDurationSec": 120
                    }
                }
            },
            "scheduled_period": {
                "durationInSec": 100.0,
                "startTime": {
                    "format": "MJD",
                    "scale": "UTC",
                    "value": 61000.0
                },
                "stopTime": {
                    "format": "MJD",
                    "scale": "UTC",
                    "value": 61000.001157
                }
            }
        }
    ]
}
EOF

# Create minimal possible_periods.json
cat > "$TEST_DIR/possible_periods_test.json" << 'EOF'
{
    "SchedulingBlock": {
        "9999999": [
            {
                "durationInSec": 500.0,
                "startTime": {
                    "format": "MJD",
                    "scale": "UTC",
                    "value": 61000.0
                },
                "stopTime": {
                    "format": "MJD",
                    "scale": "UTC",
                    "value": 61000.005787
                }
            },
            {
                "durationInSec": 300.0,
                "startTime": {
                    "format": "MJD",
                    "scale": "UTC",
                    "value": 61001.0
                },
                "stopTime": {
                    "format": "MJD",
                    "scale": "UTC",
                    "value": 61001.003472
                }
            }
        ]
    }
}
EOF

echo "✓ Test files created in $TEST_DIR"
echo

# Validate JSON syntax
echo "2. Validating JSON syntax..."
if command -v python3 &> /dev/null; then
    python3 -m json.tool "$TEST_DIR/schedule_test.json" > /dev/null && echo "✓ schedule_test.json is valid"
    python3 -m json.tool "$TEST_DIR/possible_periods_test.json" > /dev/null && echo "✓ possible_periods_test.json is valid"
else
    echo "⚠️  Python not found, skipping JSON validation"
fi
echo

# Check if binary exists
echo "3. Checking if upload_schedule binary exists..."
BINARY="/workspace/target/release/upload_schedule"
if [ ! -f "$BINARY" ]; then
    echo "❌ Binary not found. Building..."
    cargo build --manifest-path /workspace/rust_backend/Cargo.toml --bin upload_schedule --release
    echo "✓ Binary built successfully"
else
    echo "✓ Binary found at $BINARY"
fi
echo

# Test without database connection (dry run check)
echo "4. Testing binary can read and parse JSON files..."
echo "   (This will fail at DB connection if DB_PASSWORD not set)"
echo

if [ "$TEST_DB" = true ]; then
    # Full test with database
    echo "Running full upload to database..."
    "$BINARY" "$TEST_DIR/schedule_test.json" "$TEST_DIR/possible_periods_test.json"
    EXIT_CODE=$?
    
    if [ $EXIT_CODE -eq 0 ]; then
        echo
        echo "✓ Upload completed successfully!"
        echo
        echo "The test data has been uploaded to the database."
        echo "You can verify it using the Python script:"
        echo "  python3 -c \"from scripts.post_query import get_schedule; get_schedule(SCHEDULE_ID)\""
    else
        echo
        echo "❌ Upload failed with exit code $EXIT_CODE"
        exit $EXIT_CODE
    fi
else
    echo "⚠️  Skipping database upload (no DB_PASSWORD)"
    echo "   To test full upload, run:"
    echo "   DB_PASSWORD='your-password' $0"
fi

echo
echo "=== Test Summary ==="
echo "Test data location: $TEST_DIR"
echo "- schedule_test.json: 1 scheduling block"
echo "- possible_periods_test.json: 2 visibility periods"
echo
echo "To upload real data:"
echo "  DB_PASSWORD='password' ./scripts/upload_schedule.sh"
