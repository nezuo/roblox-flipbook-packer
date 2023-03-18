use std::path::PathBuf;

use image::{
    imageops::{self, FilterType},
    ImageBuffer, Rgba,
};

pub fn sequence_to_frames(image_paths: Vec<PathBuf>) -> Vec<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut frames = Vec::new();

    let frame_count = image_paths.len();

    let frame_size = if frame_count > 16 {
        128.0
    } else if frame_count > 4 {
        256.0
    } else {
        512.0
    };

    for image_path in image_paths {
        let mut image = image::open(image_path).unwrap().into_rgba8();

        let smaller = image.width().min(image.height()) as f64;
        let higher = image.width().max(image.height()) as f64;

        let (frame_width, frame_height) = if image.width() > image.height() {
            (frame_size as u32, (frame_size * smaller / higher) as u32)
        } else {
            ((frame_size * smaller / higher) as u32, frame_size as u32)
        };

        image = imageops::resize(&image, frame_width, frame_height, FilterType::CatmullRom);

        frames.push(image);
    }

    frames
}

pub fn image_to_frames(
    rows: u32,
    columns: u32,
    image_path: PathBuf,
) -> Vec<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let frame_count = rows * columns;

    let frame_size = if frame_count > 16 {
        128.0
    } else if frame_count > 4 {
        256.0
    } else {
        512.0
    };

    let mut image = image::open(image_path).unwrap().into_rgba8();

    let old_frame_width = (image.width() / columns) as f64;
    let old_frame_height = (image.height() / rows) as f64;

    let (frame_width, frame_height) = if old_frame_width > old_frame_height {
        (
            frame_size as u32,
            (frame_size * old_frame_height / old_frame_width) as u32,
        )
    } else {
        (
            (frame_size * old_frame_width / old_frame_height) as u32,
            frame_size as u32,
        )
    };

    image = imageops::resize(
        &image,
        columns * frame_width,
        rows * frame_height,
        FilterType::CatmullRom,
    );

    let mut frames = Vec::new();

    for frame in 0..frame_count {
        let row = frame / columns;
        let column = frame % columns;

        let sub_image = imageops::crop_imm(
            &image,
            column * frame_width,
            row * frame_height,
            frame_width,
            frame_height,
        );

        frames.push(sub_image.to_image());
    }

    frames
}

pub fn export_packed(frames: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>) {
    let frame_count = frames.len();

    let (rows_columns, frame_size): (u32, u32) = if frame_count > 16 {
        (8, 128)
    } else if frame_count > 4 {
        (4, 256)
    } else {
        (2, 512)
    };

    let mut final_image = ImageBuffer::new(1024, 1024);

    for (frame, image) in frames.iter().enumerate() {
        let row = frame as u32 / rows_columns;
        let column = frame as u32 % rows_columns;

        let x = column * frame_size + (frame_size - image.width()) / 2;
        let y = row * frame_size + (frame_size - image.height()) / 2;

        imageops::overlay(&mut final_image, image, x.into(), y.into())
    }

    let path = rfd::FileDialog::new()
        .add_filter("PNG", &["png"])
        .save_file();

    final_image.save(path.unwrap()).unwrap();
}

pub fn export_sequence(rows: u32, columns: u32, image_path: PathBuf) {
    let temp = image_path.clone();
    let file_name = temp
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .trim_matches('"');
    let original_image = image::open(image_path).unwrap().into_rgba8();

    let frame_width = original_image.width() / columns;
    let frame_height = original_image.height() / rows;

    let mut images = Vec::new();

    for frame in 0..rows * columns {
        let row = frame / columns;
        let column = frame % columns;

        let sub_image = imageops::crop_imm(
            &original_image,
            column * frame_width,
            row * frame_height,
            frame_width,
            frame_height,
        );

        images.push(sub_image.to_image());
    }

    let path = rfd::FileDialog::new().pick_folder().unwrap();

    for (i, image) in images.iter().enumerate() {
        image
            .save_with_format(
                path.join(format!("{} ({}).png", file_name, i + 1)),
                image::ImageFormat::Png,
            )
            .unwrap();
    }
}
