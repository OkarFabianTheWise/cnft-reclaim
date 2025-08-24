#!/bin/bash
set -e

PROGRAM_NAME="cnft_reclaim"
PROGRAM_KEYPAIR="./target/deploy/${PROGRAM_NAME}-keypair.json"
PROGRAM_SO="./target/deploy/${PROGRAM_NAME}.so"


# Function to verify build output
verify_build() {
  if [ ! -f "$PROGRAM_SO" ]; then
    echo "❌ Error: Built program not found at $PROGRAM_SO"
    echo "   Check if build completed successfully"
    exit 1
  fi
  echo "✅ Program file found: $PROGRAM_SO"
}

# Function to build the program
build_program() {
  echo "🛠  Building Solana program..."
  cargo build-sbf --manifest-path=Cargo.toml --sbf-out-dir=target/deploy
  verify_build
  echo "✅ Build complete: $PROGRAM_SO"
}

case $1 in
  build)
    build_program
    ;;
  
  deploy)
    echo "🚀 Building and deploying program..."
    build_program
    
    PROGRAM_ID=$(solana address -k "$PROGRAM_KEYPAIR")
    echo "📍 Program ID: $PROGRAM_ID"
    
  solana program deploy "$PROGRAM_SO" --keypair "$PROGRAM_KEYPAIR" --fee-payer ~/.config/solana/id.json
    ;;

  test)
    echo "🧪 Running build, deploy, and tests..."
    "$0" build
    "$0" deploy
    # Run all Rust unit and integration tests
    cargo test --all -- --nocapture
  ;;

  run-tests)
    echo "🧪 Running build, deploy, and tests..."
    # Run all Rust unit and integration tests
    cargo test -- --nocapture
  ;;
  
  test-validator)
    echo "⚡ Starting local validator with program preloaded..."
    build_program
    
    PROGRAM_ID=$(solana address -k "$PROGRAM_KEYPAIR")
    echo "📍 Program ID: $PROGRAM_ID"
    
  solana-test-validator --reset --bpf-program "$PROGRAM_ID" "$PROGRAM_SO" --fee-payer ~/.config/solana/id.json
    ;;
  
  clean)
    echo "🧹 Cleaning build artifacts..."
    cargo clean
    rm -rf target/deploy
    echo "✅ Clean complete"
    ;;
  
  debug)
    echo "🔍 Debug information:"
    echo "Program name: $PROGRAM_NAME"
    echo "Program keypair: $PROGRAM_KEYPAIR"
    echo "Program SO: $PROGRAM_SO"
    echo "Keypair exists: $([ -f "$PROGRAM_KEYPAIR" ] && echo "✅ Yes" || echo "❌ No")"
    echo "Program SO exists: $([ -f "$PROGRAM_SO" ] && echo "✅ Yes" || echo "❌ No")"
    if [ -f "$PROGRAM_KEYPAIR" ]; then
        echo "Program ID: $(solana address -k "$PROGRAM_KEYPAIR")"
    fi
    echo "Current directory: $(pwd)"
    echo "Target deploy directory contents:"
    ls -la target/deploy/ 2>/dev/null || echo "  Directory doesn't exist"
    ;;
  
  *)
    echo "Usage: ./devops.sh {build|deploy|test-validator|clean|debug}"
    echo ""
    echo "Commands:"
    echo "  build         - Build the Solana program"
    echo "  deploy        - Build and deploy to configured cluster" 
    echo "  test-validator - Start local validator with program preloaded"
    echo "  clean         - Clean build artifacts"
    echo "  debug         - Show debug information"
    echo "  test          - Build, deploy, and run all tests"
    echo "  run-tests     - Run all tests without building or deploying"
    ;;
esac