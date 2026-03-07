use crate::database::{Database, Face};
use base64::Engine;
use ndarray::Array4;
use ort::{session::builder::GraphOptimizationLevel, session::Session};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tauri::Emitter;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Semaphore;

pub struct MlContext {
    pub tx: std::sync::Mutex<UnboundedSender<String>>,
    pub pending_count: Arc<AtomicUsize>,
}

fn compute_text_embeddings(
    text_model: &mut Session,
    tokenizer: &tokenizers::Tokenizer,
) -> Vec<(String, Vec<f32>)> {
    let search_vocabulary = vec![
        "a passport", "a driver's license", "an id card", "a document", "a receipt",
        "a screenshot", "a meme", "a text message", "a cat", "a dog", "a pet",
        "an animal", "a car", "a vehicle", "a motorcycle", "a bicycle", "a person",
        "a selfie", "a group of people", "a crowd", "a building", "a house",
        "architecture", "a city", "a landscape", "nature", "a mountain", "a beach",
        "water", "food", "a meal", "a drink", "coffee", "a laptop", "a computer",
        "a phone", "a screen", "electronics", "a piece of furniture", "a room interior",
        "a sunset", "the sky", "clouds", "art", "a drawing", "a painting",
    ];

    let mut embeddings = Vec::new();
    for text_label in search_vocabulary {
        if let Ok(encoding) = tokenizer.encode(format!("a photo of {text_label}"), true) {
            let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
            if let Ok(input_ids_arr) = ndarray::Array2::from_shape_vec((1, input_ids.len()), input_ids) {
                if let Ok(id_tensor) = ort::value::Value::from_array(input_ids_arr) {
                    if let Ok(outputs) = text_model.run(ort::inputs!["input_ids" => &id_tensor]) {
                        if let Ok((_shape, text_emb_tensor)) = outputs[0].try_extract_tensor::<f32>() {
                            let mut text_embedding = vec![0.0; 512];
                            text_embedding.copy_from_slice(text_emb_tensor);
                            let text_norm: f32 = text_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                            for v in text_embedding.iter_mut() { *v /= text_norm; }
                            embeddings.push((text_label.to_string(), text_embedding));
                        }
                    }
                }
            }
        }
    }
    embeddings
}

pub fn start_background_worker(
    app: &AppHandle,
    config_path: String,
) -> (UnboundedSender<String>, Arc<AtomicUsize>) {
    let (tx, mut rx) = unbounded_channel::<String>();
    let pending_count = Arc::new(AtomicUsize::new(0));
    let pending_count_clone = Arc::clone(&pending_count);
    let app_handle = app.clone();
    let db_path = config_path.clone();

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(2));

        let models_dir = format!("{db_path}/models");
        let faces_dir = format!("{db_path}/faces");
        let _ = fs::create_dir_all(&models_dir);
        let _ = fs::create_dir_all(&faces_dir);

        let clip_visual_path = Path::new(&models_dir).join("clip-vit-base-patch32-visual.onnx");
        let clip_text_path = Path::new(&models_dir).join("clip-vit-base-patch32-text.onnx");
        let clip_tokenizer_path = Path::new(&models_dir).join("tokenizer.json");
        let ultraface_path = Path::new(&models_dir).join("version-RFB-320.onnx");

        let mut clip_visual: Option<Arc<Mutex<Session>>> = None;
        let mut face_detector: Option<Arc<Mutex<Session>>> = None;
        let mut tokenizer: Option<Arc<tokenizers::Tokenizer>> = None;
        let mut text_embeddings: Arc<Vec<(String, Vec<f32>)>> = Arc::new(Vec::new());
        let known_people: Arc<Mutex<Vec<(String, Vec<f32>)>>> = Arc::new(Mutex::new(Vec::new()));
        let mut ort_initialized = false;

        let db = Arc::new(Mutex::new(Database::new(&db_path)));
        let config = db.lock().unwrap().get_state();
        let num_threads: usize = config.get("scan_threads")
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| { if cfg!(any(target_os = "android", target_os = "ios")) { 2 } else { 4 } });
        
        println!("ML Worker: Initializing with {} threads", num_threads);
        let pool = rayon::ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap();
        let memory_semaphore = Arc::new(Semaphore::new(num_threads * 2));

        while let Some(photo_id) = rx.blocking_recv() {
            if photo_id == "__STATUS__" {
                let count = pending_count_clone.load(Ordering::SeqCst);
                let _ = app_handle.emit("indexing-progress", count);
                continue;
            }

            if !ort_initialized || photo_id == "__RELOAD_MODELS__" {
                println!("ML Worker: Loading AI models...");
                if !ort_initialized {
                    let _ = ort::init().with_name("siegu").commit();
                    ort_initialized = true;
                }

                let is_ok = |p: &Path| p.exists() && p.metadata().map(|m| m.len()).unwrap_or(0) > 1024 * 1024;
                
                tokenizer = tokenizers::Tokenizer::from_file(&clip_tokenizer_path).ok().map(Arc::new);
                clip_visual = if is_ok(&clip_visual_path) {
                    Session::builder().unwrap()
                        .with_optimization_level(GraphOptimizationLevel::Level1).unwrap()
                        .with_intra_threads(1).unwrap()
                        .commit_from_file(&clip_visual_path).ok().map(|s| Arc::new(Mutex::new(s)))
                } else { None };

                if let (Ok(mut text_session), Some(ref tok)) = (Session::builder().unwrap()
                    .with_optimization_level(GraphOptimizationLevel::Level1).unwrap()
                    .with_intra_threads(1).unwrap()
                    .commit_from_file(&clip_text_path), &tokenizer) 
                {
                    text_embeddings = Arc::new(compute_text_embeddings(&mut text_session, tok));
                }

                face_detector = if is_ok(&ultraface_path) {
                    Session::builder().unwrap()
                        .with_optimization_level(GraphOptimizationLevel::Level1).unwrap()
                        .with_intra_threads(1).unwrap()
                        .commit_from_file(&ultraface_path).ok().map(|s| Arc::new(Mutex::new(s)))
                } else { None };

                let people_vec = db.lock().unwrap().get_all_people_with_embeddings();
                if let Ok(mut lock) = known_people.lock() {
                    *lock = people_vec;
                }

                println!("ML Worker: Models ready.");
                if photo_id == "__RELOAD_MODELS__" { continue; }
            }

            let photo_id_task = photo_id.clone();
            let app_handle_task = app_handle.clone();
            let pending_count_task = Arc::clone(&pending_count_clone);
            let clip_visual_task = clip_visual.clone();
            let face_detector_task = face_detector.clone();
            let text_embeddings_task = text_embeddings.clone();
            let known_people_task = known_people.clone();
            let faces_dir_task = faces_dir.clone();
            let db_task = Arc::clone(&db);
            let sem_task = Arc::clone(&memory_semaphore);

            pool.spawn(move || {
                let _permit = sem_task.try_acquire(); 
                let mut provided_frames = Vec::new();
                let mut actual_id = photo_id_task.clone();
                
                if photo_id_task.starts_with("__VIDEO_FRAMES__:") {
                    let parts: Vec<&str> = photo_id_task.split("|||").collect();
                    if parts.len() > 1 {
                        actual_id = parts[0].replace("__VIDEO_FRAMES__:", "").to_string();
                        for b64_raw in parts.iter().skip(1) {
                            let b64 = b64_raw.replace("data:image/jpeg;base64,", "").replace("data:image/png;base64,", "");
                            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&b64) {
                                if let Ok(img) = image::load_from_memory(&bytes) { provided_frames.push(img.to_rgb8()); }
                            }
                        }
                    }
                }

                let mode = {
                    let lock = db_task.lock().unwrap();
                    let state = lock.get_state();
                    state.get("indexing_mode").map(|s| s.as_str().to_string()).unwrap_or("immediate".to_string())
                };

                if mode == "manual" {
                    let current = pending_count_task.fetch_sub(1, Ordering::SeqCst);
                    let _ = app_handle_task.emit("indexing-progress", current.saturating_sub(1));
                    return;
                }

                let mut photo_loc = String::new();
                {
                    let lock = db_task.lock().unwrap();
                    if let Ok(row) = lock.connection.query_row("SELECT location FROM photo WHERE id = ?1", [&actual_id], |r| r.get::<_, String>(0)) {
                        photo_loc = row;
                    }
                }

                if !photo_loc.is_empty() && Path::new(&photo_loc).exists() {
                    let frames = if !provided_frames.is_empty() {
                        provided_frames
                    } else {
                        image::open(&photo_loc).map(|img| vec![img.to_rgb8()]).unwrap_or_default()
                    };

                    for img in frames {
                        if let Some(ref visual_model_lock) = clip_visual_task {
                            let resized = image::imageops::resize(&img, 224, 224, image::imageops::FilterType::Triangle);
                            let mut input_img = Array4::<f32>::zeros((1, 3, 224, 224));
                            for (x, y, pixel) in resized.enumerate_pixels() {
                                input_img[[0, 0, y as usize, x as usize]] = (pixel[0] as f32 / 255.0 - 0.48145466) / 0.26862954;
                                input_img[[0, 1, y as usize, x as usize]] = (pixel[1] as f32 / 255.0 - 0.4578275) / 0.2613026;
                                input_img[[0, 2, y as usize, x as usize]] = (pixel[2] as f32 / 255.0 - 0.40821073) / 0.2757771;
                            }
                            if let Ok(img_tensor) = ort::value::Value::from_array(input_img) {
                                if let Ok(mut visual_model) = visual_model_lock.lock() {
                                    if let Ok(outputs) = visual_model.run(ort::inputs!["pixel_values" => &img_tensor]) {
                                        if let Ok((_shape, img_emb_tensor)) = outputs[0].try_extract_tensor::<f32>() {
                                            let mut img_embedding = vec![0.0; 512];
                                            img_embedding.copy_from_slice(img_emb_tensor);
                                            let img_norm: f32 = img_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                                            if img_norm > 0.0 { for v in img_embedding.iter_mut() { *v /= img_norm; } }
                                            
                                            let mut similarities = Vec::new();
                                            for (text_label, text_embedding) in text_embeddings_task.iter() {
                                                let dot_product: f32 = img_embedding.iter().zip(text_embedding.iter()).map(|(a, b)| a * b).sum();
                                                similarities.push((text_label, dot_product));
                                            }
                                            similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                                            
                                            let lock = db_task.lock().unwrap();
                                            for (class_name, score) in similarities.iter().take(5) {
                                                let _ = lock.connection.execute("INSERT INTO object (photo_id, class, probability) VALUES(?1, ?2, ?3)", (&actual_id, class_name, &score.to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(ref fmodel_lock) = face_detector_task {
                            let (orig_w, orig_h) = (img.width() as f32, img.height() as f32);
                            let resized = image::imageops::resize(&img, 320, 240, image::imageops::FilterType::Triangle);
                            let mut input = Array4::<f32>::zeros((1, 3, 240, 320));
                            for (x, y, pixel) in resized.enumerate_pixels() {
                                input[[0, 0, y as usize, x as usize]] = (pixel[0] as f32 - 127.0) / 128.0;
                                input[[0, 1, y as usize, x as usize]] = (pixel[1] as f32 - 127.0) / 128.0;
                                input[[0, 2, y as usize, x as usize]] = (pixel[2] as f32 - 127.0) / 128.0;
                            }
                            if let Ok(input_tensor) = ort::value::Value::from_array(input) {
                                if let Ok(mut fmodel) = fmodel_lock.lock() {
                                    if let Ok(outputs) = fmodel.run(ort::inputs![&input_tensor]) {
                                        let mut scores_opt = None;
                                        let mut boxes_opt = None;
                                        for i in 0..outputs.len() {
                                            if let Ok((shape, tensor)) = outputs[i].try_extract_tensor::<f32>() {
                                                if shape.len() == 3 && shape[2] == 2 { scores_opt = Some(tensor); }
                                                else if shape.len() == 3 && shape[2] == 4 { boxes_opt = Some(tensor); }
                                            }
                                        }
                                        if let (Some(scores), Some(boxes)) = (scores_opt, boxes_opt) {
                                            let anchors = crate::face_detector::generate_anchors();
                                            let mut proposals = Vec::new();
                                            for i in 0..anchors.len() {
                                                let score = scores[i * 2 + 1];
                                                if score > 0.6 {
                                                    let loc = [boxes[i * 4], boxes[i * 4 + 1], boxes[i * 4 + 2], boxes[i * 4 + 3]];
                                                    let decoded = crate::face_detector::decode(&loc, &anchors[i]);
                                                    proposals.push((decoded, score));
                                                }
                                            }
                                            let keep = crate::face_detector::nms(&mut proposals, 0.3);
                                            for &idx in &keep {
                                                let bbox = proposals[idx].0;
                                                let xmin = (bbox[0] * orig_w).max(0.0) as u32;
                                                let ymin = (bbox[1] * orig_h).max(0.0) as u32;
                                                let xmax = (bbox[2] * orig_w).min(orig_w) as u32;
                                                let ymax = (bbox[3] * orig_h).min(orig_h) as u32;
                                                if xmax > xmin && ymax > ymin {
                                                    let (w, h) = (xmax - xmin, ymax - ymin);
                                                    if w > 20 && h > 20 {
                                                        let face_crop = image::imageops::crop_imm(&img, xmin, ymin, w, h).to_image();
                                                        let face_id = format!("{actual_id}_face_{xmin}_{ymin}");
                                                        let crop_path = format!("{faces_dir_task}/{face_id}.jpg");
                                                        if face_crop.save(&crop_path).is_ok() {
                                                            let mut face_embedding = Vec::new();
                                                            if let Some(ref visual_model_lock) = clip_visual_task {
                                                                let face_resized = image::imageops::resize(&face_crop, 224, 224, image::imageops::FilterType::Triangle);
                                                                let mut face_input = Array4::<f32>::zeros((1, 3, 224, 224));
                                                                for (x, y, pixel) in face_resized.enumerate_pixels() {
                                                                    face_input[[0, 0, y as usize, x as usize]] = (pixel[0] as f32 / 255.0 - 0.48145466) / 0.26862954;
                                                                    face_input[[0, 1, y as usize, x as usize]] = (pixel[1] as f32 / 255.0 - 0.4578275) / 0.2613026;
                                                                    face_input[[0, 2, y as usize, x as usize]] = (pixel[2] as f32 / 255.0 - 0.40821073) / 0.2757771;
                                                                }
                                                                if let Ok(face_tensor) = ort::value::Value::from_array(face_input) {
                                                                    if let Ok(mut visual_model) = visual_model_lock.lock() {
                                                                        if let Ok(outputs) = visual_model.run(ort::inputs!["pixel_values" => &face_tensor]) {
                                                                            if let Ok((_shape, emb_tensor)) = outputs[0].try_extract_tensor::<f32>() {
                                                                                face_embedding = vec![0.0; 512];
                                                                                face_embedding.copy_from_slice(emb_tensor);
                                                                                let norm: f32 = face_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                                                                                if norm > 0.0 { for v in face_embedding.iter_mut() { *v /= norm; } }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }

                                                            let mut assigned_person_id = None;
                                                            if !face_embedding.is_empty() {
                                                                if let Ok(mut lock) = known_people_task.lock() {
                                                                    let mut highest_similarity = 0.0f32;
                                                                    let mut best_match_id = None;
                                                                    for (person_id, person_centroid) in lock.iter() {
                                                                        let dot_product: f32 = face_embedding.iter().zip(person_centroid.iter()).map(|(a, b)| a * b).sum();
                                                                        if dot_product > highest_similarity {
                                                                            highest_similarity = dot_product;
                                                                            best_match_id = Some(person_id.clone());
                                                                        }
                                                                    }
                                                                    if highest_similarity > 0.90 {
                                                                        assigned_person_id = best_match_id;
                                                                    } else {
                                                                        let lock_db = db_task.lock().unwrap();
                                                                        let new_id = lock_db.create_anonymous_person(&face_embedding);
                                                                        lock.push((new_id.clone(), face_embedding.clone()));
                                                                        assigned_person_id = Some(new_id);
                                                                    }
                                                                }
                                                            }

                                                            let mut buffer = std::io::Cursor::new(Vec::new());
                                                            let _ = face_crop.write_to(&mut buffer, image::ImageOutputFormat::Jpeg(80));
                                                            let encoded = format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(buffer.get_ref()));
                                                            
                                                            let lock = db_task.lock().unwrap();
                                                            lock.store_face(Face { photo_id: actual_id.clone(), face_id: face_id.clone(), crop_path, encoded, embedding: face_embedding, person_id: assigned_person_id });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        drop(img); 
                    }
                }
                let current = pending_count_task.fetch_sub(1, Ordering::SeqCst);
                let _ = app_handle_task.emit("indexing-progress", current.saturating_sub(1));
            });
        }
    });
    (tx, pending_count)
}
