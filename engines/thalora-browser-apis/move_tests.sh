#!/bin/bash

# Script to move all test files to tests/ directory

echo "Moving tests to tests/ directory..."

# Create directory structure
mkdir -p tests/{dom,fetch,storage,worker,file,events,browser,crypto,console}

# Move test files from subdirectories (keeping directory structure)
find src -name "tests.rs" -o -name "integration_tests.rs" -o -name "debug_test.rs" | while read testfile; do
    # Get the relative path from src/
    rel_path="${testfile#src/}"
    # Get the directory part
    dir_part=$(dirname "$rel_path")
    # Get the filename
    file_part=$(basename "$rel_path")

    # Create target directory
    mkdir -p "tests/$dir_part"

    # Move the file
    echo "Moving $testfile -> tests/$dir_part/$file_part"
    mv "$testfile" "tests/$dir_part/$file_part"
done

# Remove empty subdirectories left behind
find src -type d -empty -delete

echo "Test file moving complete!"
