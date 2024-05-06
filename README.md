# tfvar-export

Set Terraform Cloud variables across Projects and Workspaces from outputs of
current workspace.

## Motivation

When you need to share Terraform Output values across workspaces,
`terraform_remote_state` data source is the primary way, but this feature can
create dependencies between workspaces and then make Terraform less manageable.
However, manually registering Output values from other workspaces is not only
labor-intensive but also carries the risk of operational errors. This tool
allows semi-automatic registration of variables without creating dependencies
between workspaces.

## Remarks

- Only variable value is covered, hence description will NOT be updated or
  described.
- Outputs with type of `string`, `number` or `bool` are created/updated as
  `hcl = false` and the others as `hcl = true`.
  - Reference:
    [`Types and Values`](https://developer.hashicorp.com/terraform/language/expressions/types)
- All variables are registered...
  - as `Non sensitive`, so please **be careful not to specify sensitive output
    values**.
  - as category of `terraform` (NOT `environment variables`).

## Prerequisite

1. Export environment variables
   1. Terraform Cloud organization name as `TFVE_ORGANIZATION_NAME`
   1. Terraform Cloud token as `TFVE_TOKEN`
1. Specify outputs to be exported to other workspaces. See
   [Below](#define-export-list) for details.

### Define export list

Export list defines variables to be exported to other workspaces.

The list is specified in `Key-Value` format separated by a comma, where `Key`
indicates the name of the output variable of the current Workspace and `Value`
indicates the name of the variable at the destination.

Note that only the variables listed in this list are exported.

**Example:**

If you need to export the value of output such as:

```terraform
output "my_output" {
  description = "my output"
  value       = "some value"
}
```

define the export list as follows.

```
my_output,valiable_name_xyz

# This line will be ignored as a comment.
```

Then the value of `my_output` is created or updated as `valiable_name_xyz` at
the targeted workspace(s).

**NOTE:**

- Updating is allowed by using the `--allow-update` flag.
- To show outputs of current workspace, use `--show-outputs` flag.
- You can use `#` to comment out a whole line.

## Usage

TODO:

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

TODO: Add examples for each flag.

### Output examples

#### --show-outputs

```json
[
  {
    "terraform_workspace_id": "ws-xxxxxxxxxxxxxxxx",
    "terraform_workspace_name": "ws-x",
    "terraform_project": {
      "terraform_project_id": "prj-xxxxxxxxxxxxxxxx",
      "terraform_project_name": "pj-x"
    }
  },
  {
    "terraform_workspace_id": "ws-yyyyyyyyyyyyyyyy",
    "terraform_workspace_name": "ws-y",
    "terraform_project": {
      "terraform_project_id": "prj-yyyyyyyyyyyyyyyy",
      "terraform_project_name": "pj-y"
    }
  },
  {
    "terraform_workspace_id": "ws-zzzzzzzzzzzzzzzz",
    "terraform_workspace_name": "ws-z",
    "terraform_project": {
      "terraform_project_id": "prj-zzzzzzzzzzzzzzzz",
      "terraform_project_name": "pj-z"
    }
  }
]
```
