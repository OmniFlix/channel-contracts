#!/bin/bash

# Function to print usage
usage() {
    echo "Usage: ./cleanup.sh [options]"
    echo "Options:"
    echo "  -a, --all        Clean up all merged branches"
    echo "  -r, --release    Clean up merged release branches only"
    echo "  -f, --feature    Clean up merged feature branches only"
    echo "  -h, --help       Show this help message"
    exit 1
}

# Default values
CLEAN_ALL=false
CLEAN_RELEASE=false
CLEAN_FEATURE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -a|--all)
            CLEAN_ALL=true
            shift
            ;;
        -r|--release)
            CLEAN_RELEASE=true
            shift
            ;;
        -f|--feature)
            CLEAN_FEATURE=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

# If no options specified, show usage
if [ "$CLEAN_ALL" = false ] && [ "$CLEAN_RELEASE" = false ] && [ "$CLEAN_FEATURE" = false ]; then
    usage
fi

# Ensure we're on develop branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "develop" ]; then
    echo "Error: Must be on develop branch"
    exit 1
fi

# Pull latest changes
git pull origin develop

# Clean up branches
if [ "$CLEAN_ALL" = true ] || [ "$CLEAN_RELEASE" = true ]; then
    echo "Cleaning up release branches..."
    git branch -r | grep 'origin/release/' | while read branch; do
        git branch -d ${branch#origin/} 2>/dev/null || true
    done
    git push origin --delete $(git branch -r | grep 'origin/release/')
fi

if [ "$CLEAN_ALL" = true ] || [ "$CLEAN_FEATURE" = true ]; then
    echo "Cleaning up feature branches..."
    git branch --merged develop | grep -v 'develop' | grep -v 'main' | while read branch; do
        git branch -d $branch
    done
fi

# Clean up remote branches that no longer exist locally
echo "Cleaning up remote tracking branches..."
git remote prune origin

echo "Cleanup complete!" 