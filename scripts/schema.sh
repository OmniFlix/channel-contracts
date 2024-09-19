#!/bin/bash

# Define the main directory path (one level up from the scripts directory)
MAIN_DIR=$(dirname "$(realpath "$0")")/..

# Print out the main directory path for debugging
echo "Main directory: $MAIN_DIR"

# Define the contract directory
CONTRACT_DIR="$MAIN_DIR"
TS_DIR="$MAIN_DIR/ts"

# Print out the contract directory path for debugging
echo "Contract directory: $CONTRACT_DIR"

# Navigate to the contract directory and generate schema
if [ -d "$CONTRACT_DIR" ]; then
  echo "Processing directory: $CONTRACT_DIR"
  cd "$CONTRACT_DIR" || exit
  cargo schema
  cd "$MAIN_DIR" || exit
else
  echo "Contract directory not found!"
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

# Clean schema files in the contract directory
if [ -d "$CONTRACT_DIR/schema" ]; then
  echo "Cleaning schema files in directory: $CONTRACT_DIR"
  rm -rf "$CONTRACT_DIR/schema"
else
  echo "No schema directory found to clean."
fi
