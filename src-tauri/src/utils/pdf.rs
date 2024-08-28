use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

pub fn read(filepath: &str) -> Result<Vec<u8>, String> {
    let mut file = match File::open(filepath) {
        Ok(x) => x,
        Err(err) => return Err(format!("Failed to open ({})", err)),
    };

    match validate_pdf(&mut file) {
        Ok(_) => {}
        Err(err) => return Err(err),
    };

    let mut buffer = Vec::new();
    match file.read_to_end(&mut buffer) {
        Ok(_) => {}
        Err(err) => return Err(format!("Failed to read ({})", err)),
    };
    Ok(buffer)
}

fn validate_pdf(file: &mut File) -> Result<(), String> {
    const PDF_LEADING_HEADER: &[u8; 5] = b"%PDF-";
    let mut checker = [0; 5];
    match file.read_exact(&mut checker) {
        Ok(_) => {
            if &checker != PDF_LEADING_HEADER {
                return Err("Must not be PDF (File should start with PDF header)".to_owned());
            }
        }
        Err(err) => return Err(format!("Failed to read ({})", err)),
    };
    return Ok(());
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    buffer: Vec<u8>,
    page_indexes: Vec<usize>,
}

pub fn search(search_term: &str, filepath: &str) -> Result<SearchResult, String> {
    let pdfium = match pdfium() {
        Ok(x) => x,
        Err(err) => return Err(err),
    };

    // todo: password protected doc support
    let document = pdfium
        .load_pdf_from_file(filepath, None)
        .expect("Failed to open file");

    let mut page_indexes = Vec::<usize>::new();
    document
        .pages()
        .iter()
        .enumerate()
        .for_each(|(page_index, mut page)| {
            let search_options = PdfSearchOptions::new();
            let mut search_results_bounds = page
                .text()
                .expect("Failed to read")
                .search(search_term, &search_options)
                .iter(PdfSearchDirection::SearchForward)
                .enumerate()
                .flat_map(|(_index, segments)| {
                    segments
                        .iter()
                        .map(|segment| {
                            // println!(
                            //     "Search result {}: `{}` appears at {:#?}",
                            //     index,
                            //     segment.text(),
                            //     segment.bounds()
                            // );
                            segment.bounds()
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            if 0 < search_results_bounds.len() {
                page_indexes.push(page_index);
            }

            let highlight_color = Some(PdfColor::YELLOW.with_alpha(128));
            for result in search_results_bounds.drain(..) {
                page.objects_mut()
                    .create_path_object_rect(result, None, None, highlight_color)
                    .expect("Failed to add hilighting annotation");
            }
        });

    let buffer = document.save_to_bytes().expect("Failed to write as buffer");

    Ok(SearchResult {
        buffer,
        page_indexes,
    })
}

fn pdfium() -> Result<Pdfium, String> {
    let pdfium = match std::panic::catch_unwind(|| {
        Pdfium::new(pdfium_bind_to_library().expect("Failed to bind to pdfium lib"))
    }) {
        Ok(x) => x,
        Err(_) => return Err("pdfium lib must be missing".to_owned()),
    };
    Ok(pdfium)
}

fn pdfium_bind_to_library() -> Result<Box<dyn PdfiumLibraryBindings>, PdfiumError> {
    const APP_LIBPDFIUM_DIR: &str = "lib/pdfium/lib";
    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
        std::env::current_exe()
            .expect("Failed to get exec path")
            .parent()
            .expect("Failed to get exec parent dir path")
            .join(APP_LIBPDFIUM_DIR)
            .as_path(),
    ))
    .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")))
    .or_else(|_| Pdfium::bind_to_system_library())
}
