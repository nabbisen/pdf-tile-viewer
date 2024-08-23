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
