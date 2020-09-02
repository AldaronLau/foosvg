use foosvg::render;
use png_pong::{PngRaster, Encoder};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 2);
    let svg = std::fs::read_to_string(&args[1]).unwrap();
    let raster = PngRaster::Rgba8(render(&svg));

    let fl = std::fs::File::create(&format!("{}.png", args[1])).expect("Failed to create image");
    let mut bw = std::io::BufWriter::new(fl);
    let mut encoder = Encoder::new(&mut bw).into_step_enc();
    let step = png_pong::Step{ raster, delay: 0 };
    encoder.encode(&step).expect("Failed to add frame");
}
