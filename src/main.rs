use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "centralized-file-server-cli",
    about = "Linha de comando para interagir com servidor centralizado de armazenamento de arquivos"
)]
struct Args {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    List,
    Download {
        /// ID do arquivo que deve ser feito download
        #[structopt(long)]
        id: i32,
    },
    Upload {
        /// Caminho do arquivo que deve ser feito upload
        #[structopt(long)]
        path: String,
    },
}

#[derive(Deserialize, Debug)]
struct FileEntity {
    id: i32,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileContent {
    name: String,
    content: Vec<u8>,
}

impl FileContent {
    pub fn new(name: String, content: Vec<u8>) -> Self {
        Self { name, content }
    }
}
#[paw::main]
fn main(args: Args) {
    match args.command {
        Command::List => list_files(),
        Command::Download { id } => download_file(id),
        Command::Upload { path } => upload_file(path),
    }
}

/// Lista os arquivos disponíves no servidor
fn list_files() {
    let result = create_http_client()
        .get("http://localhost:3000/list")
        .send();
    match result {
        Ok(response) => {
            let files: Vec<FileEntity> =
                serde_json::from_str(response.text().unwrap().as_str()).unwrap();
            for file in files {
                println!("{} - {}", file.id, file.name);
            }
        }
        Err(_) => println!("Erro ao obter os arquivos do servidor!"),
    }
}

/// Faz o download do arquivo com o ID especificado
fn download_file(id: i32) {
    let url = format!("http://localhost:3000/download/{}", id);
    let result = create_http_client().get(url).send();
    match result {
        Ok(response) => {
            let file: FileContent = serde_json::from_str(response.text().unwrap().as_str()).unwrap();
            let mut arquivo = File::create(file.name).unwrap();
            let _ = arquivo.write_all(&file.content);
        }
        Err(_) => println!("Erro ao obter os dados do arquivo no servidor!"),
    }
}

/// Faz o upload do arquivo com o caminho especificado
fn upload_file(path: String) {
    let file_path = Path::new(&path);
    let file_name = file_path.file_name().unwrap().to_str().unwrap();
    let content: Vec<u8> = fs::read(&path).unwrap();
    let raw_file = FileContent::new(file_name.to_string(), content);
    let payload = serde_json::to_string(&raw_file).unwrap();
    let result = create_http_client()
        .post("http://localhost:3000/upload")
        .header("Content-Type", "application/json")
        .body(payload)
        .send();
    match result {
        Ok(response) => println!("ok {}", response.status()),
        Err(e) => println!("{}", e.to_string()),
    }
}

fn create_http_client() -> Client {
    Client::builder().no_proxy().build().unwrap()
}
