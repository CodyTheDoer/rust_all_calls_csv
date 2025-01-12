use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use syn::{File, Item};
use csv::Writer;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let spreadsheet_dir = format!("{}/spreadsheets", out_dir);
    let spreadsheet_path = format!("{}/project_references.csv", spreadsheet_dir);

    // Ensure the directory exists
    fs::create_dir_all(&spreadsheet_dir).unwrap();

    // Open CSV writer
    let mut writer = Writer::from_path(&spreadsheet_path).unwrap();
    writer.write_record(&["File", "Item Type", "Name"]).unwrap();

    // Traverse and process `.rs` files
    for entry in WalkDir::new(".").into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            process_file(path, &mut writer).unwrap();
        }
    }

    writer.flush().unwrap();
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build.rs");
}

fn process_file(path: &Path, writer: &mut Writer<std::fs::File>) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let syntax: File = syn::parse_file(&content)?;

    for item in syntax.items {
        match item {
            Item::Fn(func) => {
                writer.write_record(&[path.to_string_lossy(), "Function", func.sig.ident.to_string()])?;
            }
            Item::Enum(en) => {
                writer.write_record(&[path.to_string_lossy(), "Enum", en.ident.to_string()])?;
            }
            Item::Struct(st) => {
                writer.write_record(&[path.to_string_lossy(), "Struct", st.ident.to_string()])?;
            }
            _ => {}
        }
    }

    Ok(())
}
