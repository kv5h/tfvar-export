# tfvar-export

Set Terraform variables from output value across Projects and Workspaces.

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
