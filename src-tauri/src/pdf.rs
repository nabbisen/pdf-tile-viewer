use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

pub fn read(filepath: &str) -> Result<Vec<Vec<u8>>, String> {
    let mut file = match File::open(filepath) {
        Ok(x) => x,
        Err(err) => return Err(format!("Failed to open ({})", err)),
    };

    match validate_pdf(&mut file) {
        Ok(_) => {}
        Err(err) => return Err(err),
    };

    let pdfium = match pdfium() {
        Ok(x) => x,
        Err(err) => return Err(err),
    };
    let document = match pdfium.load_pdf_from_file(filepath, None) {
        Ok(x) => x,
        Err(err) => return Err(err.to_string()),
    };
    let page_buffers = page_buffers(document, &pdfium).expect("Failed to get matched page buffers");

    Ok(page_buffers)
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
    page_buffers: Vec<Vec<u8>>,
    page_indexes: Vec<usize>,
}

pub fn search(search_term: &str, filepath: &str) -> Result<SearchResult, String> {
    let pdfium = match pdfium() {
        Ok(x) => x,
        Err(err) => return Err(err),
    };

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

    let page_buffers = page_buffers(document, &pdfium).expect("Failed to get matched page buffers");

    Ok(SearchResult {
        page_buffers,
        page_indexes,
    })
}

fn pdfium() -> Result<Pdfium, String> {
    let pdfium = match std::panic::catch_unwind(|| {
        Pdfium::new(pdfium_bind_to_library().expect("Failed to bind to pdfium lib"))
    }) {
        Ok(x) => Ok(x),
        Err(_) => return Err("libpdfium.so must be missing".to_owned()),
    };
    pdfium
}

fn pdfium_bind_to_library() -> Result<Box<dyn PdfiumLibraryBindings>, PdfiumError> {
    const APP_LIBPDFIUM_DIR: &str = "lib/pdfium/lib";
    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
        std::env::current_exe()
            .expect("Failed to get exec dir")
            .parent()
            .expect("Failed to get exec parent dir")
            .join(APP_LIBPDFIUM_DIR)
            .as_path(),
    ))
    .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")))
    .or_else(|_| Pdfium::bind_to_system_library())
}

fn page_buffers(document: PdfDocument, pdfium: &Pdfium) -> Result<Vec<Vec<u8>>, String> {
    let page_buffers = document
        .pages()
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let mut new_document = pdfium.create_new_pdf().expect("Failed to create new doc");

            new_document
                .pages_mut()
                .copy_page_from_document(&document, i as u16, 0)
                .expect("Failed to copy on single page doc");

            new_document
                .save_to_bytes()
                .expect("Failed to save page as buffer")
        })
        .collect();

    Ok(page_buffers)
}
