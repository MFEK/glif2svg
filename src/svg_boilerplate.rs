use phf::phf_ordered_map as map;
use phf::OrderedMap as Phf;

pub static XYGRID_IDENT: &'static str = "inkscape:grid";
pub static XYGRID: Phf<&'static str, &'static str> = map! {
    "id" => "grid№1",
    "type" => "xygrid",
    "dotted" => "false",
    "enabled" => "true",
    "visible" => "true",
    "empspacing" => "10"
};

pub static NAMEDVIEW_IDENT: &'static str = "sodipodi:namedview";
pub static NAMEDVIEW: Phf<&'static str, &'static str> = map! {
    "pagecolor" => "#ffffff",
    "bordercolor" => "#666666",
    "borderopacity" => "1.0",
    "showgrid" => "true",
};

pub static XMLNS: Phf<&'static str, &'static str> = map! {
    "" => "http://www.w3.org/2000/svg",
    "svg" => "http://www.w3.org/2000/svg",
    "sodipodi" => "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd",
    "inkscape" => "http://www.inkscape.org/namespaces/inkscape",
};
