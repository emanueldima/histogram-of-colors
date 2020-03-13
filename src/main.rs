use std::fs::File;
use std::io::{BufWriter, Write};
use std::{env, fs, path};

fn main() {
    let rootpath_arg = env::args()
        .take(2)
        .skip(1)
        .next()
        .expect("First argument is expected to be a folder with images");
    let rootpath = path::PathBuf::from(rootpath_arg).canonicalize().unwrap();

    let mut image_files = vec![];
    {
        let mut dirstack = vec![rootpath];
        while let Some(path) = dirstack.pop() {
            let files_iterator = fs::read_dir(path).unwrap();
            for direntry in files_iterator.map(|r| r.unwrap()) {
                let path = direntry.path();
                let md = direntry.metadata().unwrap();
                if md.is_dir() {
                    dirstack.push(path);
                } else if direntry.file_name().to_string_lossy().ends_with(".jpeg") {
                    image_files.push(path);
                }
            }
        }
    }

    let mut histogram = Histogram::new();
    for (i, image_path) in image_files.iter().enumerate() {
        println!(
            "{}/{}: {}",
            i,
            image_files.len(),
            image_path.to_string_lossy()
        );
        if let Ok(img) = image::open(image_path) {
            for p in img.into_rgb().pixels() {
                histogram.add(p.0[0], p.0[1], p.0[2]);
            }
        }
    }

    let filename = "colorhistogram.html";
    histogram.save_histo_html(filename);
    webbrowser::open(filename).unwrap();
}

struct Histogram {
    pub counts: Vec<usize>,
    pub pixels: usize,
}

const REDUCTION: usize = 8;
const UNIT: usize = 256 / REDUCTION;

impl Histogram {
    pub fn new() -> Histogram {
        let mut v = Vec::with_capacity(UNIT * UNIT * UNIT);
        for _ in 0..v.capacity() {
            v.push(0);
        }
        Histogram {
            counts: v,
            pixels: 0,
        }
    }

    #[inline(always)]
    fn add(&mut self, r: u8, g: u8, b: u8) {
        let pos = r as usize / REDUCTION * UNIT * UNIT
            + g as usize / REDUCTION * UNIT
            + b as usize / REDUCTION;
        self.counts[pos] += 1;
        self.pixels += 1;
    }

    fn histo(&self) -> Vec<(usize, u8, u8, u8)> {
        let mut v: Vec<(usize, u8, u8, u8)> = Vec::with_capacity(UNIT * UNIT * UNIT);
        for i in 0..UNIT {
            for j in 0..UNIT {
                for k in 0..UNIT {
                    let pos = i * UNIT * UNIT + j * UNIT + k;
                    v.push((
                        self.counts[pos],
                        (i * REDUCTION) as u8,
                        (j * REDUCTION) as u8,
                        (k * REDUCTION) as u8,
                    ));
                }
            }
        }
        v.sort_by(|x, y| y.0.cmp(&x.0));
        v
    }

    fn save_histo_html(&self, file_name: &str) {
        let h = self.histo();
        let write_file = File::create(file_name).unwrap();
        let mut writer = BufWriter::new(&write_file);
        write!(&mut writer, "{}", Self::HEADER).unwrap();
        write!(
            &mut writer,
            "<div style='margin:20px'>total: {} pixels</div>",
            human(self.pixels as u64, "")
        )
        .unwrap();
        for x in h.iter() {
            write!(&mut writer,
                   r#"<div style="margin:2px">
                   <div style="padding-left: 200px; background-color:rgb({},{},{}); display:inline">
                   </div>
                   <span style="margin:10px; width:400px; height:50px;">pixels: {}; color: ({}, {}, {}); </span>
                   </div>"#,
                   x.1, x.2, x.3,
                   human(x.0 as u64, ""),
                   x.1, x.2, x.3,
            ).unwrap();
        }
        write!(&mut writer, "{}", Self::FOOTER).unwrap();
    }

    const HEADER: &'static str = r#"
<!DOCTYPE html>
<html>
<head>
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
</head>
<body>
    "#;

    const FOOTER: &'static str = r#"
</body>
</html>
    "#;
}

pub fn human(s: u64, suffix: &str) -> String {
    const K: u64 = 1000;
    if s > K * K * K {
        format!("{:.2}G{}", s as f64 / (K * K * K) as f64, suffix)
    } else if s > K * K {
        format!("{:.2}M{}", s as f64 / (K * K) as f64, suffix)
    } else if s > K {
        format!("{:.2}K{}", s as f64 / (K) as f64, suffix)
    } else {
        format!("{}{}", s, suffix)
    }
}
