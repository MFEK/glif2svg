///! glif2svg in Rust
///! (c) 2021–2022 Fredrick R. Brennan and MFEK authors. See LICENSE.

#[macro_use] extern crate derivative; // for better #[derive(…)]

mod svg_boilerplate;
use svg_boilerplate::*;

use glifparser;
use glifparser::IntegerOrFloat;
use glifparser::outline::skia::SkiaPointTransforms;
use glifparser::outline::skia::ToSkiaPaths as _;
use clap::{self, App, AppSettings, Arg};
use skia_safe::{Point, path::Verb};
use skia_safe::path::Iter as SkIter;
use mfek_ipc::{self, IPCInfo};
use xmltree;

use std::{fs, str as stdstr};

type XmlTreeAttribute = (String, String);

#[derive(Debug, Derivative)]
#[derivative(Default(new="true"))]
struct SVGPathPen {
    path: String,
    minx: f64,
    maxx: f64,
    miny: f64,
    maxy: f64,
    #[derivative(Default(value="4"))]
    precision: u8,
    no_viewbox: bool
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
    fn extend_path(&mut self, path: &str) {
        self.path.push_str(path);
    }

    #[allow(non_snake_case)]
    fn viewBox(&self) -> (f64, f64, f64, f64) {
        return (self.minx, self.miny, self.minx.abs() + self.maxx, self.miny.abs() + self.maxy)
    }

    fn width(&self) -> f64 {
        self.viewBox().2
    }

    fn height(&self) -> f64 {
        self.viewBox().3
    }

    fn p(&self, size: impl Into<IntegerOrFloat>) -> IntegerOrFloat {
        let f: f32 = f32::from(size.into());
        let precision = (10.0_f32).powf(self.precision as f32);
        IntegerOrFloat::from(f32::trunc(f * precision) / precision)
    }

    fn size_attr_impl(&self, name: &'static str, size: impl Into<IntegerOrFloat>) -> XmlTreeAttribute {
        let name = name.to_string();
        let size = format!("{}px", self.p(size.into()));
        (name, size.into())
    }

    fn px_size_attrs(&self) -> [XmlTreeAttribute; 2] {
        [
            self.size_attr_impl("width", self.width()),
            self.size_attr_impl("height", self.height()),
        ]
    }

    #[allow(non_snake_case)]
    fn viewBox_str(&self) -> String {
        let (x, y, dx, dy) = self.viewBox();
        format!("{} {} {} {}", self.p(x), self.p(y), self.p(dx), self.p(dy))
    }

    fn transform_x(&self, x: f32) -> f32 {
        x
    }

    #[allow(non_snake_case)]
    fn transform_y_viewBox(&self, y: f32) -> f32 {
        (-y) + self.maxy as f32 + self.miny as f32
    }

    fn transform_y_wh(&self, y: f32) -> f32 {
        (-y) + self.miny as f32
    }

    fn transform_y(&self, y: f32) -> f32 {
        if self.no_viewbox {
            self.transform_y_wh(y)
        } else {
            self.transform_y_viewBox(y)
        }
    }

    fn move_to(&mut self, pt: Point) {
        consider_min_max(self, &[pt]);
        self.extend_path(&format!("M {} {}", self.p(pt.x), self.p(pt.y)));
    }

    fn line_to(&mut self, pt: Point) {
        consider_min_max(self, &[pt]);
        self.extend_path(&format!("L {} {}", self.p(pt.x), self.p(pt.y)));
    }

    fn curve_to(&mut self, pt: &[Point]) {
        consider_min_max(self, pt);
        self.extend_path(&format!("C {} {} {} {} {} {}", self.p(pt[1].x), self.p(pt[1].y), self.p(pt[2].x), self.p(pt[2].y), self.p(pt[3].x), self.p(pt[3].y)));
    }

    fn qcurve_to(&mut self, pt: &[Point]) {
        consider_min_max(self, pt);
        self.extend_path(&format!("Q {} {} {} {}", self.p(pt[1].x), self.p(pt[1].y), self.p(pt[2].x), self.p(pt[2].y)));
    }

    fn close_path(&mut self) {
        self.extend_path("Z");
    }

    fn apply_outline(&mut self, outline: &glifparser::Outline<()>) {
        let skia_paths = outline.to_skia_paths(Some(SkiaPointTransforms { calc_x: &|x|self.transform_x(x), calc_y: &|y|self.transform_y(y) }));
        for path in skia_paths.open.iter().chain(skia_paths.closed.iter()) {
            let iter = SkIter::new(&path, false);
            for (verb, pts) in iter {
                match verb {
                    Verb::Move => self.move_to(pts[0]),
                    Verb::Line => self.line_to(pts[1]),
                    Verb::Quad => self.qcurve_to(&pts),
                    Verb::Cubic => self.curve_to(&pts),
                    Verb::Close => self.close_path(),
                    _ => {unimplemented!()}
                }
            }
        }
    }
}

fn main() {
    let matches = App::new("glif2svg")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .version("0.99.0")
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
        .arg(Arg::with_name("no_viewbox")
            .short("B")
            .long("no-viewbox")
            .help("Don't put viewBox in SVG"))
        .arg(Arg::with_name("no_metrics")
            .short("M")
            .long("no-metrics")
            .help("Don't consider glif's height/width when writing SVG, use minx/maxx/miny/maxy"))
        .arg(Arg::with_name("fontinfo")
            .short("F")
            .long("fontinfo")
            .takes_value(true)
            .help("fontinfo file (for metrics, should point to fontinfo.plist path)\n\n"))
        .arg(Arg::with_name("precision")
            .short("p")
            .long("precision")
            .takes_value(true)
            .default_value("16")
            .validator(|f|Ok(f.parse::<u8>().map(|_|()).map_err(|_|String::from("Precision must be 0…255"))?))
            .help("Float precision"))
        .get_matches();

    let input = matches.value_of("input").unwrap_or_else(||matches.value_of("input_file").unwrap());
    let output = matches.value_of("output").or_else(||matches.value_of("output_file"));
    let no_viewbox = matches.is_present("no_viewbox");
    let no_metrics = matches.is_present("no_metrics");
    let fontinfo_o = matches.value_of("fontinfo");

    let glif: glifparser::Glif<()> = glifparser::glif::read_from_filename(matches.value_of("input").unwrap()).unwrap();

    let mut svg = SVGPathPen::new();
    svg.precision = matches.value_of("precision").unwrap().parse::<u8>().unwrap();
    svg.no_viewbox = no_viewbox;

    if let (Ok(..), true) = (mfek_ipc::module::available("metadata".into(), "0.0.2-beta1"), !no_metrics) {
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
            if let Some(ref o) = glif.outline.as_ref() {
                svg.apply_outline(o);
            }
        }
    } else {
        eprintln!("MFEKmetadata REQUIRED for sane UFO metrics into SVG");
    }

    if !no_metrics {
        svg.minx = 0.;
        svg.maxx = glif.width.unwrap_or(0) as f64;
    }

    let mut svgxml = xmltree::Element::new("svg");
    let mut namespace = xmltree::Namespace::empty();
    for (k, v) in XMLNS.into_iter() {
        namespace.put(*k, *v);
    }
    svgxml.namespaces = Some(namespace);
    svgxml.attributes.insert("version".to_owned(), "1.1".to_owned());
    if no_viewbox {
        for (k, v) in svg.px_size_attrs() {
            svgxml.attributes.insert(k, v);
        }
    } else {
        svgxml.attributes.insert("viewBox".to_owned(), svg.viewBox_str());
    }

    let mut sodipodixml = xmltree::Element::new(NAMEDVIEW_IDENT);
    sodipodixml.attributes = NAMEDVIEW.into_iter().map(|(k, v)|((*k).to_owned(), (*v).to_owned())).collect();
    let mut xygridxml = xmltree::Element::new(XYGRID_IDENT);
    xygridxml.attributes = XYGRID.into_iter().map(|(k, v)|((*k).to_owned(), (*v).to_owned())).collect();
    let mut guidexml = xmltree::Element::new("sodipodi:guide");
    guidexml.attributes.insert("id".to_owned(), "baseline".to_owned());
    guidexml.attributes.insert("position".to_owned(), format!("{:.2},{:.2}", 0.0, svg.miny.abs()));
    guidexml.attributes.insert("orientation".to_owned(), "0.00,1.00".to_owned());
    sodipodixml.children = vec![xmltree::XMLNode::Element(xygridxml), xmltree::XMLNode::Element(guidexml)];
    svgxml.children.push(xmltree::XMLNode::Element(sodipodixml));

    if let Some(ref o) = glif.outline.as_ref() {
        svg.apply_outline(o);
    }

    let mut gxml = xmltree::Element::new("g");
    gxml.attributes.insert("id".to_owned(), "glyph".to_owned());
    let mut pathxml = xmltree::Element::new("path");
    pathxml.attributes.insert("d".to_owned(), svg.path);
    gxml.children = vec![xmltree::XMLNode::Element(pathxml)];
    svgxml.children.push(xmltree::XMLNode::Element(gxml));

    let config = xmltree::EmitterConfig::new().perform_indent(true).indent_string("    ");
    let mut outxml = Vec::<u8>::new();

    svgxml.write_with_config(&mut outxml, config).unwrap();

    outxml.push('\n' as u8);

    if let Some(outfile) = output {
        if outfile != "-" {
            fs::write(outfile, &outxml).unwrap();
            return
        }   
    }
    println!("{}", stdstr::from_utf8(&outxml).unwrap());
}
