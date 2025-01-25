use clap::Parser;
use serde_hjson::Value;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Parser, Debug)]
struct NormalizerConfig {
    #[clap(
        short,
        long,
        default_value_t = std::thread::available_parallelism().unwrap().get()-1
    )]
    threads: usize,
    #[clap(short, long, default_value_t = false)]
    formatted: bool,
    #[clap(short, long, default_value_t = String::from("./"))]
    directory: String,
}

fn write_json_to_file(value: &Value, path: &Path) -> std::io::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, value)?;
    Ok(())
}

fn main() {
    let config = NormalizerConfig::parse();
    println!("Normalizing json!");
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build_global()
        .unwrap();

    let path: Box<Path> = Box::from(Path::new(&config.directory));
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .try_for_each(|entry| {
            if !entry.metadata().unwrap().is_file() {
                return Ok::<(), anyhow::Error>(());
            }

            let extension = entry.path().extension();
            if extension.is_none() {
                return Ok::<(), anyhow::Error>(());
            }
            let extension = extension.unwrap();

            if extension != "json" && extension != "hjson" {
                return Ok::<(), anyhow::Error>(());
            }

            let str = std::fs::read_to_string(entry.path())?;

            let value: Value = serde_hjson::from_str(&str)?;

            write_json_to_file(&value, entry.path()).unwrap();
            Ok(())
        })
        .expect("Failed to normalize")
}
