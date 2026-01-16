export STARS_CORE_DIR=/workspace/install
export STARS_ROOT=/workspace/install
export CMAKE_PREFIX_PATH=/workspace/install
export LD_LIBRARY_PATH=/workspace/install/lib:$LD_LIBRARY_PATH
# Needed for rustc to find the shared libraries during linking
export RUSTFLAGS="-L native=/workspace/install/lib"

echo "Environment configured for STARS Core (local install in /workspace/install)"
echo "You can now run: cargo check --features=build-native"
