use crate::database::{Database, Face};
use ndarray::Array4;
use ort::{session::builder::GraphOptimizationLevel, session::Session};
use std::fs;
use std::path::Path;
use tauri::AppHandle;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

pub struct MlContext {
    pub tx: std::sync::Mutex<UnboundedSender<String>>,
}

async fn download_file(
    url: &str,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if path.exists() {
        return Ok(());
    }
    println!("Downloading {} to {:?}", url, path);
    let bytes = reqwest::get(url).await?.bytes().await?;
    fs::write(path, bytes)?;
    Ok(())
}

pub fn start_background_worker(_app: &AppHandle, config_path: String) -> UnboundedSender<String> {
    let (tx, mut rx) = unbounded_channel::<String>();
    let db_path = config_path.clone();

    std::thread::spawn(move || {
        println!("Background ML worker started.");

        // Ensure models directory exists
        let models_dir = format!("{}/models", db_path);
        let faces_dir = format!("{}/faces", db_path);
        let _ = fs::create_dir_all(&models_dir);
        let _ = fs::create_dir_all(&faces_dir);

        let clip_visual_path = Path::new(&models_dir).join("clip-vit-base-patch32-visual.onnx");
        let clip_text_path = Path::new(&models_dir).join("clip-vit-base-patch32-text.onnx");
        let clip_tokenizer_path = Path::new(&models_dir).join("tokenizer.json");
        let ultraface_path = Path::new(&models_dir).join("version-RFB-320.onnx");



        let db = Database::new(&db_path);

        // Load Tokenizer
        let tokenizer = tokenizers::Tokenizer::from_file(&clip_tokenizer_path).unwrap();

        // Initialize ONNX Sessions
        let _ = ort::init().with_name("mediasafe").commit();

        let mut clip_visual = match Session::builder() {
            Ok(b) => match b
                .with_optimization_level(GraphOptimizationLevel::Level1)
                .unwrap()
                .with_intra_threads(1)
                .unwrap()
                .commit_from_file(&clip_visual_path)
            {
                Ok(session) => Some(session),
                Err(e) => {
                    println!("ERROR loading clip_visual: {:?}", e);
                    None
                }
            },
            Err(_) => None,
        };

        if clip_visual.is_some() {
            println!("CLIP Visual Model loaded successfully.");
        } else {
            println!("Failed to load CLIP Visual ONNX model.");
        }

        let mut clip_text = match Session::builder() {
            Ok(b) => match b
                .with_optimization_level(GraphOptimizationLevel::Level1)
                .unwrap()
                .with_intra_threads(1)
                .unwrap()
                .commit_from_file(&clip_text_path)
            {
                Ok(session) => Some(session),
                Err(e) => {
                    println!("ERROR loading clip_text: {:?}", e);
                    None
                }
            },
            Err(_) => None,
        };

        if clip_text.is_some() {
            println!("CLIP Text Model loaded successfully.");
        } else {
            println!("Failed to load CLIP Text ONNX model.");
        }

        // Define our custom zero-shot vocabulary pool
        let search_vocabulary = vec![
            "a passport",
            "a driver's license",
            "an id card",
            "a document",
            "a receipt",
            "a screenshot",
            "a meme",
            "a text message",
            "a cat",
            "a dog",
            "a pet",
            "an animal",
            "a car",
            "a vehicle",
            "a motorcycle",
            "a bicycle",
            "a person",
            "a selfie",
            "a group of people",
            "a crowd",
            "a building",
            "a house",
            "architecture",
            "a city",
            "a landscape",
            "nature",
            "a mountain",
            "a beach",
            "water",
            "food",
            "a meal",
            "a drink",
            "coffee",
            "a laptop",
            "a computer",
            "a phone",
            "a screen",
            "electronics",
            "a piece of furniture",
            "a room interior",
            "a sunset",
            "the sky",
            "clouds",
            "art",
            "a drawing",
            "a painting",
        ];

        // Pre-compute text embeddings once
        let mut precomputed_text_embeddings: Vec<(&str, Vec<f32>)> = Vec::new();
        if let Some(ref mut text_model) = clip_text {
            for text_label in &search_vocabulary {
                if let Ok(encoding) = tokenizer.encode(format!("a photo of {}", text_label), true) {
                    let input_ids: Vec<i64> =
                        encoding.get_ids().iter().map(|&x| x as i64).collect();
                    let attention_mask: Vec<i64> = encoding
                        .get_attention_mask()
                        .iter()
                        .map(|&x| x as i64)
                        .collect();

                    if let (Ok(input_ids_arr), Ok(attention_mask_arr)) = (
                        ndarray::Array2::from_shape_vec((1, input_ids.len()), input_ids),
                        ndarray::Array2::from_shape_vec((1, attention_mask.len()), attention_mask),
                    ) {
                        if let Ok(id_tensor) = ort::value::Value::from_array(input_ids_arr) {
                            match text_model.run(ort::inputs![
                                "input_ids" => &id_tensor
                            ]) {
                                Ok(outputs) => {
                                    match outputs[0].try_extract_tensor::<f32>() {
                                        Ok((_shape, text_emb_tensor)) => {
                                            let mut text_embedding = vec![0.0; 512];
                                            text_embedding.copy_from_slice(text_emb_tensor);

                                            // Normalize text embedding
                                            let text_norm: f32 = text_embedding
                                                .iter()
                                                .map(|x| x * x)
                                                .sum::<f32>()
                                                .sqrt();
                                            for v in text_embedding.iter_mut() {
                                                *v /= text_norm;
                                            }

                                            precomputed_text_embeddings
                                                .push((text_label, text_embedding));
                                        }
                                        Err(e) => {
                                            println!("text_model tensor extraction failed: {:?}", e)
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("text_model run failed for '{}': {:?}", text_label, e)
                                }
                            }
                        }
                    }
                }
            }
        }
        println!(
            "Pre-computed {} CLIP Text Embeddings.",
            precomputed_text_embeddings.len()
        );

        let mut face_detector = match Session::builder() {
            Ok(b) => b
                .with_optimization_level(GraphOptimizationLevel::Level1)
                .unwrap()
                .with_intra_threads(1)
                .unwrap()
                .commit_from_file(&ultraface_path)
                .ok(),
            Err(_) => None,
        };

        if face_detector.is_some() {
            println!("UltraFace loaded successfully.");
        } else {
            println!("Failed to load UltraFace ONNX model.");
        }

        // Process queue
        while let Some(photo_id) = rx.blocking_recv() {
            println!("ML Worker processing photo: {}", photo_id);

            // Query DB to find the photo location
            // Since we don't have a `get_photo_by_id` returning the location easily, we can just use `list_photos`?
            // Let's add a small raw query
            let mut photo_loc = String::new();
            if let Ok(mut stmt) = db
                .connection
                .prepare("SELECT location FROM photo WHERE id = ?1")
            {
                if let Ok(mut rows) = stmt.query([&photo_id]) {
                    if let Ok(Some(row)) = rows.next() {
                        photo_loc = row.get(0).unwrap_or_default();
                    }
                }
            }

            if photo_loc.is_empty() || !Path::new(&photo_loc).exists() {
                continue;
            }

            if let (Some(ref mut visual_model), Some(ref mut text_model)) =
                (&mut clip_visual, &mut clip_text)
            {
                if let Ok(img) = image::open(&photo_loc) {
                    let img = img.to_rgb8();
                    let resized = image::imageops::resize(
                        &img,
                        224,
                        224,
                        image::imageops::FilterType::Triangle,
                    );

                    let mut input_img = Array4::<f32>::zeros((1, 3, 224, 224));
                    for y in 0..224 {
                        for x in 0..224 {
                            let pixel = resized.get_pixel(x as u32, y as u32);
                            // CLIP ImageNet Normalization
                            let r = (pixel[0] as f32 / 255.0 - 0.48145466) / 0.26862954;
                            let g = (pixel[1] as f32 / 255.0 - 0.45782750) / 0.26130258;
                            let b = (pixel[2] as f32 / 255.0 - 0.40821073) / 0.27577711;
                            input_img[[0, 0, y, x]] = r;
                            input_img[[0, 1, y, x]] = g;
                            input_img[[0, 2, y, x]] = b;
                        }
                    }

                    let img_tensor = ort::value::Value::from_array(input_img).unwrap();
                    if let Ok(outputs) =
                        visual_model.run(ort::inputs!["pixel_values" => &img_tensor])
                    {
                        if let Ok((_shape, img_emb_tensor)) = outputs[0].try_extract_tensor::<f32>()
                        {
                            let mut img_embedding = vec![0.0; 512];
                            img_embedding.copy_from_slice(img_emb_tensor);

                            // Normalize image embedding
                            let img_norm: f32 =
                                img_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                            for v in img_embedding.iter_mut() {
                                *v /= img_norm;
                            }

                            let mut similarities = Vec::new();

                            for (text_label, text_embedding) in &precomputed_text_embeddings {
                                // Cosine Similarity
                                let dot_product: f32 = img_embedding
                                    .iter()
                                    .zip(text_embedding.iter())
                                    .map(|(a, b)| a * b)
                                    .sum();
                                similarities.push((*text_label, dot_product));
                            }

                            // Sort descending
                            similarities.sort_by(|a, b| {
                                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                            });

                            for (class_name, score) in similarities.iter().take(5) {
                                println!(
                                    "{} classified as: {} (score: {})",
                                    photo_id, class_name, score
                                );
                                let _ = db.connection.execute(
                                    "INSERT INTO object (photo_id, class, probability) VALUES(?1, ?2, ?3)",
                                    (&photo_id, class_name, &score.to_string()),
                                );
                            }
                        } else {
                            println!("Tensor shape extraction failed.");
                        }
                    } else {
                        println!("Visual model run failed.");
                    }
                } else {
                    println!("Failed to open image for visual model.");
                }
            } else {
                println!("Clip visual or text models are missing!");
            }

            if let Some(ref mut fmodel) = face_detector {
                if let Ok(img) = image::open(&photo_loc) {
                    let img = img.to_rgb8();
                    let original_w = img.width() as f32;
                    let original_h = img.height() as f32;
                    let resized = image::imageops::resize(
                        &img,
                        320,
                        240,
                        image::imageops::FilterType::Triangle,
                    );

                    let mut input = Array4::<f32>::zeros((1, 3, 240, 320));
                    for y in 0..240 {
                        for x in 0..320 {
                            let pixel = resized.get_pixel(x as u32, y as u32);
                            input[[0, 0, y, x]] = (pixel[0] as f32 - 127.0) / 128.0;
                            input[[0, 1, y, x]] = (pixel[1] as f32 - 127.0) / 128.0;
                            input[[0, 2, y, x]] = (pixel[2] as f32 - 127.0) / 128.0;
                        }
                    }

                    let input_tensor = ort::value::Value::from_array(input).unwrap();
                    if let Ok(outputs) = fmodel.run(ort::inputs![&input_tensor]) {
                        let mut scores_opt = None;
                        let mut boxes_opt = None;
                        for i in 0..outputs.len() {
                            if let Ok((shape, tensor)) = outputs[i].try_extract_tensor::<f32>() {
                                if shape.len() == 3 && shape[2] == 2 {
                                    scores_opt = Some(tensor);
                                } else if shape.len() == 3 && shape[2] == 4 {
                                    boxes_opt = Some(tensor);
                                }
                            }
                        }

                        if let (Some(scores), Some(boxes)) = (scores_opt, boxes_opt) {
                            let anchors = crate::face_detector::generate_anchors();
                            let mut proposals = Vec::new();
                            let num_anchors = anchors.len();
                            for i in 0..num_anchors {
                                let score = scores[i * 2 + 1]; // class 1 is face
                                if score > 0.6 {
                                    let loc = [
                                        boxes[i * 4 + 0],
                                        boxes[i * 4 + 1],
                                        boxes[i * 4 + 2],
                                        boxes[i * 4 + 3],
                                    ];
                                    let decoded = crate::face_detector::decode(&loc, &anchors[i]);
                                    proposals.push((decoded, score));
                                }
                            }
                            let keep = crate::face_detector::nms(&mut proposals, 0.3);
                            for &idx in &keep {
                                let bbox = proposals[idx].0;
                                let xmin = (bbox[0] * original_w).max(0.0) as u32;
                                let ymin = (bbox[1] * original_h).max(0.0) as u32;
                                let xmax = (bbox[2] * original_w).min(original_w) as u32;
                                let ymax = (bbox[3] * original_h).min(original_h) as u32;
                                // Make sure width/height are at least 1, and don't panic if xmin>=xmax (due to float precision)
                                if xmax > xmin && ymax > ymin {
                                    let w = xmax - xmin;
                                    let h = ymax - ymin;

                                    // Ignore tiny faces
                                    if w > 20 && h > 20 {
                                        let face_crop =
                                            image::imageops::crop_imm(&img, xmin, ymin, w, h)
                                                .to_image();
                                        let face_id =
                                            format!("{}_face_{}_{}", photo_id, xmin, ymin);
                                        let crop_path = format!("{}/{}.jpg", faces_dir, face_id);
                                        if face_crop.save(&crop_path).is_ok() {
                                            let db_face = Face {
                                                photo_id: photo_id.clone(),
                                                face_id,
                                                crop_path,
                                            };
                                            db.store_face(db_face);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    tx
}
