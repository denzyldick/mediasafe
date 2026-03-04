use crate::database::{Database, Face};
use ndarray::Array4;
use ort::{session::builder::GraphOptimizationLevel, session::Session};
use std::fs;
use std::path::Path;
use tauri::AppHandle;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use std::collections::HashMap;

pub struct MlContext {
    pub tx: std::sync::Mutex<UnboundedSender<String>>,
    pub pending_count: Arc<AtomicUsize>,
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

pub fn start_background_worker(app: &AppHandle, config_path: String) -> (UnboundedSender<String>, Arc<AtomicUsize>) {
    let (tx, mut rx) = unbounded_channel::<String>();
    let pending_count = Arc::new(AtomicUsize::new(0));
    let pending_count_clone = Arc::clone(&pending_count);
    let app_handle = app.clone();
    let db_path = config_path.clone();

    std::thread::spawn(move || {
        println!("Background ML worker started.");
        
        macro_rules! emit_progress {
            () => {
                let count = pending_count_clone.load(Ordering::SeqCst);
                let _ = app_handle.emit("indexing-progress", count);
            };
        }

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
        let tokenizer = tokenizers::Tokenizer::from_file(&clip_tokenizer_path).ok();

        // Initialize ONNX Sessions
        let _ = ort::init().with_name("mediasafe").commit();

        let mut clip_visual: Option<Session> = None;
        let mut clip_text: Option<Session> = None;
        let mut face_detector: Option<Session> = None;

        macro_rules! load_models {
            () => {
                clip_visual = match Session::builder() {
                    Ok(b) => b
                        .with_optimization_level(GraphOptimizationLevel::Level1)
                        .unwrap()
                        .with_intra_threads(1)
                        .unwrap()
                        .commit_from_file(&clip_visual_path)
                        .ok(),
                    Err(_) => None,
                };

                clip_text = match Session::builder() {
                    Ok(b) => b
                        .with_optimization_level(GraphOptimizationLevel::Level1)
                        .unwrap()
                        .with_intra_threads(1)
                        .unwrap()
                        .commit_from_file(&clip_text_path)
                        .ok(),
                    Err(_) => None,
                };

                face_detector = match Session::builder() {
                    Ok(b) => b
                        .with_optimization_level(GraphOptimizationLevel::Level1)
                        .unwrap()
                        .with_intra_threads(1)
                        .unwrap()
                        .commit_from_file(&ultraface_path)
                        .ok(),
                    Err(_) => None,
                };
            }
        }

        load_models!();

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
            if let Some(ref tok) = tokenizer {
                for text_label in &search_vocabulary {
                    if let Ok(encoding) = tok.encode(format!("a photo of {}", text_label), true) {
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
        }

        // Process queue
        while let Some(photo_id) = rx.blocking_recv() {
            if photo_id == "__RELOAD_MODELS__" {
                load_models!();
                continue;
            }
            if photo_id == "__STATUS__" {
                emit_progress!();
                continue;
            }

            println!("ML Worker processing photo: {}", photo_id);

            // Query DB to find the photo location
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

            if !photo_loc.is_empty() && Path::new(&photo_loc).exists() {
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
                        for (x, y, pixel) in resized.enumerate_pixels() {
                            input_img[[0, 0, y as usize, x as usize]] = (pixel[0] as f32 / 255.0 - 0.48145466) / 0.26862954;
                            input_img[[0, 1, y as usize, x as usize]] = (pixel[1] as f32 / 255.0 - 0.45782750) / 0.26130258;
                            input_img[[0, 2, y as usize, x as usize]] = (pixel[2] as f32 / 255.0 - 0.40821073) / 0.27577711;
                        }

                        if let Ok(img_tensor) = ort::value::Value::from_array(input_img) {
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
                                        let _ = db.connection.execute(
                                            "INSERT INTO object (photo_id, class, probability) VALUES(?1, ?2, ?3)",
                                            (&photo_id, class_name, &score.to_string()),
                                        );
                                    }
                                }
                            }
                        }
                    }
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
                        for (x, y, pixel) in resized.enumerate_pixels() {
                            input[[0, 0, y as usize, x as usize]] = (pixel[0] as f32 - 127.0) / 128.0;
                            input[[0, 1, y as usize, x as usize]] = (pixel[1] as f32 - 127.0) / 128.0;
                            input[[0, 2, y as usize, x as usize]] = (pixel[2] as f32 - 127.0) / 128.0;
                        }

                        if let Ok(input_tensor) = ort::value::Value::from_array(input) {
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
                                        if xmax > xmin && ymax > ymin {
                                            let w = xmax - xmin;
                                            let h = ymax - ymin;
                                            if w > 20 && h > 20 {
                                                let face_crop = image::imageops::crop_imm(&img, xmin, ymin, w, h).to_image();
                                                let face_id = format!("{}_face_{}_{}", photo_id, xmin, ymin);
                                                let crop_path = format!("{}/{}.jpg", faces_dir, face_id);
                                                if face_crop.save(&crop_path).is_ok() {
                                                    use base64::{engine::general_purpose, Engine as _};
                                                    use std::io::Cursor;
                                                    let mut buffer = Cursor::new(Vec::new());
                                                    let _ = face_crop.write_to(&mut buffer, image::ImageOutputFormat::Jpeg(80));
                                                    let encoded = format!("data:image/jpeg;base64,{}", general_purpose::STANDARD.encode(buffer.get_ref()));

                                                    let db_face = Face {
                                                        photo_id: photo_id.clone(),
                                                        face_id,
                                                        crop_path,
                                                        encoded,
                                                        person_id: None,
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
            }

            // Decrement pending count and emit progress
            let current = pending_count_clone.fetch_sub(1, Ordering::SeqCst);
            let remaining = current.saturating_sub(1);
            let _ = app_handle.emit("indexing-progress", remaining);
        }
    });

    (tx, pending_count)
}
