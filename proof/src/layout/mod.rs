//! Persistent 2D Geometric Layout Serialization (Stage 11).
//!
//! Allows persisting interactive canvas coordinates, constraints, and visual
//! metadata alongside formal proof files.

use crate::id::PointId;

/// Visual layout configuration for a single point.
#[derive(Debug, Clone, PartialEq)]
pub struct PointLayout {
    pub id: PointId,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub fixed: bool,
}

/// Complete persisted scene layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneLayout {
    pub name: String,
    pub points: Vec<PointLayout>,
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
}

impl Default for SceneLayout {
    fn default() -> Self {
        Self {
            name: "default".into(),
            points: Vec::new(),
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    InvalidJson(String),
    MissingField(String),
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::InvalidJson(msg) => write!(f, "Invalid layout JSON: {}", msg),
            LayoutError::MissingField(fld) => write!(f, "Missing layout field: {}", fld),
        }
    }
}

impl std::error::Error for LayoutError {}

impl SceneLayout {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn add_point(&mut self, id: PointId, name: impl Into<String>, x: f64, y: f64, fixed: bool) {
        self.points.push(PointLayout {
            id,
            name: name.into(),
            x,
            y,
            fixed,
        });
    }

    pub fn to_json(&self) -> String {
        let mut pts_json = Vec::new();
        for p in &self.points {
            pts_json.push(format!(
                "{{\"id\": {}, \"name\": \"{}\", \"x\": {:.2}, \"y\": {:.2}, \"fixed\": {}}}",
                p.id.0, p.name, p.x, p.y, p.fixed
            ));
        }
        format!(
            "{{\n  \"name\": \"{}\",\n  \"zoom\": {:.2},\n  \"pan_x\": {:.2},\n  \"pan_y\": {:.2},\n  \"points\": [\n    {}\n  ]\n}}",
            self.name,
            self.zoom,
            self.pan_x,
            self.pan_y,
            pts_json.join(",\n    ")
        )
    }

    /// Fast lightweight parser for SceneLayout JSON without external dependencies.
    pub fn from_json(json: &str) -> Result<Self, LayoutError> {
        let mut layout = SceneLayout::default();

        // Extract name
        if let Some(pos) = json.find("\"name\":") {
            let rest = &json[pos + 7..];
            if let Some(quote1) = rest.find('"') {
                let rest2 = &rest[quote1 + 1..];
                if let Some(quote2) = rest2.find('"') {
                    layout.name = rest2[..quote2].to_string();
                }
            }
        }

        // Extract zoom
        if let Some(pos) = json.find("\"zoom\":") {
            let rest = &json[pos + 7..];
            let num_str: String = rest.chars().skip_while(|c| c.is_whitespace()).take_while(|c| c.is_digit(10) || *c == '.' || *c == '-').collect();
            if let Ok(v) = num_str.parse::<f64>() {
                layout.zoom = v;
            }
        }

        // Extract points array
        if let Some(start_pts) = json.find("\"points\":") {
            let rest = &json[start_pts + 9..];
            if let Some(arr_start) = rest.find('[') {
                if let Some(arr_end) = rest.find(']') {
                    let array_content = &rest[arr_start + 1..arr_end];
                    for chunk in array_content.split('}') {
                        if !chunk.contains('{') {
                            continue;
                        }
                        let item = chunk.trim().trim_start_matches('{');
                        let mut pid = 0;
                        let mut name = String::new();
                        let mut x = 0.0;
                        let mut y = 0.0;
                        let mut fixed = false;

                        for field in item.split(',') {
                            let parts: Vec<&str> = field.split(':').collect();
                            if parts.len() == 2 {
                                let key = parts[0].trim().trim_matches('"');
                                let val = parts[1].trim();
                                match key {
                                    "id" => pid = val.parse().unwrap_or(0),
                                    "name" => name = val.trim_matches('"').to_string(),
                                    "x" => x = val.parse().unwrap_or(0.0),
                                    "y" => y = val.parse().unwrap_or(0.0),
                                    "fixed" => fixed = val.parse().unwrap_or(false),
                                    _ => {}
                                }
                            }
                        }

                        if pid > 0 || !name.is_empty() {
                            layout.points.push(PointLayout {
                                id: PointId(pid),
                                name,
                                x,
                                y,
                                fixed,
                            });
                        }
                    }
                }
            }
        }

        Ok(layout)
    }
}
