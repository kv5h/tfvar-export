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
      1. Required if `--show-workspaces` is specified.
   2. Terraform Cloud token as `TFVE_TOKEN`
2. Specify outputs to be exported to other workspaces. See
   [Below](#define-export-list) for details.

### Define export list

Export list defines variables to be exported to other workspaces.

The list is specified in the format as below:

```text
<Output name>,<Variable name>,<Variable description>
```

where `<Output name>` is the name of the output in the output file and
`<Variable name>` and `<Variable description>` are the name and its description
of the variable at the destination.

Note that only the variables listed in this list are exported.

**Example:**

If you need to export the value of output such as:

```terraform
output "my_output" {
  description = "my output"
  value       = "some value"
}

output "my_output_2" {
  description = "my output 2"
  value       = "some value 2"
}
```

define the export list as follows.

```text
# This line is ignored as a comment
my_output,my_var,this_is_description

# Description is optional
my_output_2,my_var_2
```

Then the value of `my_output` is created or updated as `my_var` with the
description `this_is_description` at the targeted workspace(s).

As well, the value of `my_output_2` is created or updated as `my_var_2` without
description.

**REMARK:**

- Updating is allowed by using the `--allow-update` flag.
- You can use `#` to comment out a whole line.

## Usage

```text
Usage: tfvar-export [OPTIONS] [PATH_TO_OUTPUT_VALUES_FILE] [PATH_TO_EXPORT_LIST]

Arguments:
  [PATH_TO_OUTPUT_VALUES_FILE]  Path to the output values file generated with `terraform output --json`.
                                Required unless `--show-workspaces` is set.
  [PATH_TO_EXPORT_LIST]         Path to the export list.
                                Required unless `--show-workspaces` is set.

Options:
  -b, --base-url <BASE_URL>
          Base URL of Terraform API [default: https://app.terraform.io]
  -t, --target-workspaces <WORKSPACE_NAME1,WORKSPACE_NAME2,...>
          Comma separated Terraform Cloud workspace names.
          Required unless `--show-workspaces` is set.
  -l, --info-log
          Enable `Info` log.
          Note that `Error` log is always enabled regardless of this flag.
  -w, --show-workspaces
          Show available workspaces.
  -u, --allow-update
          Allow update of existing values.
  -h, --help
          Print help
  -V, --version
          Print version
```

### Output examples

#### `--show-workspaces`

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

## Testing

- Set the `TFVE_WORKSPACE_ID_TESTING` and `TFVE_WORKSPACE_ID_TESTING2`
  environment variable.
  - Testing dedicated workspaces should be used for safety.
