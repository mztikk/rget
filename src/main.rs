use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use reqwest::blocking::Response;
use std::fs::File;
use structopt::StructOpt;

/// Downloads the given uri
#[derive(StructOpt)]
struct Cli {
    /// URI to download
    uri: String,

    /// Optional filename, otherwise will be taken from response or uri
    #[structopt(short = "f", long = "filename")]
    filename: Option<String>,
}

fn filename_from_headers(resp: &Response) -> Result<String, String> {
    match resp.headers().get("Content-Disposition") {
        Some(header) => match header.to_str() {
            Ok(content_disposition) => match content_disposition.rfind("filename=") {
                Some(filename_index) => {
                    Ok(content_disposition[filename_index + "filename=".len()..].to_string())
                }
                None => Err(format!(
                    "Couldn't read filename from Content-Disposition: {}",
                    content_disposition
                )),
            },
            Err(e) => Err(e.to_string()),
        },
        None => Err("No Content-Disposition Header".to_string()),
    }
}

fn filename_from_uri(resp: &Response) -> Result<String, String> {
    let uri = resp.url().to_string();
    match uri.rfind('/') {
        Some(last_slash) => {
            let remaining = &uri[last_slash + "/".len()..uri.len()];
            if !remaining.is_empty() {
                Ok(remaining.to_string())
            } else {
                Err(format!("URI has no trailing filename '{}'", uri))
            }
        }
        None => Err(format!("URI has no trailing filename '{}'", uri)),
    }
}

fn write_line(str: String) {
    println!("{}", str);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let args: Vec<String> = env::args().skip(1).collect();
    let args = Cli::from_args();

    let uri_arg = args.uri;

    let uri = if !uri_arg.starts_with("http://") && !uri_arg.starts_with("https://") {
        format!("http://{}", uri_arg)
    } else {
        uri_arg.to_string()
    };

    println!("Sending request to '{}'", &uri);
    match reqwest::blocking::get(&uri) {
        Ok(mut resp) => {
            let filename_getters = [filename_from_headers, filename_from_uri];

            let filename = args.filename.unwrap_or(
                filename_getters
                    .iter()
                    .find_map(|f| f(&resp).map_err(write_line).ok())
                    .unwrap_or("index.html".to_string()),
            );

            println!("Filename set to: '{}'", filename);

            let mut file = File::create(filename)?;

            let n_bytes = match resp.content_length() {
                Some(content_length) => content_length,
                None => 0,
            };
            if n_bytes != 0 {
                println!("Download size is: {}", HumanBytes(n_bytes));
            } else {
                println!("No size for download found");
            }

            let pb = ProgressBar::new(n_bytes);
            // pb.set_style(ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{wide_bar.cyan/blue}] {bytes}/{total_bytes} ({eta})").progress_chars("#>-"));
            pb.set_style(ProgressStyle::default_bar()
                .template("{msg}{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("#>-"));

            std::io::copy(&mut resp, &mut pb.wrap_write(&mut file))?;
            pb.finish_with_message("downloaded");
        }
        Err(e) => println!("Failed to GET URI: '{}' ({})", uri, e),
    }

    Ok(())
}
