use crate::error::Error;
use std::{
    io::{Cursor, Read},
    sync::Arc,
};
use vulkano::{
    device::Queue,
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage, MipmapsCount},
    sampler::Sampler,
};

pub struct Texture {
    pub image: Arc<ImageView<Arc<ImmutableImage>>>,
    pub sampler: Arc<Sampler>,
}

impl Texture {
    pub fn from_png<R>(
        mut reader: R,
        queue: Arc<Queue>,
        sampler: Arc<Sampler>,
        format: Format,
    ) -> Result<Arc<Self>, Error>
    where
        R: Read,
    {
        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes)?;

        let cursor = Cursor::new(bytes);
        let decoder = png::Decoder::new(cursor);
        let mut reader = decoder.read_info()?;
        let info = reader.info();
        let dimensions = ImageDimensions::Dim2d {
            width: info.width,
            height: info.height,
            array_layers: 1,
        };
        let mut image_data = Vec::new();

        image_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut image_data)?;

        let (image, _) = ImmutableImage::from_iter(
            image_data.iter().cloned(),
            dimensions,
            MipmapsCount::Log2,
            format,
            queue,
        )?;
        let image = ImageView::new(image)?;

        Ok(Arc::new(Self { image, sampler }))
    }
}
