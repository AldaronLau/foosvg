pub use pix::{
    el::Pixel,
    ops::SrcOver,
    rgb::{Rgba8p, SRgba8},
    Raster,
};

use footile::{PathOp, Plotter};
use pointy::Pt;
use usvg::{Options, XmlOptions, Tree, PathSegment, NodeKind, Path};

/// Render an SVG onto a pixel buffer.  Returns width, height and pixels.
pub fn render(svg: &str) -> Raster<SRgba8> {
    // Simplify SVG with usvg.
    let options = Options::default();
    let tree = Tree::from_str(svg, &options.to_ref()).unwrap();
    let svg = tree.to_string(&XmlOptions::default());
    println!("SVG: {}", svg);

    // Render
    let mut iter = tree.root().descendants();

    let (width, height) = if let Some(node) = iter.next() {
        if let NodeKind::Svg(svg) = &*node.borrow() {
            (svg.size.width() as u32, svg.size.height() as u32)
        } else {
            panic!("Not an SVG!");
        }
    } else {
        panic!("SVG is an empty file!");
    };

    // Make Raster
    let mut p = Plotter::new(Raster::<Rgba8p>::with_clear(width, height));

    for node in iter {
        match &*node.borrow() {
            NodeKind::Path(Path {
                id: _id,
                transform: _transform,
                visibility: _visibility,
                fill,
                stroke,
                rendering_mode: _shape_rendering,
                text_bbox: _text_bbox,
                data: path_data,
            }) => {
                let mut pathbuilder = Vec::new();
                let mut old_x = 0.0f32;
                let mut old_y = 0.0f32;
                
                if let Some(stroke) = stroke {
                    pathbuilder.push(PathOp::PenWidth(stroke.width.value() as f32));
                }

                for seg in path_data.0.iter() {
                    println!("{:?}", seg);
                    match seg {
                        PathSegment::MoveTo { x, y } => {
                            old_x = *x as f32;
                            old_y = *y as f32;
                            pathbuilder.push(PathOp::Move(Pt::new(old_x, old_y)));
                        }
                        PathSegment::LineTo { x, y } => {
                            old_x = *x as f32;
                            old_y = *y as f32;
                            pathbuilder.push(PathOp::Line(Pt::new(old_x, old_y)));
                        }
                        PathSegment::CurveTo {
                            x1,
                            y1,
                            x2,
                            y2,
                            x,
                            y,
                        } => {
                            let (x1, y1, x2, y2) = {
                                old_x = *x as f32;
                                old_y = *y as f32;
                                (*x1 as f32, *y1 as f32, *x2 as f32, *y2 as f32)
                            };

                            // TODO: verify order.
                            pathbuilder.push(PathOp::Cubic(
                                Pt::new(x1, y1),
                                Pt::new(x2, y2),
                                Pt::new(old_x, old_y),
                            ));
                        }
                        PathSegment::ClosePath => {
                            pathbuilder.push(PathOp::Close());
                        }
                    }
                }

                let path = pathbuilder.as_slice();

                if let Some(fill) = fill {
                    let fill_rule = match fill.rule {
                        usvg::FillRule::NonZero => footile::FillRule::NonZero,
                        usvg::FillRule::EvenOdd => footile::FillRule::EvenOdd,
                    };
                    let fill_alpha = fill.opacity.to_u8();
                    let fill_color = match fill.paint {
                        usvg::Paint::Color(usvg::Color { red, green, blue }) => SRgba8::new(red, green, blue, fill_alpha),
                        usvg::Paint::Link(_) => SRgba8::new(0, 0, 0, fill_alpha),
                    };
                    p.fill(fill_rule, path, fill_color.convert());
                }

                if let Some(stroke) = stroke {
                    let stroke_alpha = stroke.opacity.to_u8();
                    let stroke_color = match stroke.paint {
                        usvg::Paint::Color(usvg::Color { red, green, blue }) => SRgba8::new(red, green, blue, stroke_alpha),
                        usvg::Paint::Link(_) => SRgba8::new(0, 0, 0, stroke_alpha),
                    };
                    let stroke_miter_limit = stroke.miterlimit.value();
                    let stroke_line_join = match stroke.linejoin {
                        usvg::LineJoin::Miter => footile::JoinStyle::Miter(stroke_miter_limit as f32),
                        usvg::LineJoin::Bevel => footile::JoinStyle::Bevel,
                        usvg::LineJoin::Round => footile::JoinStyle::Round,
                    };
                    p.set_join(stroke_line_join);
                    p.stroke(path, stroke_color.convert());
                }

                // END PATH
            }
            a => {
                println!("WARNING: Element Unknown {a:?}");
            }
        }
    }

    // Return pixels
    Raster::with_raster(&p.raster())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
