# tfvar-export

Set Terraform Cloud variables across Projects and Workspaces from output of current workspace.

## Prerequisite

1. Export environment variables
   1. Terraform Cloud organization name as `TFVE_ORGANIZATION_NAME`
   1. Terraform Cloud token as `TFVE_TOKEN`

## Usage

```bash
Usage: tfvar-export [OPTIONS]

Options:
  -u, --base-url <base_url>  Base URL of Terraform API [default: https://app.terraform.io/api/v2]
  -h, --help                 Print help
```
