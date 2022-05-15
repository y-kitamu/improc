use serde::Serialize;

use crate::feat::{keypoints::KeyPoint, matcher::Match, Distance};

pub struct ViewerWriter {
    filename: String,
    schemas: Vec<Schema>,
}

enum PartsType {
    image,
    point,
    line,
}

impl PartsType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PartsType::image => "image",
            PartsType::point => "point",
            PartsType::line => "line",
        }
    }
}

enum RenderMode {
    POINTS,
    LINES,
    LINE_LOOP,
    TRIANGLES,
    TRIANGLE_STRIP,
    TRIANGLE_FAN,
}

impl RenderMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RenderMode::POINTS => "POINTS",
            RenderMode::LINES => "LINES",
            RenderMode::LINE_LOOP => "LINE_LOOP",
            RenderMode::TRIANGLES => "TRIANGLES",
            RenderMode::TRIANGLE_STRIP => "TRIANGLE_STRIP",
            RenderMode::TRIANGLE_FAN => "TRIANGLE_FAN",
        }
    }
}

#[derive(Serialize)]
struct Schema {
    parts_type: &'static str,
    render_mode: &'static str,
    datas: Vec<Data>,
}

#[derive(Serialize)]
struct Data {
    variable_name: String,
    data: Vec<f32>,
}

impl ViewerWriter {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
            schemas: vec![],
        }
    }

    pub fn add_points(&mut self, points: &[KeyPoint]) {
        let data: Vec<f32> = points
            .iter()
            .map(|kpt| vec![kpt.x(), kpt.y()])
            .flatten()
            .collect();
        self.schemas.push(Schema {
            parts_type: PartsType::point.as_str(),
            render_mode: RenderMode::POINTS.as_str(),
            datas: vec![Data {
                variable_name: "aPos".to_string(),
                data,
            }],
        });
    }

    pub fn add_lines<T: Distance + Clone>(&mut self, matches: &[Match<T>]) {
        let data: Vec<f32> = matches
            .iter()
            .map(|m| {
                vec![
                    m.matche.0.kpt.x(),
                    m.matche.0.kpt.y(),
                    m.matche.1.kpt.x(),
                    m.matche.1.kpt.y(),
                ]
            })
            .flatten()
            .collect();
        self.schemas.push(Schema {
            parts_type: PartsType::line.as_str(),
            render_mode: RenderMode::LINES.as_str(),
            datas: vec![Data {
                variable_name: "aPos".to_string(),
                data,
            }],
        });
    }

    pub fn flush(&self) {
        let json_strs: Vec<String> = self
            .schemas
            .iter()
            .map(|schema| serde_json::to_string_pretty(&serde_json::json!(schema)).unwrap())
            .collect();
        println!("[\n{}\n]", json_strs.join(",\n"));
    }
}
