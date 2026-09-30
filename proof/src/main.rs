use proof::parser::Parser;
use proof::hir::resolve::Resolver;
use proof::elab::Elaborator;
use proof::ast::{Item, ProofStepKind};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;

/// Helper to serialize a string for JSON
fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('\"', "\\\"").replace('\n', "\\n"))
}

/// Verification payload returned to the frontend according to enterprise editor specification.
fn process_proof_request(code: &str) -> String {
    let mut parser = Parser::new(code);
    let file_ast = parser.parse_file();

    let parser_errors: Vec<String> = parser.diagnostics().iter().map(|d| d.message.clone()).collect();
    if !parser_errors.is_empty() {
        let errs_json: Vec<String> = parser_errors.iter().map(|e| json_str(e)).collect();
        return format!(
            "{{\"status\": \"Error\", \"stage\": \"Parser\", \"errors\": [{}], \"verified\": false}}",
            errs_json.join(", ")
        );
    }

    let mut resolver = Resolver::new();
    let pkg = resolver.resolve_file(&file_ast);
    let resolver_errors: Vec<String> = resolver.diagnostics().iter().map(|d| d.message.clone()).collect();
    if !resolver_errors.is_empty() {
        let errs_json: Vec<String> = resolver_errors.iter().map(|e| json_str(e)).collect();
        return format!(
            "{{\"status\": \"Error\", \"stage\": \"Resolver\", \"errors\": [{}], \"verified\": false}}",
            errs_json.join(", ")
        );
    }

    let mut elaborator = Elaborator::new();
    let results = elaborator.elaborate_package(&pkg);

    // Map each theorem's AST steps to rich JSON steps
    let mut ast_theorems: HashMap<String, &proof::ast::Theorem> = HashMap::new();
    for item in &file_ast.items {
        if let Item::Theorem(thm) = item {
            ast_theorems.insert(thm.name.clone(), thm);
        }
    }

    let mut theorems_json = Vec::new();
    let mut all_verified = true;

    for res in results {
        let ast_thm = ast_theorems.get(&res.name);
        let mut steps_json = Vec::new();

        if let Some(thm) = ast_thm {
            for (idx, step) in thm.proof.iter().enumerate() {
                let (step_text, rule_opt, conclusion_opt) = match &step.kind {
                    ProofStepKind::Suppose { name, prop } => {
                        (format!("suppose {} : {}", name, prop), None, Some(format!("{}", prop)))
                    }
                    ProofStepKind::Have { name, prop, from, .. } => {
                        let name_part = name.as_deref().unwrap_or("h");
                        let from_part = if from.is_empty() { "".into() } else { format!(" from {}", from.join(", ")) };
                        (format!("have {} : {}{}", name_part, prop, from_part), None, Some(format!("{}", prop)))
                    }
                    ProofStepKind::Derive { name, prop, from, using_rule } => {
                        let name_part = name.as_deref().unwrap_or("h");
                        let from_part = if from.is_empty() { "".into() } else { format!(" from {}", from.join(", ")) };
                        let using_part = using_rule.as_deref().map(|r| format!(" using {}", r)).unwrap_or_default();
                        (format!("derive {} : {}{}{}", name_part, prop, from_part, using_part), using_rule.clone(), Some(format!("{}", prop)))
                    }
                    ProofStepKind::Therefore { prop, from } => {
                        let from_part = if from.is_empty() { "".into() } else { format!(" from {}", from.join(", ")) };
                        (format!("therefore {}{}", prop, from_part), None, Some(format!("{}", prop)))
                    }
                    ProofStepKind::Take(binders) => {
                        let b_names: Vec<String> = binders.iter().map(|b| b.name.clone()).collect();
                        (format!("take {}", b_names.join(", ")), None, None)
                    }
                    ProofStepKind::Construct { name, as_desc, args } => {
                        let a_str: Vec<String> = args.iter().map(|t| t.to_string()).collect();
                        (format!("construct {} as {} of {}", name, as_desc, a_str.join(", ")), None, None)
                    }
                    _ => ("proof step".into(), None, None),
                };

                let rule_field = match rule_opt {
                    Some(r) => format!(", \"rule\": {}", json_str(&r)),
                    None => "".into(),
                };
                let concl_field = match conclusion_opt {
                    Some(c) => format!(", \"conclusion\": {}", json_str(&c)),
                    None => "".into(),
                };

                steps_json.push(format!(
                    "{{\"id\": {}, \"text\": {}, \"status\": \"Valid\"{}{}}}",
                    idx + 1,
                    json_str(&step_text),
                    rule_field,
                    concl_field
                ));
            }
        }

        if let Some(proven) = res.proven {
            theorems_json.push(format!(
                "{{\"name\": {}, \"status\": \"Verified\", \"proven\": {}, \"steps\": [{}]}}",
                json_str(&res.name),
                json_str(&proven.to_string()),
                steps_json.join(", ")
            ));
        } else {
            all_verified = false;
            let err_msgs: Vec<String> = res.errors.iter().map(|e| json_str(&format!("{:?}", e))).collect();
            theorems_json.push(format!(
                "{{\"name\": {}, \"status\": \"Rejected\", \"errors\": [{}], \"steps\": [{}]}}",
                json_str(&res.name),
                err_msgs.join(", "),
                steps_json.join(", ")
            ));
        }
    }

    // Geometry Scene Generation
    let mut scene_json = String::from("{}");
    if let Some(fig) = pkg.figures.first() {
        let doc_scene = proof::editor::GeometryScene::from_geo_figure(fig);
        let mut pts_json = Vec::new();
        for (id, pos) in &doc_scene.point_positions {
            pts_json.push(format!("{{\"id\": \"pt#{}\", \"x\": {}, \"y\": {}}}", id.0, pos.x, pos.y));
        }
        scene_json = format!("{{\"name\": {}, \"points\": [{}]}}", json_str(&fig.name), pts_json.join(", "));
    }

    format!(
        "{{\"status\": \"Ok\", \"verified\": {}, \"theorems\": [{}], \"geometryScene\": {}}}",
        all_verified,
        theorems_json.join(", "),
        scene_json
    )
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 65536];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return,
    };
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    if request.starts_with("OPTIONS") {
        let response = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    if request.contains("POST /api/verify") {
        let body = if let Some(idx) = request.find("\r\n\r\n") {
            &request[idx + 4..]
        } else {
            ""
        };

        // Extract code from json or raw body
        let code = if let Some(start) = body.find("\"code\":\"") {
            let rest = &body[start + 8..];
            if let Some(end) = rest.find("\"}") {
                rest[..end].replace("\\n", "\n").replace("\\\"", "\"")
            } else {
                body.to_string()
            }
        } else {
            body.to_string()
        };

        let result_json = process_proof_request(&code);
        let response = format!(
            "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            result_json.len(),
            result_json
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    // Serve static files
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nProofer Daemon Online";
    let _ = stream.write_all(response.as_bytes());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--server" {
        let port = 8086;
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind server");
        println!("🚀 Proofer Verification Server listening on http://127.0.0.1:{}", port);
        for stream in listener.incoming() {
            if let Ok(s) = stream {
                std::thread::spawn(|| handle_client(s));
            }
        }
    } else {
        println!("Run with `cargo run -- --server` to start the live interactive verification daemon.");
    }
}
