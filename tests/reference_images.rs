//! Compares the decoding results with reference renderings.

use crc32fast;
use glob;
use image_bmp;

use std::fs;
use std::io;
use std::path::PathBuf;
use std::u32;

use crc32fast::Hasher as Crc32;

const BASE_PATH: [&str; 2] = [".", "tests"];
const IMAGE_DIR: &str = "images";
const REFERENCE_DIR: &str = "reference";

fn process_images<F>(dir: &str, input_decoder: Option<&str>, func: F)
where
    F: Fn(&PathBuf, PathBuf, &str),
{
    let base: PathBuf = BASE_PATH.iter().collect();
    let decoders = &["bmp"];
    for decoder in decoders {
        let mut path = base.clone();
        path.push(dir);
        path.push(decoder);
        path.push("**");
        path.push(
            "*.".to_string()
                + match input_decoder {
                    Some(val) => val,
                    None => decoder,
                },
        );
        let pattern = &*format!("{}", path.display());
        for path in glob::glob(pattern).unwrap().filter_map(Result::ok) {
            func(&base, path, decoder)
        }
    }
}

/// Describes a single test case of `check_references`.
struct ReferenceTestCase {
    orig_filename: String,
    crc: u32,
    kind: ReferenceTestKind,
}

enum ReferenceTestKind {
    /// The test image is loaded using `image::open`, and the result is compared
    /// against the reference image.
    SingleImage,
}

impl std::str::FromStr for ReferenceTestCase {
    type Err = &'static str;

    /// Construct `ReferenceTestCase` from the file name of a reference
    /// image.
    fn from_str(filename: &str) -> Result<Self, Self::Err> {
        let mut filename_parts = filename.rsplitn(3, '.');

        // Ignore the file extension
        filename_parts.next().unwrap();

        // The penultimate part of `filename_parts` represents the metadata,
        // describing the test type and other details.
        let meta_str = filename_parts.next().ok_or("missing metadata part")?;
        let meta = meta_str.split('_').collect::<Vec<_>>();
        let (crc, kind);

        if meta.len() == 1 {
            // `CRC`
            crc = parse_crc(&meta[0]).ok_or("malformed CRC")?;
            kind = ReferenceTestKind::SingleImage;
        } else {
            return Err("unrecognized reference image metadata format");
        }

        // The remaining part represents the original file name
        let orig_filename = filename_parts
            .next()
            .ok_or("missing original file name")?
            .to_owned();

        Ok(Self {
            orig_filename,
            crc,
            kind,
        })
    }
}

/// Parse the given string as a hexadecimal CRC hash, used by `check_references`.
fn parse_crc(src: &str) -> Option<u32> {
    u32::from_str_radix(src, 16).ok()
}

#[test]
fn check_references() {
    process_images(REFERENCE_DIR, Some("png"), |base, path, decoder| {
        println!("check_references {}", path.display());

        let f = io::BufReader::new(fs::File::open(&path).unwrap());
        let ref_img = match image_bmp::BMPDecoder::new(f) {
            // TODO: This was to_rgba()
            Ok(mut img) => img.read_image_data().unwrap(),
            // Do not fail on unsupported error
            // This might happen because the testsuite contains unsupported images
            // or because a specific decoder included via a feature.
            Err(image_bmp::ImageError::UnsupportedError(_)) => return,
            Err(err) => panic!(format!("{:?}", err)),
        };

        let (filename, testsuite) = {
            let mut path: Vec<_> = path.components().collect();
            (path.pop().unwrap(), path.pop().unwrap())
        };

        // Parse the file name to obtain the test case information
        let filename_str = filename.as_os_str().to_str().unwrap();
        let case: ReferenceTestCase = filename_str.parse().unwrap();

        let mut img_path = base.clone();
        img_path.push(IMAGE_DIR);
        img_path.push(decoder);
        img_path.push(testsuite.as_os_str());
        img_path.push(case.orig_filename);

        // Load the test image
        let test_img;

        match case.kind {
            ReferenceTestKind::SingleImage => {
                // Read the input file as a single image
                let f = io::BufReader::new(fs::File::open(&path).unwrap());
                match image_bmp::BMPDecoder::new(f) {
                    // TODO: This was .to_rgba()
                    Ok(mut img) => test_img = img.read_image_data().unwrap(),
                    // Do not fail on unsupported error
                    // This might happen because the testsuite contains unsupported images
                    // or because a specific decoder included via a feature.
                    Err(image_bmp::ImageError::UnsupportedError(_)) => return,
                    Err(err) => {
                        panic!(format!("decoding of {:?} failed with: {:?}", img_path, err))
                    }
                };
            }
        }

        let test_crc_actual = {
            let mut hasher = Crc32::new();
            hasher.update(&*test_img);
            hasher.finalize()
        };

        if test_crc_actual != case.crc {
            panic!(
                "The decoded image's hash does not match (expected = {:08x}, actual = {:08x}).",
                case.crc, test_crc_actual
            );
        }

        if *ref_img == *test_img {
            panic!("Reference rendering does not match.");
        }
    })
}

/// Check that BMP files with large values could cause OOM issues are rejected.
///
/// The images are postfixed with `bad_bmp` to not be loaded by the other test.
#[test]
fn bad_bmps() {
    let path: PathBuf = BASE_PATH
        .iter()
        .collect::<PathBuf>()
        .join(IMAGE_DIR)
        .join("bmp/images")
        .join("*.bad_bmp");

    let pattern = &*format!("{}", path.display());
    for path in glob::glob(pattern).unwrap().filter_map(Result::ok) {
        let f = io::BufReader::new(fs::File::open(&path).unwrap());
        let im = image_bmp::BMPDecoder::new(f);
        assert!(im.is_err());
    }
}
