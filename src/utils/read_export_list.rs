//! Read output values file and return outputs.

use std::{
    collections::HashMap,
    io::{prelude::*, BufReader},
};

/// Read export list and return a HashMap.
///
/// ## Remark
///
/// Return a HashMap for searching efficiency.
pub fn read_export_list(
    file_path: &str,
) -> Result<Option<HashMap<String, (String, Option<String>)>>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(file_path).expect("Failed to open a file.");
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    contents = contents.trim().to_string(); // Trim leading and trailing empty lines

    let mut output: HashMap<String, (String, Option<String>)> = HashMap::new();
    let mut lines = contents.lines();

    let mut entries = Vec::new();
    while let Some(mut line) = lines.next() {
        line = line.trim();
        if line.is_empty() || line.starts_with("#") {
            // Skip an empty or a comment line
            continue;
        }
        entries.push(line)
    }

    if entries.len() < 1 {
        log::warn!("No valid entries were found in the export list.");
        return Ok(None);
    }

    entries.into_iter().for_each(|entry| {
        let record: Vec<String> = entry.split(',').map(|val| val.to_string()).collect();
        let source = record.get(0).expect("Failed to read entry.").to_owned();
        let dest = record.get(1).expect("Failed to read entry.").to_owned();
        let description = match record.get(2) {
            Some(val) => Some(val.to_owned()),
            None => None,
        };
        output.insert(source, (dest, description));
    });

    Ok(Some(output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_export_list_succeed() {
        // Neat entries
        let path = "files/test/export_list.txt";
        let resp = read_export_list(&path).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            "number_float".to_string(),
            (
                "number_float_copy".to_string(),
                Some("number_float_description".to_string()),
            ),
        );
        expected.insert(
            "set_of_object".to_string(),
            ("set_of_object_copy".to_string(), None),
        );
        assert_eq!(resp.unwrap(), expected);

        // With empty lines
        let path = "files/test/export_list.with_empty_lines.txt";
        let resp = read_export_list(&path).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            "number_float".to_string(),
            ("number_float_copy".to_string(), Some("".to_string())),
        );
        expected.insert(
            "set_of_object".to_string(),
            (
                "set_of_object_copy".to_string(),
                Some("set_of_object_description".to_string()),
            ),
        );
        assert_eq!(resp.unwrap(), expected);
    }

    #[test]
    fn test_read_export_list_fail() {
        let path = "files/test/export_list.no_line.txt";
        let resp = read_export_list(&path).unwrap();
        assert_eq!(resp, None);
    }
}
