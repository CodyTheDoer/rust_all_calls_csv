use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::path::Path;

use csv::{Writer, ReaderBuilder};
use syn::{File as SynFile, Item};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spreadsheet_path = "./spreadsheets/project_references.csv";
    fs::create_dir_all("./spreadsheets")
        .expect("Failed to create spreadsheets directory");

    // 2. Read existing CSV (if it exists) into a HashSet<(file, item_type, name)>
    let mut existing_entries = HashSet::new();

    if Path::new(&spreadsheet_path).exists() {
        let file = File::open(&spreadsheet_path)
            .expect("Failed to open project_references.csv");
        let mut rdr = ReaderBuilder::new()
            // If you have a header row, you can set .has_headers(true), 
            // but let's assume we manually handle the header
            .has_headers(true)
            .from_reader(file);

        for result in rdr.records() {
            let record = result?;
            // record[0] = File, record[1] = Item Type, record[2] = Name
            existing_entries.insert((
                record[0].to_string(),
                record[1].to_string(),
                record[2].to_string(),
            ));
        }
    }

    // 3. Build the writer
    let mut writer = Writer::from_path(&spreadsheet_path)
        .expect("Failed to open CSV writer");
    writer.write_record(&["File", "Item Type", "Name"])
        .expect("Failed to write CSV header");

    // 4. Gather new entries by scanning .rs files
    let mut new_entries = Vec::new();

    for entry in WalkDir::new(".")
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|x| {
            // Skip any entries in the `target` folder.
            !x.path().components().any(|x| x.as_os_str() == "target")
        })
    {
        let path = entry.path();

        if path.extension().and_then(|f| f.to_str()) == Some("rs") {
            match process_file(path) {
                Ok(file_items) => {
                    // file_items is a Vec<(String, String, String)>
                    new_entries.extend(file_items);
                }
                Err(e) => {
                    eprintln!("Error processing file {:?}: {:?}", path, e);
                }
            }
        }
    }
    
    // 5. Merge new entries into the existing set
    for entry in new_entries {
        existing_entries.insert(entry);
    }

    // 6. Write out the merged data (overwriting the old CSV)
    writer.flush().unwrap();
    println!("Done! Updated/Merged CSV file at: {}", spreadsheet_path);
    
    Ok(())
}

/// Returns a vector of (file, item_type, name) for all items in the given .rs file.
fn process_file(path: &Path) -> Result<Vec<(String, String, String)>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let syntax: SynFile = syn::parse_file(&content)?;

    let mut results = Vec::new();
    let path_str = path.to_string_lossy().to_string();

    for item in syntax.items {
        match item {
            Item::Fn(func) => {
                results.push((
                    path_str.clone(), 
                    "Function".to_string(), 
                    func.sig.ident.to_string()
                ));
            }
            Item::Enum(en) => {
                results.push((
                    path_str.clone(), 
                    "Enum".to_string(), 
                    en.ident.to_string()
                ));
            }
            Item::Struct(st) => {
                results.push((
                    path_str.clone(), 
                    "Struct".to_string(), 
                    st.ident.to_string()
                ));
            }
            _ => {}
        }
    }

    Ok(results)
}