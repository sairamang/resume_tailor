mod ai_provider;
mod resume_analysis;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

use ai_provider::ProviderConfig;
use resume_analysis::{analyze_resume, ResumeAnalysis};

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} <input-file> [--analyze]", program_name);
}

fn read_input(path: &Path) -> Result<String, String> {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
    {
        pdf_extract::extract_text(path).map_err(|err| {
            format!("Failed to extract text from PDF '{}': {}", path.display(), err)
        })
    } else {
        fs::read_to_string(path).map_err(|err| format!("Failed to read '{}': {}", path.display(), err))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args.first().map(|s| s.as_str()).unwrap_or("resume_tailor");

    if args.len() < 2 || args.len() > 3 {
        print_usage(program_name);
        process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let analyze = args.get(2).map(|value| value == "--analyze").unwrap_or(false);

    match read_input(input_path) {
        Ok(contents) => {
            if analyze {
                let config = match ProviderConfig::from_env() {
                    Ok(config) => config,
                    Err(err) => {
                        eprintln!("{}", err);
                        process::exit(2);
                    }
                };

                match analyze_resume(&contents, &config) {
                    Ok(analysis) => print_analysis(&analysis),
                    Err(err) => {
                        eprintln!("{}", err);
                        process::exit(2);
                    }
                }
            } else {
                println!("{}", contents);
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(2);
        }
    }
}

fn print_analysis(analysis: &ResumeAnalysis) {
    println!("Score: {}/10", analysis.score);
    println!("Summary: {}", analysis.summary);
    println!("Strengths:");
    for strength in &analysis.strengths {
        println!("- {}", strength);
    }
    println!("Improvements:");
    for improvement in &analysis.improvements {
        println!("- {}", improvement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn sample_pdf_bytes(text: &str) -> Vec<u8> {
        let mut pdf = Vec::new();
        pdf.extend_from_slice(b"%PDF-1.4\n");

        let mut object_offsets = Vec::new();
        let mut add_object = |body: String| {
            object_offsets.push(pdf.len());
            pdf.extend_from_slice(body.as_bytes());
        };

        add_object(format!("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n"));
        add_object(format!("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n"));
        add_object(format!("3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 144] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n"));

        let stream_data = format!("BT\n/F1 24 Tf\n72 72 Td\n({text}) Tj\nET\n");
        let stream_length = stream_data.len();
        add_object(format!(
            "4 0 obj\n<< /Length {stream_length} >>\nstream\n{stream_data}endstream\nendobj\n"
        ));
        add_object(format!("5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n"));

        let xref_start = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", object_offsets.len() + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for offset in object_offsets {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(b"trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n");
        pdf.extend_from_slice(xref_start.to_string().as_bytes());
        pdf.extend_from_slice(b"\n%%EOF\n");

        pdf
    }

    #[test]
    fn reads_plain_text_files() {
        let temp_dir = std::env::temp_dir().join(format!(
            "resume_tailor-test-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("sample.txt");
        fs::write(&path, "Hello from plain text").unwrap();
        let contents = read_input(&path).unwrap();
        assert_eq!(contents, "Hello from plain text");
    }

    #[test]
    fn extracts_text_from_pdf_files() {
        let temp_dir = std::env::temp_dir().join(format!(
            "resume_tailor-test-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("sample.pdf");
        fs::write(&path, sample_pdf_bytes("Hello from PDF")).unwrap();
        let contents = read_input(&path).unwrap();
        assert!(contents.contains("Hello from PDF"));
    }
}
