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
    page_nums: Vec<usize>,
}

pub fn search(search_term: &str, filepath: &str) -> Result<SearchResult, String> {
    let pdfium = match std::panic::catch_unwind(|| {
        Pdfium::new(pdfium_bind_to_library().expect("Failed to bind to pdfium lib"))
    }) {
        Ok(x) => x,
        Err(_) => return Err("libpdfium.so must be missing".to_owned()),
    };

    let mut document = pdfium
        .load_pdf_from_file(filepath, None)
        .expect("Failed to open file");

    let mut page = document.pages().first().expect("Failed to get leading");

    let search_options = PdfSearchOptions::new();
    let mut search_results_bounds = page
        .text()
        .expect("Failed to read")
        .search(search_term, &search_options)
        .iter(PdfSearchDirection::SearchForward)
        .enumerate()
        .flat_map(|(index, segments)| {
            segments
                .iter()
                .map(|segment| {
                    println!(
                        "Search result {}: `{}` appears at {:#?}",
                        index,
                        segment.text(),
                        segment.bounds()
                    );

                    segment.bounds()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // We now have a list of page areas that contain the search target.
    // Highlight them in yellow...

    let highlight_color = Some(PdfColor::YELLOW.with_alpha(128));
    for result in search_results_bounds.drain(..) {
        page.objects_mut()
            .create_path_object_rect(result, None, None, highlight_color)
            .expect("Failed to add hilighting annotation");
    }

    // ... and save the result out to a new document.

    while document.pages().len() > 1 {
        document
            .pages_mut()
            .last()
            .expect("Failed to get last page")
            .delete()
            .expect("Failed to delete tailing");
    }

    let buffer = document.save_to_bytes().expect("Failed to write as buffer");

    let page_nums = Vec::<usize>::new();
    // let mut page_nums = Vec::<usize>::new();
    // for (i, page) in document.pages().iter().enumerate() {
    //     let text = page.text().expect("Failed to get page text");

    //     if text
    //         .search(search_term, &search_options)
    //         .find_next()
    //         .is_some()
    //     {
    //         page_nums.push(i + 1);
    //     }
    // }

    Ok(SearchResult { buffer, page_nums })
}

fn pdfium_bind_to_library() -> Result<Box<dyn PdfiumLibraryBindings>, PdfiumError> {
    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
        std::env::current_exe()
            .expect("Failed to get exec dir")
            .parent()
            .expect("Failed to get exec parent dir"),
    ))
    .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")))
    .or_else(|_| Pdfium::bind_to_system_library())
}
