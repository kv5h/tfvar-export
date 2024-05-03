# tfvar-export

Set Terraform Cloud variables across Projects and Workspaces from outputs of
current workspace.

## Use cases

When you need to share Terraform Output values across workspaces,
`terraform_remote_state` data source is the primary way, but this feature can
create dependencies between workspaces and then make Terraform less manageable.
However, manually registering Output values from other workspaces is not only
labor-intensive but also carries the risk of operational errors. This tool
allows semi-automatic registration of variables without creating dependencies
between workspaces.

## Prerequisite

1. Export environment variables
   1. Terraform Cloud organization name as `TFVE_ORGANIZATION_NAME`
   1. Terraform Cloud token as `TFVE_TOKEN`
1. Define variables to be exported to other workspaces. See
   [Below](#define-export-list) for details.

### Define export list

Export list defines variables to be exported to other workspaces.

The list is specified in `Key-Value` format separated by a comma, where `Key`
indicates the name of the output variable of the current Workspace and `Value`
indicates the name of the variable at the destination.

Note that only the variables listed in this list are exported, and the variable
names can be the same value.

**Example:**

If you need to export the value of output such as below as
`dynamodb_table_attribute`,

```terraform
output "aws_dynamodb_table_attribute" {
  description = "description"
  value       = aws_dynamodb_table.rss.attribute
}
```

define the export list  as follows:

```
aws_dynamodb_table_attribute,dynamodb_table_attribute
```

Then the value of `aws_dynamodb_table_attribute` is created or updated as
`dynamodb_table_attribute` at the targeted workspace(s).

**NOTE:**

- Updating is allowed with `--allow-update (-u)` flag.
- To list outputs of current workspace, use `--show-outputs (-s)` flag.

## Usage

```
Usage: tfvar-export [OPTIONS] <PATH_TO_TFSTATE_FILE> <PATH_TO_EXPORT_LIST>

Arguments:
  <PATH_TO_TFSTATE_FILE>  Path to tfstate file
  <PATH_TO_EXPORT_LIST>   Path to export list

Options:
  -b, --base-url <BASE_URL>  Base URL of Terraform API [default: https://app.terraform.io/api/v2]
  -s, --show-outputs         Show list of Output and exit
  -u, --allow-update         Allow update of the value when a variable already exists
  -h, --help                 Print help
  -V, --version              Print version
```
