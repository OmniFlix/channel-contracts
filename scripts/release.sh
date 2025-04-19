#!/bin/bash

# Check if version argument is provided
if [ -z "$1" ]; then
    echo "Usage: ./release.sh <version> [rc]"
    echo "Example: ./release.sh 1.0.0 rc"
    exit 1
fi

VERSION=$1
RC=$2

# Ensure we're on develop branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "develop" ]; then
    echo "Error: Must be on develop branch"
    exit 1
fi

# Pull latest changes
git pull origin develop

# Create release branch
RELEASE_BRANCH="release/v$VERSION"
git checkout -b $RELEASE_BRANCH

# Update version in Cargo.toml
cargo set-version $VERSION

# Create RC tag if specified
if [ "$RC" = "rc" ]; then
    git commit -am "Bump version to v$VERSION-rc1"
    git tag -a v$VERSION-rc1 -m "Release candidate 1 for v$VERSION"
    git push origin v$VERSION-rc1
else
    git commit -am "Bump version to v$VERSION"
    git tag -a v$VERSION -m "Release v$VERSION"
    git push origin v$VERSION
fi

# Push release branch
git push origin $RELEASE_BRANCH

echo "Release branch $RELEASE_BRANCH created and pushed"
echo "Next steps:"
echo "1. Wait for CI/CD pipeline to complete"
echo "2. Test the release on testnet"
echo "3. If everything is good, merge to main:"
echo "   git checkout main"
echo "   git merge $RELEASE_BRANCH"
echo "   git push origin main"
echo "4. Merge back to develop:"
echo "   git checkout develop"
echo "   git merge $RELEASE_BRANCH"
echo "   git push origin develop"
echo "5. Clean up:"
echo "   git branch -d $RELEASE_BRANCH"
echo "   git push origin --delete $RELEASE_BRANCH" 