#!/bin/bash

# Test script to demonstrate PHLOW_MAIN_FILE functionality
# This shows how to use a specific file from a Git repository

echo "=== Testing PHLOW_MAIN_FILE functionality ==="
echo

# Example 1: Without PHLOW_MAIN_FILE (uses default main.phlow)
echo "1. Testing without PHLOW_MAIN_FILE (should look for main.phlow):"
echo "   Command: phlow git@github.com:cogup/studio-server.git"
echo "   Expected: Will search for main.phlow, main.yaml, or main.yml in root"
echo

# Example 2: With PHLOW_MAIN_FILE (uses specific file)
echo "2. Testing with PHLOW_MAIN_FILE (should use specific file):"
echo "   Command: PHLOW_MAIN_FILE='phlows/webhook/webhook.phlow' phlow git@github.com:cogup/studio-server.git"
echo "   Expected: Will use the specified file path"
echo

# Example 3: With PHLOW_MAIN_FILE and branch
echo "3. Testing with PHLOW_MAIN_FILE and branch:"
echo "   Command: PHLOW_MAIN_FILE='phlows/webhook/webhook.phlow' phlow git@github.com:cogup/studio-server.git#develop"
echo "   Expected: Will clone develop branch and use the specified file"
echo

# Example 4: Error case - file not found
echo "4. Testing error case (file not found):"
echo "   Command: PHLOW_MAIN_FILE='non/existent/file.phlow' phlow git@github.com:cogup/studio-server.git"
echo "   Expected: Should show error message about file not found"
echo

echo "=== How to use ==="
echo "1. Set PHLOW_MAIN_FILE environment variable to specify the exact file path"
echo "2. Run phlow with the Git repository URL"
echo "3. The system will clone the repo and use your specified file instead of searching for defaults"
echo
echo "Examples:"
echo "  export PHLOW_MAIN_FILE='phlows/webhook/webhook.phlow'"
echo "  phlow git@github.com:cogup/studio-server.git"
echo
echo "  # Or in one line:"
echo "  PHLOW_MAIN_FILE='phlows/webhook/webhook.phlow' phlow git@github.com:cogup/studio-server.git"
