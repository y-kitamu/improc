use anyhow::{Context, Result};
use nalgebra as na;
use std::{
    fs::{self, File},
    io::prelude::*,
    path::Path,
};

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
    partsType: &'static str,
    renderMode: &'static str,
    datas: Vec<Data>,
}

#[derive(Serialize)]
struct Data {
    variableName: String,
    data: Vec<f32>,
}

impl ViewerWriter {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
            schemas: vec![],
        }
    }

    pub fn add_points(&mut self, points: &[KeyPoint], color: &na::Vector3<f32>) {
        let data: Vec<f32> = points
            .iter()
            .map(|kpt| vec![kpt.x(), kpt.y(), 0.0])
            .flatten()
            .collect();
        self.schemas.push(Schema {
            partsType: PartsType::point.as_str(),
            renderMode: RenderMode::POINTS.as_str(),
            datas: vec![
                Data {
                    variableName: "aPos".to_string(),
                    data,
                },
                Data {
                    variableName: "aColor".to_string(),
                    data: (0..(points.len()))
                        .map(|_| vec![color.x, color.y, color.z])
                        .flatten()
                        .collect(),
                },
            ],
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
            partsType: PartsType::line.as_str(),
            renderMode: RenderMode::LINES.as_str(),
            datas: vec![Data {
                variableName: "aPos".to_string(),
                data,
            }],
        });
    }

    pub fn flush(&self) -> Result<String> {
        let json_strs: Vec<String> = self
            .schemas
            .iter()
            .map(|schema| serde_json::to_string_pretty(&serde_json::json!(schema)).unwrap())
            .collect();
        let output_str = format!("[\n{}\n]", json_strs.join(",\n"));
        let mut file = File::create(&self.filename)?;
        {
            let outdir = Path::new(&self.filename)
                .parent()
                .context("Failed to get parent path")?;
            fs::create_dir_all(outdir)?;
        }
        file.write_all(output_str.as_bytes())?;
        Ok(output_str)
    }
}
