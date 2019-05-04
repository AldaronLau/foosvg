pub use footile::Rgba8;

use footile::{PathBuilder,Plotter,Raster};
use usvg;
use usvg::svgdom::WriteBuffer;
use usvg::svgdom::{ AttributeId, AttributeValue, Document, ElementId, FilterSvg, PathSegment };

/// Render an SVG onto a pixel buffer.
pub fn render(svg: &str, width: u32, height: u32, pixels: Vec<Rgba8>) -> Vec<Rgba8> {
    // Simplify SVG with usvg.
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
    let svg = tree.to_svgdom().with_write_opt(&usvg::svgdom::WriteOptions::default()).to_string();
//    println!("SVG: {}", svg);

    // Make Raster
    let mut p = Plotter::new(width, height);
    let mut r = Raster::with_pixels(p.width(), p.height(), pixels);

    // Render
    let doc = Document::from_str(&svg).unwrap();
    for (id, node) in doc.root().descendants().svg() {
        match id {
            ElementId::Path => {
                let mut pathbuilder = PathBuilder::new();
                let mut old_x = 0.0f32;
                let mut old_y = 0.0f32;

                let attrs = node.attributes();
                if let Some(&AttributeValue::Path(ref path)) = attrs.get_value(AttributeId::D) {
                    for seg in path.iter() {
                        println!("{:?}", seg);
                        match seg {
                            PathSegment::MoveTo { abs, x, y } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.move_to(*x as f32, *y as f32);
                                old_x = *x as f32;
                                old_y = *y as f32;
                            }
                            PathSegment::LineTo { abs, x, y } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.line_to(*x as f32, *y as f32);
                                old_x = *x as f32;
                                old_y = *y as f32;
                            }
                            PathSegment::HorizontalLineTo { abs, x } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.line_to(*x as f32, old_y);
                                old_x = *x as f32;
                            }
                            PathSegment::VerticalLineTo { abs, y } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.line_to(old_x, *y as f32);
                                old_y = *y as f32;
                            }
                            PathSegment::Quadratic { abs, x1, y1, x, y } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.quad_to(*x1 as f32, *y1 as f32, *x as f32, *y as f32);
                            }
                            PathSegment::CurveTo { abs, x1, y1, x2, y2, x, y } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.cubic_to(*x1 as f32, *y1 as f32, *x2 as f32, *y2 as f32, *x as f32, *y as f32); // TODO: verify order.
                            }
                            PathSegment::ClosePath { abs } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.close();
                            }
                            a => {
                                println!("WARNING: Path Unknown {:?}", a);
                            },
                        }
                    }
                }

                let path = pathbuilder.build();
                r.over(p.fill(&path, footile::FillRule::NonZero), Rgba8::rgb(208, 255, 208));
            }
            a => {
                println!("WARNING: Element Unknown {}", a);
            }
        }
    }

    // Return pixels
    r.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
