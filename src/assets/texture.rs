use std::{io::Cursor, sync::Arc};
use vulkano::{
    device::Queue,
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage, MipmapsCount},
};

pub struct Texture {
    pub image: Arc<ImageView<Arc<ImmutableImage>>>,
}

impl Texture {
    pub fn new(queue: Arc<Queue>, bytes: Vec<u8>) -> Arc<Self> {
        let cursor = Cursor::new(bytes);
        let decoder = png::Decoder::new(cursor);
        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();
        let dimensions = ImageDimensions::Dim2d {
            width: info.width,
            height: info.height,
            array_layers: 1,
        };
        let mut image_data = Vec::new();

        image_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut image_data).unwrap();

        let (image, _) = ImmutableImage::from_iter(
            image_data.iter().cloned(),
            dimensions,
            MipmapsCount::One,
            Format::R8G8B8A8_SRGB,
            queue.clone(),
        )
        .unwrap();
        let image = ImageView::new(image).unwrap();

        Arc::new(Self { image })
    }
}
