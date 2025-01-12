use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use syn::{File, Item};
use csv::Writer;

fn main() {
    println!("all_calls_csv CLI -> generating CSV of calls in local .rs files");
    
    let spreadsheet_dir = "./spreadsheets";
    let spreadsheet_path = format!("{}/project_references.csv", spreadsheet_dir);

    // Ensure the directory exists
    fs::create_dir_all(&spreadsheet_dir).expect("Failed to create spreadsheets directory");
    
    // Open CSV writer
    let mut writer = Writer::from_path(&spreadsheet_path)
        .expect("Failed to open CSV writer");
    writer.write_record(&["File", "Item Type", "Name"])
        .expect("Failed to write CSV header");

    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            // Exclude `target` and possibly other directories
            !e.path().starts_with("./target")
        })
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Err(e) = process_file(path, &mut writer) {
                eprintln!("Error processing file {:?}: {:?}", path, e);
            }
        }
    }
    
    writer.flush().expect("Failed to flush CSV writer");
    println!("Done! CSV file generated at: {}", spreadsheet_path);
    
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build.rs");
}

fn process_file(path: &Path, writer: &mut Writer<std::fs::File>) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let syntax: File = syn::parse_file(&content)?;

    for item in syntax.items {
        match item {
            Item::Fn(func) => {
                writer.write_record(&[
                    path.to_string_lossy().to_string(),
                    "Function".to_string(),
                    func.sig.ident.to_string(),
                ])?;
            }
            Item::Enum(en) => {
                writer.write_record(&[
                    path.to_string_lossy().to_string(),
                    "Enum".to_string(),
                    en.ident.to_string(),
                ])?;
            }
            Item::Struct(st) => {
                writer.write_record(&[
                    path.to_string_lossy().to_string(),
                    "Struct".to_string(),
                    st.ident.to_string(),
                ])?;
            }
            _ => {}
        }
    }

    Ok(())
}