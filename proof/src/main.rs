use proof::api::{process_proof_request, extract_json_string_field};
use proof::incremental::IncrementalEngine;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn is_allowed_origin(origin: &str) -> bool {
    let lower = origin.trim().to_ascii_lowercase();
    lower.starts_with("http://localhost:")
        || lower.starts_with("http://127.0.0.1:")
        || lower == "http://localhost"
        || lower == "http://127.0.0.1"
        || lower.starts_with("https://localhost:")
        || lower.starts_with("https://127.0.0.1:")
        || lower.starts_with("tauri://")
        || lower.starts_with("https://tauri.localhost")
        || lower.starts_with("vscode-webview://")
}

fn read_http_request(stream: &mut TcpStream) -> Result<(String, String), std::io::Error> {
    let mut buffer = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];

    let header_end = loop {
        if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos + 4;
        }
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Client disconnected"));
        }
        buffer.extend_from_slice(&chunk[..n]);
        if buffer.len() > 65536 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "HTTP headers exceed 64KB limit"));
        }
    };

    let header_str = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut content_length = 0;
    for line in header_str.lines() {
        if line.to_ascii_lowercase().starts_with("content-length:") {
            if let Some(val_str) = line.split(':').nth(1) {
                content_length = val_str.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    const MAX_BODY: usize = 10 * 1024 * 1024; // 10MB limit
    if content_length > MAX_BODY {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Body too large"));
    }

    let already_read_body = buffer.len() - header_end;
    if already_read_body < content_length {
        let remaining = content_length - already_read_body;
        let mut remaining_buf = vec![0u8; remaining];
        stream.read_exact(&mut remaining_buf)?;
        buffer.extend_from_slice(&remaining_buf);
    }

    let body_str = String::from_utf8_lossy(&buffer[header_end..header_end + content_length]).to_string();
    Ok((header_str, body_str))
}

fn handle_client(mut stream: TcpStream) {
    let (headers, body) = match read_http_request(&mut stream) {
        Ok(res) => res,
        Err(_) => return,
    };

    let origin = headers.lines().find_map(|l| {
        if l.to_ascii_lowercase().starts_with("origin:") {
            Some(l[7..].trim().to_string())
        } else {
            None
        }
    });

    let cors_header = match &origin {
        Some(o) if is_allowed_origin(o) => format!("Access-Control-Allow-Origin: {}\r\nVary: Origin\r\n", o),
        _ => String::new(),
    };

    // Handle CORS preflight
    if headers.starts_with("OPTIONS") {
        let response = format!(
            "HTTP/1.1 204 No Content\r\n{}Access-Control-Allow-Methods: POST, GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n",
            cors_header
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    // Handle POST verification or synthesis request
    if headers.starts_with("POST") {
        let is_synthesize = headers.starts_with("POST /api/synthesize")
            || body.contains("\"action\":\"synthesize\"")
            || body.contains("\"synthesize\":true");

        let code = extract_json_string_field(&body, "code").unwrap_or_else(|| body.clone());

        let result_json = if is_synthesize {
            let step = proof::synthesizer::ProofSynthesizer::infill_next_step(&code);
            let candidates = proof::synthesizer::ProofSynthesizer::synthesize_candidates(&code);
            let step_str = step.map(|s| s.to_json()).unwrap_or_else(|| "null".to_string());
            let candidates_str: Vec<String> = candidates.iter().map(|c| c.to_json()).collect();
            format!(
                "{{\"step\": {}, \"candidates\": [{}]}}",
                step_str,
                candidates_str.join(", ")
            )
        } else {
            process_proof_request(&code)
        };

        let response = format!(
            "HTTP/1.1 200 OK\r\nConnection: close\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            cors_header,
            result_json.len(),
            result_json
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    // Serve frontend static files safely
    if headers.starts_with("GET") {
        let first_line = headers.lines().next().unwrap_or("");
        let mut req_path = "/";
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            req_path = parts[1];
        }

        // Clean query parameters and fragments
        let clean_url = req_path.split('?').next().unwrap_or("").split('#').next().unwrap_or("");

        // Defense-in-depth: Immediately reject path traversal attempts
        if clean_url.contains("..") || clean_url.contains('\0') || clean_url.contains('\\') {
            let forbidden = "HTTP/1.1 403 Forbidden\r\nConnection: close\r\nContent-Type: text/plain\r\n\r\n403 Forbidden: Invalid Path";
            let _ = stream.write_all(forbidden.as_bytes());
            return;
        }

        let rel_path = if clean_url == "/" || clean_url.is_empty() {
            "index.html"
        } else {
            clean_url.trim_start_matches('/')
        };

        // Allowed static search directories
        let static_roots = [
            std::path::PathBuf::from("static"),
            std::path::PathBuf::from("proof/static"),
        ];

        let mut served = false;
        for root in &static_roots {
            if let Ok(canonical_root) = root.canonicalize() {
                let candidate = canonical_root.join(rel_path);
                if let Ok(canonical_candidate) = candidate.canonicalize() {
                    // Strict boundary check: file must be inside canonical root
                    if canonical_candidate.starts_with(&canonical_root) && canonical_candidate.is_file() {
                        if let Ok(bytes) = std::fs::read(&canonical_candidate) {
                            let content_type = if rel_path.ends_with(".html") {
                                "text/html; charset=utf-8"
                            } else if rel_path.ends_with(".js") {
                                "application/javascript; charset=utf-8"
                            } else if rel_path.ends_with(".css") {
                                "text/css; charset=utf-8"
                            } else if rel_path.ends_with(".svg") {
                                "image/svg+xml"
                            } else if rel_path.ends_with(".wasm") {
                                "application/wasm"
                            } else {
                                "application/octet-stream"
                            };

                            let header = format!(
                                "HTTP/1.1 200 OK\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                                content_type,
                                bytes.len()
                            );
                            let _ = stream.write_all(header.as_bytes());
                            let _ = stream.write_all(&bytes);
                            served = true;
                            break;
                        }
                    }
                }
            }
        }

        if served {
            return;
        }

        if clean_url != "/" {
            let not_found = "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Type: text/plain\r\n\r\n404 Not Found";
            let _ = stream.write_all(not_found.as_bytes());
            return;
        }
    }

    // Default status
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nProofer Daemon Online";
    let _ = stream.write_all(response.as_bytes());
}

/// Execute batch verification on a source file.
pub fn execute_batch_check(file_path: &Path) -> Result<bool, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Cannot read file {:?}: {}", file_path, e))?;

    let mut engine = IncrementalEngine::new();
    let result = engine.compile_source(&content, None)
        .map_err(|e| format!("Verification error: {}", e))?;

    println!("Checking {:?}", file_path);
    for thm in &result.theorems {
        let status = if thm.is_verified() { "PASS" } else { "FAIL" };
        println!("  [{}] theorem '{}'", status, thm.name);
    }
    println!("Completed in {} µs (Reused: {}, Recompiled: {})",
        result.elapsed_micros, result.reused_count, result.recompiled_count);

    Ok(result.verified)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--server" | "serve" => {
                let port: u16 = std::env::var("PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8086);
                let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
                let bind_addr = format!("{}:{}", host, port);
                let listener = TcpListener::bind(&bind_addr).unwrap_or_else(|e| {
                    eprintln!("Failed to bind server to {}: {}", bind_addr, e);
                    std::process::exit(1);
                });
                println!("Proofer Verification Server listening on http://{}", bind_addr);
                for stream in listener.incoming() {
                    if let Ok(s) = stream {
                        let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                        let _ = s.set_write_timeout(Some(std::time::Duration::from_secs(5)));
                        std::thread::spawn(|| handle_client(s));
                    }
                }
            }
            "lsp" | "--lsp" => {
                let _ = proof::lsp::run_server();
            }
            "daemon" => {
                let stdin = std::io::stdin();
                let stdout = std::io::stdout();
                let mut reader = BufReader::new(stdin.lock());
                let mut writer = stdout.lock();
                let mut line = String::new();

                while let Ok(n) = reader.read_line(&mut line) {
                    if n == 0 {
                        break;
                    }
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        let code = extract_json_string_field(trimmed, "code")
                            .unwrap_or_else(|| trimmed.to_string());

                        let result = process_proof_request(&code);
                        let _ = writeln!(writer, "{}", result);
                        let _ = writer.flush();
                    }
                    line.clear();
                }
            }
            "verify" => {
                let code = if args.len() > 2 && args[2] != "-" {
                    match std::fs::read_to_string(&args[2]) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("Error reading file {}: {}", args[2], e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    let mut buffer = String::new();
                    if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
                        eprintln!("Error reading from stdin: {}", e);
                        std::process::exit(1);
                    }
                    buffer
                };
                let result = process_proof_request(&code);
                println!("{}", result);
            }
            "check" => {
                if args.len() < 3 {
                    eprintln!("Usage: proofer check [--json] <file.proof>");
                    std::process::exit(1);
                }
                let is_json = args.iter().any(|a| a == "--json");
                let file_arg = args.iter().skip(2).find(|a| *a != "--json");
                let path_str = match file_arg {
                    Some(p) => p,
                    None => {
                        eprintln!("Usage: proofer check [--json] <file.proof>");
                        std::process::exit(1);
                    }
                };
                let path = Path::new(path_str);
                if is_json {
                    match std::fs::read_to_string(path) {
                        Ok(code) => {
                            let result = process_proof_request(&code);
                            println!("{}", result);
                        }
                        Err(e) => {
                            eprintln!("Cannot read file {:?}: {}", path, e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    match execute_batch_check(path) {
                        Ok(true) => {
                            println!("Verification succeeded.");
                            std::process::exit(0);
                        }
                        Ok(false) => {
                            eprintln!("Verification failed.");
                            std::process::exit(1);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
            "repl" => {
                println!("Proofer Interactive REPL v0.1.0");
                println!("Type theorem definitions or 'quit' to exit.\n");
                let mut engine = IncrementalEngine::new();
                let stdin = std::io::stdin();
                let mut buffer = String::new();

                loop {
                    print!("proofer> ");
                    let _ = std::io::stdout().flush();
                    buffer.clear();
                    if stdin.read_line(&mut buffer).is_err() || buffer.trim() == "quit" {
                        break;
                    }
                    let trimmed = buffer.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match engine.compile_source(trimmed, None) {
                        Ok(res) => {
                            for thm in res.theorems {
                                println!("  {} (verified: {})", thm.name, thm.is_verified());
                            }
                        }
                        Err(e) => println!("  Error: {}", e),
                    }
                }
            }
            "new" => {
                if args.len() < 3 {
                    eprintln!("Usage: proofer new <path_or_name>");
                    std::process::exit(1);
                }
                let target_path = Path::new(&args[2]);
                let project_name = target_path.file_name().and_then(|n| n.to_str()).unwrap_or("proof_project");
                match proof::package::PackageManager::create_project(target_path, project_name) {
                    Ok(()) => {
                        println!("Created new Proofer package '{}' at {:?}", project_name, target_path);
                    }
                    Err(e) => {
                        eprintln!("Failed to create project: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "build" => {
                let target_dir = if args.len() > 2 {
                    Path::new(&args[2])
                } else {
                    Path::new(".")
                };
                match proof::package::PackageManager::build_project(target_dir) {
                    Ok(summary) => {
                        println!("Package: {} v{}", summary.package_name, summary.version);
                        println!("Files: {}, Theorems: {} (Verified: {})", summary.files_count, summary.theorems_count, summary.verified_count);
                        if !summary.errors.is_empty() {
                            eprintln!("\nVerification Errors:");
                            for err in summary.errors {
                                eprintln!("  - {}", err);
                            }
                            std::process::exit(1);
                        } else {
                            println!("All theorems in package verified successfully.");
                        }
                    }
                    Err(e) => {
                        eprintln!("Build failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "synthesize" => {
                if args.len() < 3 {
                    eprintln!("Usage: proofer synthesize <file.proof>");
                    std::process::exit(1);
                }
                let path = Path::new(&args[2]);
                match std::fs::read_to_string(path) {
                    Ok(code) => {
                        if let Some(step) = proof::synthesizer::ProofSynthesizer::infill_next_step(&code) {
                            println!("Synthesized Step (Kernel Verified: {}):", step.verified_by_kernel);
                            println!("  {}", step.text);
                            println!("Explanation: {}", step.explanation);
                        } else {
                            println!("No valid verified step could be synthesized for this context.");
                        }
                    }
                    Err(e) => {
                        eprintln!("Cannot read file {:?}: {}", path, e);
                        std::process::exit(1);
                    }
                }
            }
            _ => {
                println!("Unknown command: {}", args[1]);
                println!("Usage:");
                println!("  proofer --server          Start verification daemon");
                println!("  proofer check <file>      Batch check proof file");
                println!("  proofer repl              Start interactive REPL");
                println!("  proofer new <dir>         Create new proof package");
                println!("  proofer build [dir]       Build and verify entire package");
                println!("  proofer synthesize <file> Synthesize next verified proof step");
            }
        }
    } else {
        println!("Proofer Proof System");
        println!("Usage:");
        println!("  proofer --server          Start verification daemon");
        println!("  proofer check <file>      Batch check proof file");
        println!("  proofer repl              Start interactive REPL");
        println!("  proofer new <dir>         Create new proof package");
        println!("  proofer build [dir]       Build and verify entire package");
        println!("  proofer synthesize <file> Synthesize next verified proof step");
    }
}
