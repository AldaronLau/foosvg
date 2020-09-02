pub use pix::{Raster, rgb::{SRgba8, Rgba8p}, ops::SrcOver, el::Pixel};

use footile::{PathBuilder,Plotter};
use usvg;
use usvg::svgdom::{WriteBuffer, AttributeId, AttributeValue, Document, ElementId, FilterSvg, PathSegment };

/// Render an SVG onto a pixel buffer.  Returns width, height and pixels.
pub fn render(svg: &str) -> Raster<SRgba8> {
    // Simplify SVG with usvg.
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
    let svg = tree.to_svgdom().with_write_opt(&usvg::svgdom::WriteOptions::default()).to_string();
    println!("SVG: {}", svg);

    // Render
    let doc = Document::from_str(&svg).unwrap();
    let mut iter = doc.root().descendants().svg();

    let (width, height) = if let Some((id, node)) = iter.next() {
        if id == ElementId::Svg {
            let attrs = node.attributes();
            let width;
            let height;

            println!("{:?}", attrs);

            if let Some(&AttributeValue::Length(ref v)) = attrs.get_value(AttributeId::Width) {
                width = v.num as u32;
            } else {
                panic!("Width unspecified!");
            }
            if let Some(&AttributeValue::Length(ref v)) = attrs.get_value(AttributeId::Height) {
                height = v.num as u32;
            } else {
                panic!("Height unspecified!");
            }

            (width, height)
        } else {
            panic!("Not an SVG!");
        }
    } else {
        panic!("SVG is an empty file!");
    };

    // Make Raster
    let mut p = Plotter::new(Raster::<Rgba8p>::with_clear(width, height));

    for (id, node) in iter {
        match id {
            ElementId::Path => {
                let mut pathbuilder = PathBuilder::default();
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

                if let Some(&AttributeValue::Color(ref c)) = attrs.get_value(AttributeId::Fill) {
                    p.fill(footile::FillRule::NonZero, &path, SRgba8::new(c.red, c.green, c.blue, 255).convert());
                }

                if let Some(&AttributeValue::Color(ref c)) = attrs.get_value(AttributeId::Stroke) {
                    p.stroke(&path, SRgba8::new(c.red, c.green, c.blue, 255).convert());
                }
                // END PATH
            }
            a => {
                println!("WARNING: Element Unknown {}", a);
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
