use actix_web::{get, web, post, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use actix_web::http::header;
use serde::Deserialize;
use regex::Regex;
use std::process::Command;
use std::io;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

#[post("/compile")]
async fn compile(data: web::Json<Data>) -> Result<HttpResponse, io::Error> {
    let code = data.code.clone();
    let result: Result<String, io::Error>;
    
    if data.language == "python" {
        result = run_command(vec!["docker", "run", "--rm", "python:3", "python", "-c", &code]);
    } else if data.language == "javascript" {
        result = run_command(vec!["docker", "run", "--rm", "node:18", "node", "-e", &code]);
    } else if data.language == "cpp" {
        result = run_command(vec![
            "docker", 
            "run", 
            "--rm", 
            "gcc:latest", 
            "bash", 
            "-c", 
            &format!("echo '{}' > program.cpp && g++ -o program program.cpp && ./program", &code)
        ]);
    } else if data.language == "java" {
        let class_name = extract_class_name(code.clone());
        result = run_command(vec![
            "docker", 
            "run", 
            "--rm", 
            "openjdk:latest", 
            "bash", 
            "-c", 
            &format!("echo '{}' > Main.java && javac Main.java && java '{}'", &code, &class_name)
        ]);
    } else {
        result = Ok("Unsupported language".to_string());
    }

    match result {
        Ok(output) => Ok(HttpResponse::Ok().body(output)),
        Err(error) => Ok(HttpResponse::InternalServerError().body(format!("Error: {}", error))),
    }
}

fn run_command(args: Vec<&str>) -> Result<String, io::Error> {
    let output = Command::new(args[0])
        .args(&args[1..])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr).to_string();
        Err(io::Error::new(io::ErrorKind::Other, error_message))
    }
}

fn extract_class_name(code: String) -> String {
    let re = Regex::new(r"class\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{").unwrap();
    if let Some(captures) = re.captures(&code) {
        return captures[1].to_string();
    }
    "Main".to_string()
}

#[derive(Deserialize)]
struct Data {
    code: String,
    language: String
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .service(index)
            .service(compile)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}