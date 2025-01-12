use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::path::Path;

use csv::{Writer, ReaderBuilder};
use syn::{
    File as SynFile, 
    ImplItem,
    Item,
    TraitItem,
    Type,
};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Ensure the spreadsheets folder exists
    let spreadsheet_path = "./spreadsheets/project_references.csv";
    fs::create_dir_all("./spreadsheets")?;

    // 2. Read existing CSV (if it exists) into a HashSet<(file, item_type, name)>
    let mut existing_entries = HashSet::new();

    if Path::new(&spreadsheet_path).exists() {
        let file = File::open(&spreadsheet_path)?;
        let mut rdr = ReaderBuilder::new()
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
        println!("CSV exists. Read {} existing entries.", existing_entries.len());
    } else {
        println!("No existing CSV found; starting fresh.");
    }

    // 3. Build the CSV writer
    let mut writer = Writer::from_path(&spreadsheet_path)?;
    writer.write_record(&["File", "Item Type", "Name"])?;

    // 4. Gather new entries by scanning .rs files
    let mut new_entries = Vec::new();

    for entry in WalkDir::new(".")
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|x| {
            // Skip any entries in the `target` folder.
            !x.path().components().any(|comp| comp.as_os_str() == "target")
        })
    {
        let path = entry.path();
        if path.extension().and_then(|f| f.to_str()) == Some("rs") {
            println!("Processing .rs file: {}", path.display());
            
            match process_file(path) {
                Ok(file_items) => {
                    println!("  Found {} items in {}", file_items.len(), path.display());
                    new_entries.extend(file_items);
                }
                Err(e) => {
                    eprintln!("Error processing file {:?}: {:?}", path, e);
                }
            };
        }
    }

    // 5. Merge new entries into the existing set
    for entry in new_entries {
        existing_entries.insert(entry);
    }

    // 6. Write out the merged data (overwriting the old CSV)
    for (file, item_type, name) in &existing_entries {
        writer.write_record(&[file, item_type, name])?;
    }

    writer.flush()?;
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
            // --- top-level fn, enum, struct ---
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

            // --- impl SomeType { ... } ---
            Item::Impl(item_impl) => {
                // Extract the type name from `item_impl.self_ty` if possible
                // e.g. impl SomeType { fn foo() {} }
                let type_name = match &*item_impl.self_ty {
                    Type::Path(type_path) => {
                        // e.g. SomeType
                        type_path.path.segments.last()
                            .map(|seg| seg.ident.to_string())
                            .unwrap_or_else(|| "UnknownType".to_string())
                    }
                    _ => "UnknownType".to_string(),
                };

                for impl_item in item_impl.items {
                    match impl_item {
                        // This captures `fn` inside an impl block
                        ImplItem::Fn(method) => {
                            results.push((
                                path_str.clone(),
                                "ImplFn".to_string(),
                                // We can store "SomeType::method_name"
                                format!("{}::{}", type_name, method.sig.ident),
                            ));
                        }
                        _ => {}
                    }
                }
            }

            // --- trait SomeTrait { ... } ---
            Item::Trait(item_trait) => {
                let trait_name = item_trait.ident.to_string();
                for trait_item in item_trait.items {
                    match trait_item {
                        TraitItem::Fn(method) => {
                            results.push((
                                path_str.clone(), 
                                "TraitFn".to_string(),
                                // We can store "SomeTrait::method_name"
                                format!("{}::{}", trait_name, method.sig.ident),
                            ));
                        }
                        _ => {}
                    }
                }
            }

            // Everything else is ignored for now
            _ => {}
        }
    }

    Ok(results)
}
