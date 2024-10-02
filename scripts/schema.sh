#!/bin/bash
# Main directory is the parent directory of the directory where this script is located
MAIN_DIR="$(dirname "$(dirname "$(realpath "$0")")")"

# Print out the main directory path for debugging
echo "Main directory: $MAIN_DIR"

# Define the contract directory
CONTRACTS_DIR="$MAIN_DIR/contracts"
TS_DIR="$MAIN_DIR/ts"

# Print out the contract directory path for debugging
echo "Contracts directory: $CONTRACTS_DIR"

# Navigate to the contracts directory and generate schema files
# Iterate over each contract directory
if [ -d "$CONTRACTS_DIR" ]; then
  cd "$CONTRACTS_DIR" || exit
  for CONTRACT_DIR in */; do
    echo "Generating schema files for contract: $CONTRACT_DIR"
    cd "$CONTRACT_DIR" || exit
    cargo schema
    cd ..
  done
  cd "$MAIN_DIR" || exit
else
  echo "Contracts directory not found!"
  exit 1
fi

# Print out the ts directory path for debugging
echo "TS directory: $TS_DIR"

# Navigate to the ts directory and generate TypeScript code
if [ -d "$TS_DIR" ]; then
  cd "$TS_DIR" || exit
  yarn generate-ts
  cd "$MAIN_DIR" || exit
else
  echo "TS directory not found!"
  exit 1
fi

# Clean schema files in the contracts directory
if [ -d "$CONTRACTS_DIR" ]; then
  cd "$CONTRACTS_DIR" || exit
  for CONTRACT_DIR in */; do
    echo "Cleaning schema files for contract: $CONTRACT_DIR"
    cd "$CONTRACT_DIR" || exit
    rm -rv schema
    cd ..
  done
  cd "$MAIN_DIR" || exit
else
  echo "Contracts directory not found!"
  exit 1
fi
