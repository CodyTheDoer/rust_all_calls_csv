use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use syn::{File, Item};
use csv::Writer;

fn main() {
    let spreadsheet_path = "./spreadsheets/project_references.csv";
    fs::create_dir_all("./spreadsheets").expect("Failed to create spreadsheets directory");

    let mut writer = Writer::from_path(spreadsheet_path)
        .expect("Failed to open CSV writer");
    writer.write_record(&["File", "Item Type", "Name"])
        .expect("Failed to write CSV header");

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
            println!("Processing .rs file: {}", path.display());

            if let Err(e) = process_file(path, &mut writer) {
                eprintln!("Error processing file {:?}: {:?}", path, e);
            }
        }
    }

    writer.flush().unwrap();
    println!("Done! CSV file generated at: {}", spreadsheet_path);
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
