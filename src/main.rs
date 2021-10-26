use glifparser;
use clap::{self, App, AppSettings, Arg};
use skia_safe::{Point, path::Verb};
use skia_safe::path::Iter as SkIter;
use mfek_ipc::{self, Available, IPCInfo};
use xmltree;

use std::{fs, str as stdstr};

struct SVGPathPen {
    path: String,
    minx: f64,
    maxx: f64,
    miny: f64,
    maxy: f64
}

fn consider_min_max(svg: &mut SVGPathPen, points: &[Point]) {
    for p in points {
        if (svg.minx as f32) > p.x { svg.minx = p.x.into(); }
        if (svg.maxx as f32) < p.x { svg.maxx = p.x.into(); }
        if (svg.miny as f32) > p.y { svg.miny = p.y.into(); }
        if (svg.maxy as f32) < p.y { svg.maxy = p.y.into(); }
    }
}

impl SVGPathPen {
    fn new() -> Self {
        Self { path: String::new(), minx: 0., miny: 0., maxx: 0., maxy: 0. }
    }

    fn extend_path(&mut self, path: &str) {
        self.path.push_str(path);
    }

    #[allow(non_snake_case)]
    fn viewBox(&self) -> (f64, f64, f64, f64) {
        return (self.minx, self.miny, self.maxx+(self.minx).abs(), self.maxy+(self.miny).abs())
    }

    #[allow(non_snake_case)]
    fn viewBox_str(&self) -> String {
        let (x, y, dx, dy) = self.viewBox();
        return format!("{:.0} {:.0} {:.0} {:.0}", x, y, dx, dy);
    }

    fn transform(&self) -> String {
        return format!("translate(0 {:.0})", (-self.maxy).abs() - (self.miny).abs());
    }

    fn move_to(&mut self, pt: Point) {
        consider_min_max(self, &[pt]);
        self.extend_path(&format!("M {} {}", pt.x, -pt.y));
    }

    fn line_to(&mut self, pt: Point) {
        consider_min_max(self, &[pt]);
        self.extend_path(&format!("L {} {}", pt.x, -pt.y));
    }

    fn curve_to(&mut self, pt: &[Point]) {
        consider_min_max(self, pt);
        self.extend_path(&format!("C {} {} {} {} {} {}", pt[1].x, -pt[1].y, pt[2].x, -pt[2].y, pt[3].x, -pt[3].y));
    }

    fn qcurve_to(&mut self, pt: &[Point]) {
        consider_min_max(self, pt);
        self.extend_path(&format!("Q {} {} {} {}", pt[1].x, -pt[1].y, pt[2].x, -pt[2].y));
    }

    fn close_path(&mut self) {
        self.extend_path("Z");
    }
}

fn main() {
    let matches = App::new("glif2svg")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("0.0.0")
        .author("Fredrick R. Brennan <copypasteⒶkittens⊙ph>; MFEK Authors")
        .about("Convert between glif to SVG")
        .arg(Arg::with_name("input_file")
            .short("in")
            .long("input")
            .takes_value(true)
            .conflicts_with("input")
            .hidden(true))
        .arg(Arg::with_name("input")
            .index(1)
            .help("The path to the input file.")
            .conflicts_with("input_file")
            .required_unless("input_file"))
        .arg(Arg::with_name("output_file")
            .short("out")
            .long("output")
            .takes_value(true)
            .conflicts_with("output")
            .display_order(1)
            .help("The path to the output file. If not provided, or `-`, stdout.\n\n\n"))
        .arg(Arg::with_name("output")
            .index(2)
            .hidden(true))
        .arg(Arg::with_name("no_transform")
            .short("T")
            .long("no-transform")
            .help("Don't put transform=\"translate(…)\" in SVG"))
        .arg(Arg::with_name("no_metrics")
            .short("M")
            .long("no-metrics")
            .help("Don't consider glif's height/width when writing SVG, use minx/maxx/miny/maxy"))
        .arg(Arg::with_name("fontinfo")
            .short("F")
            .long("fontinfo")
            .takes_value(true)
            .help("fontinfo file (for metrics, should point to fontinfo.plist path)\n\n"))
        .get_matches();

    let input = matches.value_of("input").unwrap_or_else(||matches.value_of("input_file").unwrap());
    let output = matches.value_of("output").or_else(||matches.value_of("output_file"));
    let no_transform = matches.is_present("no_transform");
    let no_metrics = matches.is_present("no_metrics");
    let fontinfo_o = matches.value_of("fontinfo");

    let glif: glifparser::Glif<()> = glifparser::glif::read_from_filename(matches.value_of("input").unwrap()).unwrap();

    let mut svg = SVGPathPen::new();

    use glifparser::outline::skia::ToSkiaPaths;
    if let Some(o) = glif.outline {
        let skia_paths = o.to_skia_paths(None);
        for path in skia_paths.open.iter().chain(skia_paths.closed.iter()) {
            let iter = SkIter::new(&path, false);
            for (verb, pts) in iter {
                match verb {
                    Verb::Move => svg.move_to(pts[0]),
                    Verb::Line => svg.line_to(pts[0]),
                    Verb::Quad => svg.qcurve_to(&pts),
                    Verb::Cubic => svg.curve_to(&pts),
                    Verb::Close => svg.close_path(),
                    _ => {unimplemented!()}
                }
            }
        }
    }

    if !no_metrics {
        svg.minx = 0.;
        svg.maxx = glif.width.unwrap_or(0) as f64;
    }

    let (status, _) = mfek_ipc::module_available("metadata".into());
    if status == Available::Yes && !no_metrics {
        let ipc_info = if let Some(fi) = fontinfo_o {
            IPCInfo::from_fontinfo_path("glif2svg".to_string(), &fi)
        } else {
            IPCInfo::from_glif_path("glif2svg".to_string(), &input)
        };

        if let Ok((ascender, descender)) = mfek_ipc::helpers::metadata::ascender_descender(&ipc_info) {
            svg.maxy = ascender as f64;
            svg.miny = descender as f64;
        } else {
            eprintln!("Failed to set metrics of SVG from glif font!");
        }
    } else {
        eprintln!("MFEKmetadata REQUIRED for sane UFO metrics into SVG");
    }

    let mut svgxml = xmltree::Element::new("svg");
    svgxml.attributes.insert("version".to_owned(), "1.1".to_owned());
    svgxml.attributes.insert("xmlns".to_owned(), "http://www.w3.org/2000/svg".to_owned());
    svgxml.attributes.insert("viewBox".to_owned(), svg.viewBox_str());

    let mut pathxml = xmltree::Element::new("path");
    if !no_transform {
        pathxml.attributes.insert("transform".to_owned(), if !no_metrics {
            format!("translate(0 {})", svg.miny.abs())
        } else {
            svg.transform()
        });
    }
    pathxml.attributes.insert("d".to_owned(), svg.path);
    svgxml.children.push(xmltree::XMLNode::Element(pathxml));

    let config = xmltree::EmitterConfig::new().perform_indent(true);
    let mut outxml = Vec::<u8>::new();

    svgxml.write_with_config(&mut outxml, config).unwrap();

    if let Some(outfile) = output {
        if outfile != "-" {
            fs::write(outfile, &outxml).unwrap();
            return
        }   
    }
    print!("{}", stdstr::from_utf8(&outxml).unwrap());
}
