#!/bin/bash
# Template Setup Script
# This script renames the project from "allmaptout" to your chosen name.
# Run: ./setup-template.sh myproject

set -e

if [ -z "$1" ]; then
    echo "Usage: ./setup-template.sh <project-name>"
    echo "Example: ./setup-template.sh mywebsite"
    exit 1
fi

NEW_NAME="$1"
NEW_NAME_SNAKE=$(echo "$NEW_NAME" | tr '-' '_')
NEW_NAME_KEBAB=$(echo "$NEW_NAME" | tr '_' '-')

OLD_NAME="allmaptout"
OLD_NAME_SNAKE="allmaptout"
OLD_NAME_KEBAB="allmaptout"

echo "Renaming project from '$OLD_NAME' to '$NEW_NAME'..."

# Files to update
FILES=(
    "backend/Cargo.toml"
    "backend/src/main.rs"
    "backend/src/bin/openapi.rs"
    "backend/src/schemas/mod.rs"
    "k8s/backend.yaml"
    "k8s/frontend.yaml"
    "k8s/ingress.yaml"
    ".github/workflows/ci.yml"
    "README.md"
    ".claude/CLAUDE.md"
    ".claude/PROJECT_STATUS.md"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        # Replace snake_case version (for Rust crate names)
        sed -i '' "s/${OLD_NAME_SNAKE}/${NEW_NAME_SNAKE}/g" "$file"
        # Replace kebab-case version (for package names, URLs)
        sed -i '' "s/${OLD_NAME_KEBAB}/${NEW_NAME_KEBAB}/g" "$file"
        echo "  Updated: $file"
    fi
done

# Update GitHub Container Registry paths (replace the username too if needed)
echo ""
echo "NOTE: You may also need to update:"
echo "  - GitHub container registry paths in k8s/*.yaml and .github/workflows/ci.yml"
echo "  - The 'min-andrew' GitHub username to your own"
echo "  - AWS EKS cluster name in .github/workflows/ci.yml"
echo "  - Kubernetes secrets (DATABASE_URL, JWT_SECRET, CORS_ORIGIN)"
echo ""

# Clean up build artifacts
echo "Cleaning build artifacts..."
rm -rf backend/target
rm -rf frontend/node_modules frontend/dist

echo ""
echo "Done! Project renamed to '$NEW_NAME'."
echo ""
echo "Next steps:"
echo "  1. Review the changes: git diff"
echo "  2. Update container registry paths if needed"
echo "  3. Delete example schemas in backend/src/schemas/mod.rs"
echo "  4. Delete this script: rm setup-template.sh"
echo "  5. Commit: git add -A && git commit -m 'chore: rename project to $NEW_NAME'"
