# Agedashi Development Scripts

This directory contains utility scripts for developing and maintaining Agedashi.

## Icon Coverage Tools

### `list-aws-resources.sh`

Extracts all AWS resource types from the OpenTofu/Terraform AWS provider and saves them to `test/aws-provider-resources.txt`.

**Requirements:**
- OpenTofu (`tofu`) or Terraform (`terraform`) installed

**Usage:**
```bash
./scripts/list-aws-resources.sh
```

**Output:**
- Displays all AWS resources available in the provider
- Saves list to `test/aws-provider-resources.txt`
- This file is used by the icon coverage test

### Icon Coverage Test

After running the script above, you can check which AWS resources don't have icons:

```bash
cargo test test_document_missing_icon_coverage -- --nocapture
```

**Output:**
```
AWS Provider Icon Coverage Report
═══════════════════════════════════════════════════
Total AWS resources in provider: 792
Resources with icon mappings: 265
Resources without icons: 527
Coverage: 33.5%
```

The test also shows:
- Which icon maps to which resources
- Top services without icon coverage (grouped by service prefix)
- Detailed breakdown for prioritizing which icons to add

**Note:** This test always passes - it's informational only. We don't require 100% coverage since many AWS resources are rarely used in infrastructure diagrams.

## Updating the Resource List

The AWS provider adds new resources over time. To update the list:

1. Run the extraction script:
   ```bash
   ./scripts/list-aws-resources.sh
   ```

2. Commit the updated file:
   ```bash
   git add test/aws-provider-resources.txt
   git commit -m "chore: update AWS provider resources list"
   ```

3. Check coverage to see if new high-priority resources were added:
   ```bash
   cargo test test_document_missing_icon_coverage -- --nocapture
   ```

## Adding New Icon Mappings

When you want to add support for a new AWS service:

1. Check if AWS Architecture Icons includes an icon for the service:
   - Icons are in `icons/Asset-Package.7z`
   - Or check: https://aws.amazon.com/architecture/icons/

2. If an icon exists, add to `src/main.rs`:
   - Add search pattern to `get_icon_search_pattern()`
   - Add service mapping to `get_service_info()`
   - Add icon name to extraction lists (search for `let icon_names = vec![`)

3. Run tests to verify:
   ```bash
   cargo test test_all_aws_resource_icons
   ```

4. Check updated coverage:
   ```bash
   cargo test test_document_missing_icon_coverage -- --nocapture
   ```
