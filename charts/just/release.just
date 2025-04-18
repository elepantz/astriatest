set allow-duplicate-recipes

import 'defaults.just'

# Set the working directory for all recipes to the project's root.
set working-directory := "../../"

_default_release:
  @just --list release

# Check for required dependencies
_check_dependencies:
  #!/usr/bin/env bash
  set -eu
  if ! command -v yq >/dev/null 2>&1; then
    echo "Error: yq is not installed. Please install it first:"
    echo "  brew install yq  # on macOS"
    echo "  or visit https://github.com/mikefarah/yq#install for other platforms"
    exit 1
  fi

# Default values for release management
defaultVersion := "0.1.0"
defaultComponent := "charts"
defaultVersionType := "chart"

# Default release message
defaultReleaseMessage := "Update component versions"

# Base directory for charts
CHARTS_DIR := "charts"
VALUES_FILE := CHARTS_DIR + "/values.yaml"

########################
## Version Management ##
########################

# Validate Version
###################
[doc("
Validates if a version string follows semantic versioning format.
Usage:
  just release validate-version <VERSION>
")]
validate-version version:
  @if ! echo "{{version}}" | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' > /dev/null; then \
    echo "Error: Invalid version format. Must be semantic version (e.g., 1.0.0)"; \
    exit 1; \
  fi
  @echo "Version {{version}} is valid"

# Update Version
################
[doc("
Updates the version for a component in values.yaml.
Usage:
  just release update-version <COMPONENT> <VERSION_TYPE> <NEW_VERSION>
  (defaults: 'charts', 'chart', '0.1.0')
")]
update-version component=defaultComponent version_type=defaultVersionType new_version=defaultVersion: _check_dependencies
  @just release validate-version {{new_version}}
  @yq e ".global.versions.{{component}}.{{version_type}} = \"{{new_version}}\"" -i {{VALUES_FILE}}
  @echo "Updated {{component}} {{version_type}} version to {{new_version}}"

# Update Image Tag
################
[doc("
Updates the image tag for a component in values.yaml.
Usage:
  just release update-image <COMPONENT> <NEW_TAG>
  (defaults: 'charts', 'latest')
")]
update-image component=defaultComponent new_tag="latest": _check_dependencies
  @yq e ".global.versions.{{component}}.image.tag = \"{{new_tag}}\"" -i {{VALUES_FILE}}
  @echo "Updated {{component}} image tag to {{new_tag}}"

# Get Version
#############
[doc("
Gets the current version for a component from values.yaml.
Usage:
  just release get-version <COMPONENT> <VERSION_TYPE>
  (defaults: 'charts', 'chart')
")]
get-version component=defaultComponent version_type=defaultVersionType: _check_dependencies
  @yq e ".global.versions.{{component}}.{{version_type}}" {{VALUES_FILE}}

# Validate All Versions
######################
[doc("
Validates all versions in values.yaml.
Usage:
  just release validate-all
")]
validate-all: _check_dependencies
  @for component in $(yq e '.global.versions | keys | .[]' {{VALUES_FILE}}); do \
    chart_version=$(yq e ".global.versions.$$component.chart" {{VALUES_FILE}}); \
    app_version=$(yq e ".global.versions.$$component.app" {{VALUES_FILE}}); \
    echo "✓ $$component: chart=$$chart_version, app=$$app_version"; \
  done

# Show Version Diff
##################
[doc("
Shows version differences between environments.
Usage:
  just release version-diff <ENV1> <ENV2>
  (defaults: 'dev', 'mainnet')
")]
version-diff env1="dev" env2="mainnet":
  @echo "Comparing versions between {{env1}} and {{env2}}:"
  # This is a placeholder - implement actual version comparison logic
  # when environment-specific versions are implemented

# Bump Version
#############
[doc("
Bumps the version for a component.
Usage:
  just release bump-version <COMPONENT> <VERSION_TYPE> <BUMP_TYPE>
  (defaults: 'charts', 'chart', 'patch')
  BUMP_TYPE can be: major, minor, patch
")]
bump-version component=defaultComponent version_type=defaultVersionType bump_type="patch": _check_dependencies
  #!/usr/bin/env bash
  set -eu
  current_version=$(yq e ".global.versions.{{component}}.{{version_type}}" {{VALUES_FILE}})
  IFS='.' read -r major minor patch <<< "$current_version"
  if [ "{{bump_type}}" = "major" ]; then
    new_version="$((major + 1)).0.0"
  elif [ "{{bump_type}}" = "minor" ]; then
    new_version="$major.$((minor + 1)).0"
  elif [ "{{bump_type}}" = "patch" ]; then
    new_version="$major.$minor.$((patch + 1))"
  else
    echo "Error: Invalid bump type. Use major, minor, or patch"
    exit 1
  fi
  yq e ".global.versions.{{component}}.{{version_type}} = \"$new_version\"" -i {{VALUES_FILE}}
  echo "Bumped {{component}} {{version_type}} version from $current_version to $new_version"

# Create Release Commit
#####################
[doc("
Creates a release commit for Release Please.
Usage:
  just release create-release <COMPONENT> <VERSION> [MESSAGE]
  (defaults: 'charts', '0.1.0', 'Update component versions')
")]
create-release component=defaultComponent version=defaultVersion message=defaultReleaseMessage:
  @git commit --allow-empty -m "chore({{component}}): release {{version}}" -m "{{message}}" -m "Release-As: {{version}}"
  @echo "Created release commit for {{component}} version {{version}}"

# Create Release
###############
[doc("
Creates a new release by updating versions and creating a release commit.
Usage:
  just release create <COMPONENT> <VERSION> [MESSAGE]
  (defaults: 'charts', '0.1.0', 'Update component versions')
")]
create component=defaultComponent version=defaultVersion message=defaultReleaseMessage:
  @just release update-version {{component}} chart {{version}}
  @just release update-version {{component}} app {{version}}
  @just release create-release {{component}} {{version}} "{{message}}"

# Create Release for All Components
##################################
[doc("
Creates a new release for all components with the same version.
Usage:
  just release create-all <VERSION> [MESSAGE]
  (defaults: '0.1.0', 'Update all component versions')
")]
create-all version=defaultVersion message="Update all component versions": _check_dependencies
  @for component in $(yq e '.global.versions | keys | .[]' {{VALUES_FILE}}); do \
    just release update-version $$component chart {{version}}; \
    just release update-version $$component app {{version}}; \
  done
  @just release create-release charts {{version}} "{{message}}"

# Bump All Versions
##################
[doc("
Bumps all component versions by the specified amount.
Usage:
  just release bump-all <BUMP_TYPE>
  (default: 'patch')
  BUMP_TYPE can be: major, minor, patch
")]
bump-all bump_type="patch": _check_dependencies
  #!/usr/bin/env bash
  set -eu
  components=$(yq e '.global.versions | keys | .[]' {{VALUES_FILE}})
  for component in $components; do
    just release bump-version "$component" chart {{bump_type}}
    just release bump-version "$component" app {{bump_type}}
  done
  echo "Bumped all component versions by {{bump_type}}"

# Create Changelog
#################
[doc("
Creates or updates the CHANGELOG.md file.
Usage:
  just release changelog <VERSION> [MESSAGE]
  (defaults: '0.1.0', 'Initial release')
")]
changelog version=defaultVersion message="Initial release":
  @if [ ! -f {{CHARTS_DIR}}/CHANGELOG.md ]; then \
    echo "# Changelog\n\n## [{{version}}] - $(date +%Y-%m-%d)\n\n### Features\n* {{message}}\n\n### Components" > {{CHARTS_DIR}}/CHANGELOG.md; \
    for component in $(yq e '.global.versions | keys | .[]' {{VALUES_FILE}}); do \
      echo "* $$component" >> {{CHARTS_DIR}}/CHANGELOG.md; \
    done; \
  else \
    echo "Changelog already exists. Please update it manually."; \
  fi